use mob_in_the_middle::replace_addresses;
use shared::pool::ThreadPool;
use std::{
    io::{prelude::*, BufReader, ErrorKind, Result},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(25);

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

fn handle_connection(client_stream: TcpStream) -> Result<()> {
    let proxy_stream = TcpStream::connect("chat.protohackers.com:16963")?;

    let mut proxy_reader = BufReader::new(&proxy_stream);
    let mut proxy_writer = &proxy_stream;
    let mut proxy_message = String::new();
    proxy_stream.set_nonblocking(true)?;

    let mut client_reader = BufReader::new(&client_stream);
    let mut client_writer = &client_stream;
    let mut client_message = String::new();
    client_stream.set_nonblocking(true)?;

    loop {
        match client_reader.read_line(&mut client_message) {
            Ok(0) => break,
            Ok(_) => {
                write!(
                    proxy_writer,
                    "{}",
                    replace_addresses(client_message.to_string())
                )?;
                client_message.clear();
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => break,
        }

        match proxy_reader.read_line(&mut proxy_message) {
            Ok(0) => break,
            Ok(_) => {
                write!(
                    client_writer,
                    "{}",
                    replace_addresses(proxy_message.to_string())
                )?;
                proxy_message.clear();
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
    }

    Ok(())
}
