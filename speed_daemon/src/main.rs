mod serialization;

use serialization::{iterate::consume_messages, models::{IAmCamera, IAmDispatcher, Message, MessageIterator}};
use shared::pool::ThreadPool;
use speed_daemon::{Car, Sighting, SpeedMessage};
use std::{
    collections::HashMap, io::{prelude::*, BufReader, Result}, net::{TcpListener, TcpStream}, time::{Duration, Instant}
};
use tokio::sync::broadcast;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(151);

    let (speed_broadcast, _) = broadcast::channel(2048);

    let broadcast = speed_broadcast.clone();
    pool.execute(move |_| {
        if let Err(e) = orchestrate(broadcast) {
            eprintln!("Orchestration error: {}", e);
        }
    });

    for stream in listener.incoming() {
        let stream = stream?;
        let speed_broadcast = speed_broadcast.clone();

        pool.execute(move |_| {
            let reader = BufReader::new(&stream);
            let mut messages = consume_messages(reader);

            match messages.next() {
                Some(Ok(Message::IAmCamera(camera))) => {
                    stream.set_nonblocking(true).unwrap();
                    let writer = &stream;

                    if let Err(e) = handle_camera(camera, speed_broadcast, messages, writer) {
                        eprintln!("Camera error: {}", e);
                    }
                }
                Some(Ok(Message::IAmDispatcher(dispatcher))) => {
                    stream.set_nonblocking(true).unwrap();
                    let writer = &stream;

                    if let Err(e) = handle_dispatcher(dispatcher, speed_broadcast, messages, writer) {
                        eprintln!("Camera error: {}", e);
                    }
                }
                _ => {},
            }
        })
    }

    Ok(())
}

fn orchestrate(
    speed_broadcast: broadcast::Sender<SpeedMessage>,
) -> Result<()> {
    let mut cars = HashMap::<String, Car>::new();

    let mut receiver = speed_broadcast.subscribe();

    'l: loop {
        for car in cars.values_mut() {
            for ticket in car.get_outstanding_tickets() {
                if let Err(e) = speed_broadcast.send(SpeedMessage::TicketRequired(ticket)) {
                    println!("{}", e);
                    break 'l;
                }
            }
        }

        match receiver.try_recv() {
            Ok(SpeedMessage::Sighting(message)) => {
                if let Some(car) = cars.get_mut(&message.plate.to_string()) {
                    car.add_sighting(message);
                } else {
                    let mut car = Car::new(message.plate.clone());
                    car.add_sighting(message.clone());

                    cars.insert(message.plate.clone(), car);
                }
            }
            Ok(SpeedMessage::TicketSent(message)) => {
                if let Some(car) = cars.get_mut(&message.plate.to_string()) {
                    car.mark_ticket_dispatched(message);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

fn handle_camera(
    camera: IAmCamera,
    speed_broadcast: broadcast::Sender<SpeedMessage>,
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
                speed_broadcast.send(SpeedMessage::Sighting(Sighting{
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
    speed_broadcast: broadcast::Sender<SpeedMessage>,
    mut messages: MessageIterator<&TcpStream>,
    mut writer: &TcpStream,
) -> Result<()> {
    println!("Dispatcher connected for roads {:?}", dispatcher.roads);

    let mut receiver = speed_broadcast.subscribe();
    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = Duration::from_secs(6000);

    loop {
        if last_heartbeat.elapsed() > heartbeat_interval {
            let bytes = vec![0x40];
            writer.write_all(&bytes)?;
            last_heartbeat = Instant::now();
        }

        match receiver.try_recv() {
            Ok(SpeedMessage::TicketRequired(ticket)) if dispatcher.roads.contains(&ticket.road) => {
                match bincode::serialize(&ticket) {
                    Ok(mut bytes) => {
                        bytes.insert(0, 0x21);
                        let byte = vec![0x40];
                        writer.write_all(&byte)?;
                        speed_broadcast.send(SpeedMessage::TicketSent(ticket)).unwrap();
                    },
                    Err(e) => {
                        println!("{}", e);
                        break;
                    }
                }
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
