mod helpers;
mod models;
mod read;
mod site;
mod write;

use helpers::is_valid_visit;
use models::{Hello, Message, PestControlError, SiteVisit};
use read::consume_messages;
use shared::pool::ThreadPool;
use site::Site;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{BufReader, Error, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self},
    time::Duration,
};
use write::write_message;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(100);

    let (client_sender, client_receiver) = mpsc::channel();

    pool.execute(move |_| {
        if let Err(e) = control(client_receiver) {
            eprintln!("Orchestration error: {}", e);
        }
    });

    for stream in listener.incoming() {
        let stream = stream?;
        let sender = client_sender.clone();

        pool.execute(move |_| {
            if let Err(e) = handle_client(sender, stream) {
                eprintln!("Client error: {}", e);
            }
        })
    }

    Ok(())
}

fn control(client_receiver: mpsc::Receiver<SiteVisit>) -> Result<()> {
    let pool = ThreadPool::new(100);
    let mut sites = HashMap::new();

    while let Ok(visit) = client_receiver.recv() {
        if let Entry::Vacant(entry) = sites.entry(visit.site) {
            let (site_sender, site_receiver) = mpsc::channel();
            entry.insert(site_sender);

            pool.execute(move |_| match Site::new(visit.site, site_receiver) {
                Ok(mut site) => {
                    if let Err(e) = site.poll() {
                        eprintln!("Error polling site: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error creating site: {}", e);
                }
            });
        }

        let sender = sites.get(&visit.site).unwrap();
        sender.send(visit).unwrap();
    }

    Ok(())
}

fn handle_client(client_sender: mpsc::Sender<SiteVisit>, stream: TcpStream) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(20)))?;

    let mut writer = &stream;
    let mut messages = consume_messages(BufReader::new(&stream));

    write_message(
        &mut writer,
        Message::Hello(Hello {
            protocol: "pestcontrol".to_string(),
            version: 1,
        }),
    )?;

    let valid_hello = matches!(
        messages.next(),
        Some(Ok(Message::Hello(message)))
            if message.protocol == "pestcontrol" && message.version == 1
    );

    if !valid_hello {
        write_message(
            &mut writer,
            Message::PestControlError(PestControlError {
                message: "Invalid hello".to_string(),
            }),
        )?;

        return Err(Error::from(ErrorKind::InvalidData));
    }

    while let Some(Ok(Message::SiteVisit(message))) = messages.next() {
        if !is_valid_visit(message.clone()) {
            break;
        }

        client_sender.send(message).unwrap();
    }

    write_message(
        &mut writer,
        Message::PestControlError(PestControlError {
            message: "Invalid message".to_string(),
        }),
    )?;

    Ok(())
}
