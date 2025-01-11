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

        pool.execute(|| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Connection error: {}", e);
            };
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf_reader = BufReader::new(&stream);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer)?;
    stream.write_all(&buffer)?;

    Ok(())
}
