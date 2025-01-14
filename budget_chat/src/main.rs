use budget_chat::{
    format_names, is_valid_name,
    models::{Member, Message},
};
use shared::pool::ThreadPool;
use std::{
    collections::HashSet,
    io::{prelude::*, BufReader, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(10);

    let members = Arc::new(Mutex::new(HashSet::<Member>::new()));
    let (sender, _) = broadcast::channel(2048);

    for stream in listener.incoming() {
        let stream = stream?;

        let members = Arc::clone(&members);
        let sender = sender.clone();

        pool.execute(move |thread_id| {
            if let Err(e) = handle_connection(thread_id, stream, members, sender) {
                eprintln!("Connection error: {}", e);
            }
        })
    }

    Ok(())
}

fn handle_connection(
    thread_id: usize,
    stream: TcpStream,
    members: Arc<Mutex<HashSet<Member>>>,
    sender: broadcast::Sender<Message>,
) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    let mut name = String::new();
    writeln!(writer, "Enter name:")?;
    reader.read_line(&mut name)?;

    let member = if is_valid_name(&name) {
        Member {
            id: thread_id,
            name: name.trim().to_string(),
        }
    } else {
        return Ok(());
    };

    let formatted_names = format_names(members.lock().unwrap().clone());
    writeln!(writer, "{}", formatted_names)?;
    members.lock().unwrap().insert(member.clone());

    let mut receiver = sender.subscribe();

    sender
        .send(Message {
            sender_id: member.id,
            contents: format!("* {} has entered the room", member.name),
        })
        .unwrap();

    let mut message = String::new();
    stream.set_nonblocking(true)?;

    loop {
        if let Ok(value) = receiver.try_recv() {
            if value.sender_id != member.id {
                writeln!(writer, "{}", value.contents)?;
            }
        }

        match reader.read_line(&mut message) {
            Ok(0) => break,
            Ok(_) => {
                sender
                    .send(Message {
                        sender_id: member.id,
                        contents: format!("[{}] {}", member.name, message.trim()),
                    })
                    .unwrap();
                message.clear();
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
    }

    sender
        .send(Message {
            sender_id: member.id,
            contents: format!("* {} has left the room", member.name),
        })
        .unwrap();
    members.lock().unwrap().remove(&member);

    Ok(())
}
