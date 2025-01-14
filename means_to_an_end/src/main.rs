use means_to_an_end::*;
use shared::pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, Result},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(5);

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
    let mut writer = &stream;
    let mut prices = HashMap::new();

    for message in consume_messages(reader) {
        let message = message?;

        match message.message_type {
            'I' => {
                prices.insert(message.a, message.b);
            }
            'Q' => {
                let mean = find_mean_price(message.a, message.b, &prices);
                writer.write_all(&mean.to_be_bytes())?;
            }
            _ => break,
        }
    }

    Ok(())
}
