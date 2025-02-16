use std::{
    io::{self, Error, ErrorKind},
    str,
};

#[derive(Debug)]
pub enum LcrpMessage {
    Ack {
        session_id: i32,
        len: i32,
    },
    Close {
        session_id: i32,
    },
    Connect {
        session_id: i32,
    },
    Data {
        session_id: i32,
        pos: i32,
        data: Vec<u8>,
    },
}

pub fn escape(data: String) -> String {
    let mut escaped = str::replace(&str::replace(data.as_str(), r"\", r"\\"), r"`", r"\/");
    String::truncate(&mut escaped, 950);

    escaped
}

pub fn unescape(data: String) -> String {
    str::replace(&str::replace(data.as_str(), r"\\", r"\"), r"\/", r"`")
}

pub fn parse_message(packet: &[u8]) -> io::Result<LcrpMessage> {
    let data = str::from_utf8(packet).unwrap();

    match unescape(data.to_string()).split('/').collect::<Vec<&str>>()[..] {
        ["", "connect", session_id_str, ""] => match session_id_str.parse::<i32>() {
            Ok(session_id) => Ok(LcrpMessage::Connect { session_id }),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        },
        ["", "data", session_id_str, pos_str, data, ""] => {
            match (session_id_str.parse::<i32>(), pos_str.parse::<i32>()) {
                (Ok(session_id), Ok(pos)) => Ok(LcrpMessage::Data {
                    session_id,
                    data: data.as_bytes().to_vec(),
                    pos,
                }),
                _ => Err(Error::from(ErrorKind::InvalidData)),
            }
        }
        ["", "ack", session_id_str, length_str, ""] => {
            match (session_id_str.parse::<i32>(), length_str.parse::<i32>()) {
                (Ok(session_id), Ok(len)) => Ok(LcrpMessage::Ack { session_id, len }),
                _ => Err(Error::from(ErrorKind::InvalidData)),
            }
        }
        ["", "close", session_id_str, ""] => match session_id_str.parse::<i32>() {
            Ok(session_id) => Ok(LcrpMessage::Close { session_id }),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        },
        [""] => Err(Error::from(ErrorKind::UnexpectedEof)),
        _ => Err(Error::from(ErrorKind::InvalidData)),
    }
}
