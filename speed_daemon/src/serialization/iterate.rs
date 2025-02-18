use bincode::Options;
use std::io::ErrorKind;
use std::io::{prelude::Read, BufReader, Error};

use super::models::{
    ClientError, Heartbeat, IAmCamera, IAmDispatcher, Message, MessageIterator, Plate, Ticket,
    WantHeartbeat,
};

impl<R: Read> Iterator for MessageIterator<R> {
    type Item = Result<Message, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![0; 1000];
        let serialize_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian();

        match self.reader.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => {
                let buffer = &buffer[..n];

                match buffer[0] {
                    0x10 => match serialize_options.deserialize::<ClientError>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::ClientError(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid error message",
                        )))),
                    },
                    0x20 => match serialize_options.deserialize::<Plate>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::Plate(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid plate message",
                        )))),
                    },
                    0x21 => match serialize_options.deserialize::<Ticket>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::Ticket(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid ticket message",
                        )))),
                    },
                    0x40 => match serialize_options.deserialize::<WantHeartbeat>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::WantHeartbeat(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid want heartbeat message",
                        )))),
                    },
                    0x41 => Some(Ok(Message::Heartbeat(Heartbeat {}))),
                    0x80 => match serialize_options.deserialize::<IAmCamera>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::IAmCamera(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid camera message",
                        )))),
                    },
                    0x81 => match serialize_options.deserialize::<IAmDispatcher>(&buffer[1..]) {
                        Ok(message) => Some(Ok(Message::IAmDispatcher(message))),
                        _ => Some(Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid dispatcher message",
                        )))),
                    },
                    _ => Some(Err(Box::new(Error::new(
                        ErrorKind::InvalidData,
                        "Unknown message type",
                    )))),
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(e) => Some(Err(Box::new(e))),
        }
    }
}

pub fn consume_messages<R>(reader: BufReader<R>) -> MessageIterator<R> {
    MessageIterator { reader }
}
