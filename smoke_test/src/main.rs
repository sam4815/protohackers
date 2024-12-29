use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use smoke_test::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer).expect("Error reading stream");
    stream.write_all(&buffer).unwrap();
}
