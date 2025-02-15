use std::{io::{self, Error}, str};

#[derive(Debug)]
pub enum LcrpMessage {
    Ack { session_id: i32, len: i32 },
    Close { session_id: i32 },
    Connect { session_id: i32 },
    Data { session_id: i32, pos: i32, data: Vec<u8> },
}

pub fn parse_message(packet: &[u8], session_id: i32) -> io::Result<LcrpMessage> {
    let escaped = str::from_utf8(packet).unwrap();
    println!("({}) {}", session_id, escaped);
    let mut unescaped = str::replace(escaped, "//", "/");
    unescaped = str::replace(unescaped.as_str(), r"\\", r"\");

    match unescaped.split('/').collect::<Vec<&str>>()[..] {
        ["", "connect", session_id_str, ""] => {
            match session_id_str.parse::<i32>() {
                Ok(session_id) => Ok(LcrpMessage::Connect{ session_id }),
                _ => Err(Error::other("Unable to parse connect message."))
            }
        }
        ["", "data", session_id_str, pos_str, data, ""] => {
            match (session_id_str.parse::<i32>(), pos_str.parse::<i32>()) {
                (Ok(session_id), Ok(pos)) => Ok(LcrpMessage::Data{session_id, data: data.as_bytes().to_vec(), pos}),
                _ => Err(Error::other("Unable to parse data message."))
            }
        }
        ["", "ack", session_id_str, length_str, ""] => {
            match (session_id_str.parse::<i32>(), length_str.parse::<i32>()) {
                (Ok(session_id), Ok(len)) => Ok(LcrpMessage::Ack { session_id, len }),
                _ => Err(Error::other("Unable to parse ack message."))
            }
        }
        ["", "close", session_id_str, ""] => {
            match session_id_str.parse::<i32>() {
                Ok(session_id) => Ok(LcrpMessage::Close{ session_id }),
                _ => Err(Error::other("Unable to parse close message."))
            }
        }
        _ => Err(Error::other("Unable to parse unidentified message."))
    }
}