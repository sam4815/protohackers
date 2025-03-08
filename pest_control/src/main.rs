mod checksum;
mod models;
mod read;
mod write;

use models::{Hello, Message, PestControlError, SiteVisit};
use read::consume_messages;
use shared::pool::ThreadPool;
use std::{
    io::{BufReader, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self},
};
use write::write_message;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(100);

    let (client_sender, client_receiver) = mpsc::channel();

    pool.execute(move |_| {
        if let Err(e) = orchestrate(client_receiver) {
            eprintln!("Orchestration error: {}", e);
        }
    });

    for stream in listener.incoming() {
        let stream = stream?;
        let camera_sender = client_sender.clone();

        pool.execute(move |_| {
            if let Err(e) = handle_client(camera_sender, stream) {
                eprintln!("Client error: {}", e);
            }
        })
    }

    Ok(())
}

fn orchestrate(client_receiver: mpsc::Receiver<SiteVisit>) -> Result<()> {
    Ok(())
}

fn handle_client(client_sender: mpsc::Sender<SiteVisit>, stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;
    let mut messages = consume_messages(reader);

    match messages.next() {
        Some(Ok(Message::Hello(message))) => {
            if message.protocol == "pestcontrol" && message.version == 1 {
                write_message(
                    &mut writer,
                    Message::Hello(Hello {
                        protocol: "pestcontrol".to_string(),
                        version: 1,
                    }),
                )?
            } else {
                write_message(
                    &mut writer,
                    Message::PestControlError(PestControlError {
                        message: "Invalid hello".to_string(),
                    }),
                )?
            }
        }
        Some(Ok(Message::SiteVisit(message))) => {
            client_sender.send(message).unwrap();
        }
        Some(Err(e)) if e.kind() == ErrorKind::InvalidData => write_message(
            &mut writer,
            Message::PestControlError(PestControlError {
                message: "Invalid message".to_string(),
            }),
        )?,
        _ => {}
    }

    Ok(())
}
