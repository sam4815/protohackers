mod lcrp;

use lcrp::{LrcpListener, LrcpStream};
use shared::pool::ThreadPool;
use std::io::{BufRead, BufReader, ErrorKind, Result, Write};

fn main() -> Result<()> {
    let mut listener = LrcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(100);

    for stream in listener.incoming() {
        let stream = stream?;

        pool.execute(|_| {
            if let Err(e) = handle_connection(stream) {
                eprint!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(stream: LrcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                line.pop();
                let mut response = line.chars().rev().collect::<String>();
                response.push('\n');
                writer.write_all(response.as_bytes())?;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
    }

    Ok(())
}
