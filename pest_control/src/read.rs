use std::io::ErrorKind;
use std::io::{prelude::Read, BufReader, Error, Result};
use std::str;

use crate::helpers::calculate_checksum;
use crate::models::{
    Hello, Message, MessageIterator, Okay, PestControlError, PolicyResult, SiteVisit,
    TargetPopulation, TargetPopulations, VisitPopulation,
};

fn is_valid_message(bytes_read: Vec<u8>, message_length: u32, checksum: u8) -> bool {
    let expected_checksum = calculate_checksum(&bytes_read[..bytes_read.len() - 1]);

    expected_checksum == checksum && bytes_read.len() == (message_length as usize)
}

fn read_byte<R: Read>(reader: &mut BufReader<R>, bytes_read: &mut Vec<u8>) -> std::io::Result<u8> {
    let mut buffer = [0; 1];
    reader.read_exact(&mut buffer)?;

    bytes_read.extend_from_slice(&buffer);

    Ok(buffer[0])
}

fn read_u32<R: Read>(reader: &mut BufReader<R>, bytes_read: &mut Vec<u8>) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;

    bytes_read.extend_from_slice(&buffer);

    Ok(u32::from_be_bytes(buffer))
}

fn read_string<R: Read>(
    reader: &mut BufReader<R>,
    bytes_read: &mut Vec<u8>,
) -> std::io::Result<String> {
    let string_length = read_u32(reader, bytes_read)?;

    let mut string_buffer = vec![0; string_length as usize];
    reader.read_exact(&mut string_buffer)?;

    bytes_read.extend_from_slice(&string_buffer);

    match str::from_utf8(&string_buffer) {
        Ok(string) => Ok(string.to_string()),
        Err(_) => Err(Error::from(ErrorKind::InvalidData)),
    }
}

fn read_target_populations<R: Read>(
    reader: &mut BufReader<R>,
    bytes_read: &mut Vec<u8>,
) -> std::io::Result<Vec<TargetPopulation>> {
    let vec_size = read_u32(reader, bytes_read)?;
    let mut target_populations = Vec::new();

    for _ in 0..vec_size as usize {
        let species = read_string(reader, bytes_read)?;
        let min = read_u32(reader, bytes_read)?;
        let max = read_u32(reader, bytes_read)?;

        target_populations.push(TargetPopulation { species, min, max })
    }

    Ok(target_populations)
}

fn read_visit_populations<R: Read>(
    reader: &mut BufReader<R>,
    bytes_read: &mut Vec<u8>,
) -> std::io::Result<Vec<VisitPopulation>> {
    let vec_size = read_u32(reader, bytes_read)?;
    let mut visit_populations = Vec::new();

    for _ in 0..vec_size as usize {
        let species = read_string(reader, bytes_read)?;
        let count = read_u32(reader, bytes_read)?;

        visit_populations.push(VisitPopulation { species, count })
    }

    Ok(visit_populations)
}

impl<R: Read> Iterator for MessageIterator<R> {
    type Item = Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes_read = Vec::new();

        match read_byte(&mut self.reader, &mut bytes_read) {
            Ok(0x50) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let protocol = read_string(&mut self.reader, &mut bytes_read).ok()?;
                let version = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid Hello message",
                    )));
                }

                Some(Ok(Message::Hello(Hello { protocol, version })))
            }
            Ok(0x51) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let message = read_string(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid PestControlError message",
                    )));
                }

                Some(Ok(Message::PestControlError(PestControlError { message })))
            }
            Ok(0x52) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid Okay message",
                    )));
                }

                Some(Ok(Message::Okay(Okay {})))
            }
            Ok(0x54) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let site = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let populations =
                    read_target_populations(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid TargetPopulations message",
                    )));
                }

                Some(Ok(Message::TargetPopulations(TargetPopulations {
                    site,
                    populations,
                })))
            }
            Ok(0x57) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let policy = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid PolicyResult message",
                    )));
                }

                Some(Ok(Message::PolicyResult(PolicyResult { policy })))
            }
            Ok(0x58) => {
                let message_length = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let site = read_u32(&mut self.reader, &mut bytes_read).ok()?;
                let populations = read_visit_populations(&mut self.reader, &mut bytes_read).ok()?;
                let checksum = read_byte(&mut self.reader, &mut bytes_read).ok()?;

                if !is_valid_message(bytes_read, message_length, checksum) {
                    return Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid SiteVisit message",
                    )));
                }

                Some(Ok(Message::SiteVisit(SiteVisit { site, populations })))
            }
            Ok(_) => Some(Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid starting byte",
            ))),
            Err(e) => Some(Err(e)),
        }
    }
}

pub fn consume_messages<R>(reader: BufReader<R>) -> MessageIterator<R> {
    MessageIterator { reader }
}
