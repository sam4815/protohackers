mod models;
mod write;
mod read;
mod car;

use car::Car;
use read::consume_messages;
use models::{IAmCamera, IAmDispatcher, Message, MessageIterator, Sighting};
use write::{write_heartbeat, write_ticket};
use shared::pool::ThreadPool;
use std::{
    collections::HashMap, io::{prelude::*, BufReader, Result}, net::{TcpListener, TcpStream}, sync::{mpsc, Arc, Mutex}, time::{Duration, Instant}
};

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
            let reader = BufReader::new(&stream);
            let mut messages = consume_messages(reader);

            match messages.next() {
                Some(Ok(Message::IAmCamera(camera))) => {
                    stream.set_nonblocking(true).unwrap();
                    let writer = &stream;

                    if let Err(e) = handle_camera(camera, camera_sender, messages, writer) {
                        eprintln!("Camera error: {}", e);
                    }
                }
                Some(Ok(Message::IAmDispatcher(dispatcher))) => {
                    stream.set_nonblocking(true).unwrap();
                    let writer = &stream;

                    let (dispatch_sender, dispatch_receiver) = mpsc::channel();
                    for road in dispatcher.roads.iter() {
                        dispatchers.lock().unwrap().insert(*road, dispatch_sender.clone());
                    }

                    if let Err(e) = handle_dispatcher(dispatcher, dispatch_receiver, messages, writer) {
                        eprintln!("Camera error: {}", e);
                    }
                }
                Some(Err(e)) => {
                    println!("{:?}", e);
                },
                _ => {}
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

        match camera_receiver.try_recv() {
            Ok(Message::Sighting(message)) => {
                if let Some(car) = cars.get_mut(&message.plate.to_string()) {
                    car.add_sighting(message);
                } else {
                    let mut car = Car::new(message.plate.clone());
                    car.add_sighting(message.clone());

                    cars.insert(message.plate.clone(), car);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

fn handle_camera(
    camera: IAmCamera,
    camera_sender: mpsc::Sender<Message>,
    mut messages: MessageIterator<&TcpStream>,
    mut writer: &TcpStream,
) -> Result<()> {
    println!("Camera connected for road {}", camera.road);

    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = Duration::from_secs(6000);

    loop {
        if last_heartbeat.elapsed() > heartbeat_interval {
            let bytes = vec![0x40];
            writer.write_all(&bytes)?;
            last_heartbeat = Instant::now();
        }

        match messages.next() {
            Some(Ok(Message::Plate(message))) => {
                camera_sender.send(Message::Sighting(Sighting{
                    plate: message.plate.to_string(),
                    timestamp: message.timestamp,
                    road: camera.road,
                    mile: camera.mile,
                    limit: camera.limit,
                })).unwrap();
            }
            Some(Ok(Message::WantHeartbeat(message))) => {
                heartbeat_interval = Duration::from_secs((message.interval / 10).into());
            }
            Some(Ok(message)) => {
                println!("Camera unexpectedly received {:?}", message);
                break;
            }
            Some(Err(e)) => {
                println!("{}", e);
                break;
            }
            None => {}
        }
    }

    println!("Camera for road {} disconnecting", camera.road);

    Ok(())
}

fn handle_dispatcher(
    dispatcher: IAmDispatcher,
    dispatch_receiver: mpsc::Receiver<Message>,
    mut messages: MessageIterator<&TcpStream>,
    mut writer: &TcpStream,
) -> Result<()> {
    println!("Dispatcher connected for roads {:?}", dispatcher.roads);

    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = Duration::from_secs(6000);

    loop {
        if last_heartbeat.elapsed() > heartbeat_interval {
            write_heartbeat(&mut writer)?;
            last_heartbeat = Instant::now();
        }

        match dispatch_receiver.try_recv() {
            Ok(Message::Ticket(ticket)) if dispatcher.roads.contains(&ticket.road) => {
                write_ticket(&mut writer, ticket)?
            }
            _ => (),
        }

        match messages.next() {
            Some(Ok(Message::WantHeartbeat(message))) => {
                heartbeat_interval = Duration::from_secs((message.interval / 10).into());
            }
            Some(Ok(message)) => {
                println!("Disaptcher unexpectedly received {:?}", message);
                break;
            }
            Some(Err(e)) => {
                println!("{}", e);
                break;
            }
            None => {}
        }
    }

    println!("Dispatcher for roads {:?} disconnecting", dispatcher.roads);

    Ok(())
}
