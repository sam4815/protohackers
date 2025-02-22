use std::io::ErrorKind;
use std::io::{prelude::Read, BufReader, Error};
use std::str;

use crate::models::{Heartbeat, IAmCamera, IAmDispatcher, Message, MessageIterator, Plate};

fn read_char<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<u8> {
    let mut buffer = [0; 1];
    reader.read_exact(&mut buffer)?;
    
    Ok(buffer[0])
}

fn read_string<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<String> {
    let mut length_buffer = [0; 1];
    reader.read_exact(&mut length_buffer)?;
    
    let mut string_buffer = vec![0; length_buffer[0] as usize];
    reader.read_exact(&mut string_buffer)?;

    match str::from_utf8(&string_buffer) {
        Ok(string) => Ok(string.to_string()),
        Err(_) => Err(Error::from(ErrorKind::InvalidData))
    }
}

fn read_vec<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<Vec<u16>> {
    let mut length_buffer = [0; 1];
    reader.read_exact(&mut length_buffer)?;
    
    let mut numbers = Vec::new();
    for _ in 0..length_buffer[0] as usize {
        let mut buffer = [0u8; 2];
        reader.read_exact(&mut buffer)?;
        numbers.insert(0, u16::from_be_bytes(buffer))
    }

    Ok(numbers)
}

fn read_u32<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;

    Ok(u32::from_be_bytes(buffer))
}

fn read_u16<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<u16> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;

    Ok(u16::from_be_bytes(buffer))
}

impl<R: Read> Iterator for MessageIterator<R> {
    type Item = Result<Message, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_char(&mut self.reader) {
            Ok(0x20) => {
                let plate = read_string(&mut self.reader).ok()?;
                let timestamp = read_u32(&mut self.reader).ok()?;

                Some(Ok(Message::Plate(Plate{ plate, timestamp })))
            },
            Ok(0x41) => {
                Some(Ok(Message::Heartbeat(Heartbeat{})))
            },
            Ok(0x80) => {
                let road = read_u16(&mut self.reader).ok()?;
                let mile = read_u16(&mut self.reader).ok()?;
                let limit = read_u16(&mut self.reader).ok()?;

                Some(Ok(Message::IAmCamera(IAmCamera{ road, mile, limit })))
            },
            Ok(0x81) => {
                let roads = read_vec(&mut self.reader).ok()?;

                Some(Ok(Message::IAmDispatcher(IAmDispatcher{ roads })))
            },
            Ok(_) => Some(Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                "Unknown message type",
            )))),
            Err(e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(e) => Some(Err(Box::new(e))),
        }
    }
}

pub fn consume_messages<R>(reader: BufReader<R>) -> MessageIterator<R> {
    MessageIterator { reader }
}
