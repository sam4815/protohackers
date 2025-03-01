use core::str;
use std::io::{prelude::Read, BufReader, Result};

#[derive(Debug)]
pub enum Operation {
    ReverseBits,
    XorN(u8),
    XorPos,
    AddN(u8),
    AddPos,
}

pub struct Cipher {
    pub spec: Vec<Operation>,
    server_pos: i32,
    client_pos: i32,
}

impl Cipher {
    pub fn new<R: Read>(reader: &mut BufReader<R>) -> Result<Cipher> {
        let mut spec: Vec<Operation> = Vec::new();
        let mut buffer = [0; 1];

        loop {
            reader.read_exact(&mut buffer)?;

            match buffer[0] {
                0x00 => break,
                0x01 => spec.push(Operation::ReverseBits),
                0x02 => {
                    reader.read_exact(&mut buffer)?;
                    spec.push(Operation::XorN(buffer[0]));
                }
                0x03 => spec.push(Operation::XorPos),
                0x04 => {
                    reader.read_exact(&mut buffer)?;
                    spec.push(Operation::AddN(buffer[0]));
                }
                0x05 => spec.push(Operation::AddPos),
                _ => {}
            }
        }

        Ok(Cipher {
            spec,
            server_pos: -1,
            client_pos: -1,
        })
    }

    fn encode_byte(&mut self, byte: u8) -> u8 {
        self.client_pos += 1;

        self.spec
            .iter()
            .fold(byte, |encoded, operation| match operation {
                Operation::ReverseBits => encoded.reverse_bits(),
                Operation::XorN(n) => encoded ^ n,
                Operation::XorPos => encoded ^ (self.client_pos & 0xFF) as u8,
                Operation::AddN(n) => encoded.wrapping_add(*n),
                Operation::AddPos => encoded.wrapping_add((self.client_pos & 0xFF) as u8),
            })
    }

    fn decode_byte(&mut self, byte: u8) -> u8 {
        self.server_pos += 1;

        self.spec
            .iter()
            .rev()
            .fold(byte, |decoded, operation| match operation {
                Operation::ReverseBits => decoded.reverse_bits(),
                Operation::XorN(n) => decoded ^ n,
                Operation::XorPos => decoded ^ (self.server_pos & 0xFF) as u8,
                Operation::AddN(n) => decoded.wrapping_sub(*n),
                Operation::AddPos => decoded.wrapping_sub((self.server_pos & 0xFF) as u8),
            })
    }

    pub fn encode_string(&mut self, str: String) -> Vec<u8> {
        str.as_bytes()
            .iter()
            .map(|byte| self.encode_byte(*byte))
            .collect()
    }

    pub fn decode_line<R: Read>(&mut self, reader: &mut BufReader<R>) -> Result<String> {
        let mut decoded_bytes = Vec::new();
        let mut buffer = [0; 1];

        while decoded_bytes.last() != Some(&b'\n') {
            reader.read_exact(&mut buffer)?;
            decoded_bytes.push(self.decode_byte(buffer[0]));
        }

        match str::from_utf8(&decoded_bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Ok("".to_string()),
        }
    }

    pub fn is_redundant(&mut self) -> bool {
        let test_word = "abcdefg".to_string();
        let encoded = self.encode_string(test_word.clone());

        self.client_pos = -1;

        match str::from_utf8(&encoded) {
            Ok(s) => s == test_word.clone(),
            _ => false,
        }
    }
}
