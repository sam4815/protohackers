mod models;

use models::{Request, Response};
use prime_time::PrimeCheck;
use shared::pool::ThreadPool;
use std::{
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

    for line in reader.lines() {
        let line = line?;

        let request: Request = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Malformed JSON: {}", error);
                break;
            }
        };

        if request.method != "isPrime" {
            eprintln!("Unexpected method: {}", request.method);
            break;
        }

        let response = Response {
            method: request.method,
            prime: match request.number.as_i64() {
                Some(n) => n.is_prime(),
                _ => false,
            },
        };

        let response_json = serde_json::to_string(&response)?;
        writeln!(writer, "{}", response_json)?;
    }

    Ok(())
}
