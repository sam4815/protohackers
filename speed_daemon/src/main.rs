mod serialization;

use serialization::{iterate::consume_messages, models::Message};
use shared::pool::ThreadPool;
use std::{
    io::{BufReader, Result},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(150);

    for stream in listener.incoming() {
        let stream = stream?;

        pool.execute(|_| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Connection error: {}", e);
            }
        })
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);

    for message in consume_messages(reader) {
        match message {
            Ok(Message::ClientError(error)) => {
                println!("{:?}", error);
            }
            Ok(Message::Plate(plate)) => {
                println!("{:?}", plate);
            }
            Ok(Message::Ticket(ticket)) => {
                println!("{:?}", ticket);
            }
            Ok(Message::WantHeartbeat(want)) => {
                println!("{:?}", want);
            }
            Ok(Message::Heartbeat(heartbeat)) => {
                println!("{:?}", heartbeat);
            }
            Ok(Message::IAmCamera(camera)) => {
                println!("{:?}", camera);
            }
            Ok(Message::IAmDispatcher(dispatcher)) => {
                println!("{:?}", dispatcher);
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }

    Ok(())
}
