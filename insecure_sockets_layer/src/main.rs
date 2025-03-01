mod cipher;

use cipher::Cipher;
use shared::pool::ThreadPool;
use std::{
    io::{prelude::*, BufReader, Error, ErrorKind, Result},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        let stream = stream?;

        pool.execute(|_| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Connection error: {}", e);
            };
        });
    }

    Ok(())
}

fn parse_int(str: &str) -> usize {
    let digits = str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>();
    digits.parse::<usize>().unwrap()
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    let mut cipher = Cipher::new(&mut reader)?;

    if cipher.is_redundant() {
        return Err(Error::from(ErrorKind::InvalidData));
    }

    while let Ok(line) = cipher.decode_line(&mut reader) {
        let toys = line.trim().split(',');
        let mut max_toy = toys
            .max_by(|a, b| parse_int(a).cmp(&parse_int(b)))
            .unwrap()
            .to_string();
        max_toy.push('\n');

        let encoded = cipher.encode_string(max_toy);
        writer.write_all(&encoded)?;
    }

    Ok(())
}
