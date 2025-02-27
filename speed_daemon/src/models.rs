use std::io::BufReader;

#[derive(Clone, PartialEq, Debug)]
pub struct ClientError {
    pub message: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Plate {
    pub plate: String,
    pub timestamp: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ticket {
    pub plate: String,
    pub road: u16,
    pub mile1: u16,
    pub timestamp1: u32,
    pub mile2: u16,
    pub timestamp2: u32,
    pub speed: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Sighting {
    pub plate: String,
    pub timestamp: u32,
    pub road: u16,
    pub mile: u16,
    pub limit: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WantHeartbeat {
    pub interval: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Heartbeat {}

#[derive(Clone, PartialEq, Debug)]
pub struct IAmCamera {
    pub road: u16,
    pub mile: u16,
    pub limit: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IAmDispatcher {
    pub roads: Vec<u16>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Plate(Plate),
    Ticket(Ticket),
    WantHeartbeat(WantHeartbeat),
    IAmCamera(IAmCamera),
    IAmDispatcher(IAmDispatcher),
    Sighting(Sighting),
}

pub struct MessageIterator<R> {
    pub reader: BufReader<R>,
}
