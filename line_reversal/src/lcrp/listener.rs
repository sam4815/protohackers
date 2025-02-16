use crate::lcrp::message::{parse_message, LcrpMessage};
use crate::lcrp::stream::LrcpStream;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    str, thread,
    time::{Duration, Instant},
};

pub struct LrcpListener {
    pub socket: UdpSocket,
    pub sessions: HashMap<i32, Instant>,
}

pub struct Incoming<'a> {
    pub listener: &'a mut LrcpListener,
}

impl LrcpListener {
    pub fn bind(address: &str) -> io::Result<LrcpListener> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;

        Ok(LrcpListener {
            socket,
            sessions: HashMap::new(),
        })
    }

    pub fn incoming(&mut self) -> Incoming {
        Incoming { listener: self }
    }

    pub fn accept(&mut self) -> io::Result<LrcpStream> {
        loop {
            thread::sleep(Duration::from_millis(5));
            self.sessions
                .retain(|_, v| v.elapsed() < Duration::from_secs(30));

            let mut buf = vec![0; 1000];
            let (amt, src) = match self.socket.peek_from(&mut buf) {
                Ok((0, _)) => {
                    self.socket.recv_from(&mut buf)?;
                    continue;
                }
                Ok(value) => value,
                Err(_) => (
                    0,
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                ),
            };

            match parse_message(&buf[..amt]) {
                Ok(LcrpMessage::Connect { session_id }) => {
                    self.socket.recv_from(&mut buf)?;

                    let response = format!("/ack/{}/0/", session_id);
                    self.socket.send_to(response.to_string().as_bytes(), src)?;

                    let clone = self.socket.try_clone()?;

                    if self.sessions.contains_key(&session_id) {
                        continue;
                    }

                    self.sessions.insert(session_id, Instant::now());

                    let stream = LrcpStream {
                        session_id,
                        src,
                        socket: clone,
                        ack: RefCell::new(0),
                        llen: RefCell::new(0),
                        received: RefCell::new("".to_string()),
                        sent: RefCell::new("".to_string()),
                    };

                    return Ok(stream);
                }
                Ok(LcrpMessage::Close { ref session_id })
                    if self.sessions.contains_key(session_id) =>
                {
                    self.socket.recv_from(&mut buf)?;
                    self.sessions.remove(session_id);
                }
                Ok(
                    LcrpMessage::Ack { ref session_id, .. }
                    | LcrpMessage::Data { ref session_id, .. },
                ) if self.sessions.contains_key(session_id) => {
                    self.sessions.insert(*session_id, Instant::now());
                }
                Ok(
                    LcrpMessage::Ack { ref session_id, .. }
                    | LcrpMessage::Close { ref session_id }
                    | LcrpMessage::Data { ref session_id, .. },
                ) if !self.sessions.contains_key(session_id) => {
                    self.socket.recv_from(&mut buf)?;
                    let response = format!("/close/{}/", session_id);
                    self.socket.send_to(response.to_string().as_bytes(), src)?;
                }
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    self.socket.recv_from(&mut buf)?;
                }
                _ => {}
            }
        }
    }
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<LrcpStream>;
    fn next(&mut self) -> Option<io::Result<LrcpStream>> {
        Some(self.listener.accept())
    }
}
