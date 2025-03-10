use std::io::{Error, ErrorKind, Write};

use crate::{helpers::calculate_checksum, models::Message};

fn write_byte(bytes_to_write: &mut Vec<u8>, character: u8) {
    bytes_to_write.push(character);
}

fn write_string(bytes_to_write: &mut Vec<u8>, string: String) {
    bytes_to_write.extend_from_slice(&(string.clone().len() as u32).to_be_bytes());
    bytes_to_write.extend_from_slice(&string.into_bytes());
}

fn write_u32(bytes_to_write: &mut Vec<u8>, number: u32) {
    bytes_to_write.extend_from_slice(&number.to_be_bytes());
}

fn write_length(bytes_to_write: &mut Vec<u8>) {
    let message_length = (bytes_to_write.len() + 5) as u32;

    bytes_to_write.splice(1..1, message_length.to_be_bytes());
}

fn write_checksum(bytes_to_write: &mut Vec<u8>) {
    bytes_to_write.push(calculate_checksum(bytes_to_write));
}

pub fn write_message(writer: &mut impl Write, message: Message) -> std::io::Result<()> {
    let mut bytes_to_write = Vec::new();

    match message {
        Message::Hello(message) => {
            write_byte(&mut bytes_to_write, 0x50);
            write_string(&mut bytes_to_write, message.protocol);
            write_u32(&mut bytes_to_write, message.version);
            write_length(&mut bytes_to_write);
            write_checksum(&mut bytes_to_write);

            writer.write_all(&bytes_to_write)
        }
        Message::PestControlError(message) => {
            write_byte(&mut bytes_to_write, 0x51);
            write_string(&mut bytes_to_write, message.message);
            write_length(&mut bytes_to_write);
            write_checksum(&mut bytes_to_write);

            writer.write_all(&bytes_to_write)
        }
        Message::DialAuthority(message) => {
            write_byte(&mut bytes_to_write, 0x53);
            write_u32(&mut bytes_to_write, message.site);
            write_length(&mut bytes_to_write);
            write_checksum(&mut bytes_to_write);

            writer.write_all(&bytes_to_write)
        }
        Message::CreatePolicy(message) => {
            write_byte(&mut bytes_to_write, 0x55);
            write_string(&mut bytes_to_write, message.species);
            write_byte(&mut bytes_to_write, message.action);
            write_length(&mut bytes_to_write);
            write_checksum(&mut bytes_to_write);

            writer.write_all(&bytes_to_write)
        }
        Message::DeletePolicy(message) => {
            write_byte(&mut bytes_to_write, 0x56);
            write_u32(&mut bytes_to_write, message.policy);
            write_length(&mut bytes_to_write);
            write_checksum(&mut bytes_to_write);

            writer.write_all(&bytes_to_write)
        }
        _ => Err(Error::from(ErrorKind::InvalidInput)),
    }
}
