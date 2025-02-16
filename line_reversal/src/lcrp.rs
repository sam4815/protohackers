#[path = "./parse.rs"]
mod parse;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Error, ErrorKind, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    str, thread,
    time::Duration,
};

use parse::{escape, parse_message, LcrpMessage};

pub struct LrcpListener {
    pub socket: UdpSocket,
    pub sessions: HashMap<i32, i32>,
}

pub struct Incoming<'a> {
    pub listener: &'a mut LrcpListener,
}

pub struct LrcpStream {
    pub session_id: i32,
    pub socket: UdpSocket,
    pub src: SocketAddr,
    pub ack: RefCell<i32>,
    pub llen: RefCell<i32>,
    pub received: RefCell<String>,
    pub sent: RefCell<String>,
    pub open: RefCell<bool>,
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
            let mut buf = vec![0; 10000];
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

                    self.sessions.insert(session_id, 0);

                    let stream = LrcpStream {
                        session_id,
                        src,
                        socket: clone,
                        ack: RefCell::new(0),
                        llen: RefCell::new(0),
                        open: RefCell::new(true),
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
                    self.increment_all();
                    self.sessions.insert(*session_id, 0);
                }
                Ok(
                    LcrpMessage::Ack { ref session_id, .. }
                    | LcrpMessage::Close { ref session_id }
                    | LcrpMessage::Data { ref session_id, .. },
                ) if !self.sessions.contains_key(session_id) => {
                    self.increment_all();
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

    pub fn increment_all(&mut self) {
        for value in self.sessions.values_mut() {
            *value += 1;
        }

        self.sessions.retain(|_, v| *v < 25000);
    }
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<LrcpStream>;
    fn next(&mut self) -> Option<io::Result<LrcpStream>> {
        Some(self.listener.accept())
    }
}

impl LrcpStream {
    fn poll(&self) -> io::Result<()> {
        thread::sleep(Duration::from_millis(5));

        let mut lrcp_buf = vec![0; 10000];
        let (amt, _) = match self.socket.peek_from(&mut lrcp_buf) {
            Ok(value) => value,
            Err(_) => (
                0,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            ),
        };

        match parse_message(&lrcp_buf[..amt]) {
            Ok(LcrpMessage::Data {
                session_id,
                data,
                pos,
            }) if session_id == self.session_id => {
                self.socket.recv_from(&mut lrcp_buf)?;

                let mut ack = self.ack.borrow_mut();

                if pos > *ack {
                    let response = format!("/ack/{}/{}/", session_id, ack);
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                }

                if pos < *ack && (pos + data.len() as i32) > *ack {
                    let response = format!("/ack/{}/{}/", session_id, ack);
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                }

                if pos == *ack {
                    self.received
                        .borrow_mut()
                        .push_str(str::from_utf8(&data).unwrap());
                    *ack += data.len() as i32;

                    let response = format!("/ack/{}/{}/", session_id, ack);
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                }

                Ok(())
            }
            Ok(LcrpMessage::Ack { session_id, len }) if session_id == self.session_id => {
                self.socket.recv_from(&mut lrcp_buf)?;

                if len > self.sent.borrow().len() as i32 {
                    let response = format!("/close/{}/", session_id);
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                    return Err(Error::from(ErrorKind::ConnectionAborted));
                }

                let mut llen = self.llen.borrow_mut();
                *llen = len;

                if len < self.sent.borrow().len() as i32 {
                    let response = format!(
                        "/data/{}/{}/{}/",
                        self.session_id,
                        len,
                        escape(self.sent.borrow()[(len as usize)..].to_string()),
                    );
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                    thread::sleep(Duration::from_millis(10));
                }

                Ok(())
            }
            Ok(LcrpMessage::Close { session_id }) if session_id == self.session_id => {
                let response = format!("/close/{}/", self.session_id);
                self.socket
                    .send_to(response.to_string().as_bytes(), self.src)?;

                Err(Error::from(ErrorKind::ConnectionAborted))
            }
            _ => {
                let llen = self.llen.borrow();

                if *llen < self.sent.borrow().len() as i32 {
                    let response = format!(
                        "/data/{}/{}/{}/",
                        self.session_id,
                        llen,
                        escape(self.sent.borrow()[(*llen as usize)..].to_string()),
                    );
                    self.socket
                        .send_to(response.to_string().as_bytes(), self.src)?;
                }

                Ok(())
            }
        }
    }
}

impl Read for &LrcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.poll() {
            Ok(()) => {
                let mut received = self.received.borrow_mut();

                match received.rfind('\n') {
                    Some(n) => {
                        buf[..=n].copy_from_slice(received[..=n].as_bytes());
                        received.drain(..=n);
                        Ok(n + 1)
                    }
                    _ => Err(Error::from(ErrorKind::WouldBlock)),
                }
            }
            Err(_) => Ok(0),
        }
    }
}

impl Write for &LrcpStream {
    fn write(&mut self, write_buf: &[u8]) -> io::Result<usize> {
        let data = str::from_utf8(write_buf).unwrap();

        let response = format!(
            "/data/{}/{}/{}/",
            self.session_id,
            self.sent.borrow().len(),
            escape(data.to_string())
        );
        self.socket
            .send_to(response.to_string().as_bytes(), self.src)?;

        self.sent.borrow_mut().push_str(data);

        Ok(write_buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
