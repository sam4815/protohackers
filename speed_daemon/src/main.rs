mod car;
mod models;
mod read;
mod write;

use car::Car;
use models::{IAmCamera, Message, Sighting};
use read::consume_messages;
use shared::pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{BufReader, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use write::{write_error, write_heartbeat, write_ticket};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(151);

    let (camera_sender, camera_receiver) = mpsc::channel();
    let dispatchers = Arc::new(Mutex::new(HashMap::<u16, mpsc::Sender<Message>>::new()));
    let orch_dispatchers = Arc::clone(&dispatchers);

    pool.execute(move |_| {
        if let Err(e) = orchestrate(orch_dispatchers, camera_receiver) {
            eprintln!("Orchestration error: {}", e);
        }
    });

    for stream in listener.incoming() {
        let stream = stream?;
        let dispatchers = Arc::clone(&dispatchers);
        let camera_sender = camera_sender.clone();

        pool.execute(move |_| {
            if let Err(e) = handle_client(camera_sender, dispatchers, stream) {
                eprintln!("Client error: {}", e);
            }
        })
    }

    Ok(())
}

fn orchestrate(
    dispatch_senders: Arc<Mutex<HashMap<u16, mpsc::Sender<Message>>>>,
    camera_receiver: mpsc::Receiver<Message>,
) -> Result<()> {
    let mut cars = HashMap::<String, Car>::new();

    'l: loop {
        for car in cars.values_mut() {
            for ticket in car.get_outstanding_tickets() {
                if let Some(sender) = dispatch_senders.lock().unwrap().get(&ticket.road) {
                    if let Err(e) = sender.send(Message::Ticket(ticket.clone())) {
                        println!("{}", e);
                        break 'l;
                    }
                    car.mark_ticket_dispatched(ticket.clone());
                }
            }
        }

        if let Ok(Message::Sighting(message)) = camera_receiver.try_recv() {
            if let Some(car) = cars.get_mut(&message.plate.to_string()) {
                car.add_sighting(message);
            } else {
                let mut car = Car::new(message.plate.clone());
                car.add_sighting(message.clone());

                cars.insert(message.plate.clone(), car);
            }
        }
    }

    Ok(())
}

fn handle_client(
    camera_sender: mpsc::Sender<Message>,
    dispatchers: Arc<Mutex<HashMap<u16, Sender<Message>>>>,
    stream: TcpStream,
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;

    let reader = BufReader::new(&stream);
    let mut writer = &stream;
    let mut messages = consume_messages(reader);

    let mut some_camera: Option<IAmCamera> = None;
    let mut some_dispatcher: Option<mpsc::Receiver<Message>> = None;
    let mut some_heartbeat_interval: Option<Duration> = None;
    let mut last_heartbeat = Instant::now();

    loop {
        if let Some(ref mut interval) = some_heartbeat_interval {
            if last_heartbeat.elapsed() > *interval {
                write_heartbeat(&mut writer)?;
                last_heartbeat = Instant::now();
            }
        }

        if let Some(ref mut receiver) = some_dispatcher {
            if let Ok(Message::Ticket(ticket)) = receiver.try_recv() {
                write_ticket(&mut writer, ticket.clone())?;
                continue;
            }
        }

        match messages.next() {
            Some(Ok(Message::IAmCamera(message))) => {
                if some_dispatcher.is_none() && some_camera.is_none() {
                    some_camera = Some(message.clone());
                } else {
                    write_error(&mut writer, "Client already identified".to_string())?
                }
            }
            Some(Ok(Message::IAmDispatcher(message))) => {
                if some_dispatcher.is_none() && some_camera.is_none() {
                    let (sender, receiver) = mpsc::channel();
                    some_dispatcher = Some(receiver);
                    for road in message.roads.iter() {
                        dispatchers.lock().unwrap().insert(*road, sender.clone());
                    }
                } else {
                    write_error(&mut writer, "Client already identified".to_string())?
                }
            }
            Some(Ok(Message::Plate(message))) => {
                if let Some(ref mut camera) = some_camera {
                    camera_sender
                        .send(Message::Sighting(Sighting {
                            plate: message.plate.to_string(),
                            timestamp: message.timestamp,
                            road: camera.road,
                            mile: camera.mile,
                            limit: camera.limit,
                        }))
                        .ok();
                } else {
                    write_error(&mut writer, "Unknown client sending plate".to_string())?
                }
            }
            Some(Ok(Message::WantHeartbeat(message))) => {
                if some_heartbeat_interval.is_none() {
                    if message.interval > 0 {
                        some_heartbeat_interval =
                            Some(Duration::from_millis((message.interval * 100).into()));
                    }
                } else {
                    write_error(&mut writer, "Heartbeat already set".to_string())?
                }
            }
            Some(Err(e)) if e.kind() == ErrorKind::InvalidData => {
                write_error(&mut writer, "Invalid message".to_string())?
            }
            Some(Err(_)) => break,
            _ => {}
        }
    }

    Ok(())
}
