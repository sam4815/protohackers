use serde::Deserialize;
use std::io::BufReader;

#[derive(PartialEq, Debug)]
pub struct LengthPrefixedString(pub String);

#[derive(PartialEq, Debug)]
pub struct LengthPrefixedVector(pub Vec<u16>);

#[derive(Deserialize, PartialEq, Debug)]
pub struct ClientError {
    pub message: LengthPrefixedString,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Plate {
    pub plate: LengthPrefixedString,
    pub timestamp: u32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Ticket {
    pub plate: LengthPrefixedString,
    pub road: u16,
    pub mile1: u16,
    pub timestamp1: u32,
    pub mile2: u16,
    pub timestamp2: u32,
    pub speed: u16,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct WantHeartbeat {
    pub interval: u32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Heartbeat {}

#[derive(Deserialize, PartialEq, Debug)]
pub struct IAmCamera {
    pub road: u16,
    pub mile: u16,
    pub limit: u16,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct IAmDispatcher {
    pub roads: LengthPrefixedVector,
}

#[derive(Deserialize, Debug)]
pub enum Message {
    ClientError(ClientError),
    Plate(Plate),
    Ticket(Ticket),
    WantHeartbeat(WantHeartbeat),
    Heartbeat(Heartbeat),
    IAmCamera(IAmCamera),
    IAmDispatcher(IAmDispatcher),
}

pub struct MessageIterator<R> {
    pub reader: BufReader<R>,
}
