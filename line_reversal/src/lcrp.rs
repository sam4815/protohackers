#[path = "./parse.rs"] mod parse;
use std::{cell::RefCell, collections::HashSet, io::{self, Error, ErrorKind, Read, Write}, net::{SocketAddr, UdpSocket}, str, thread, time::Duration, usize};

use parse::{parse_message, LcrpMessage};

pub struct LrcpListener {
    pub socket: UdpSocket,
    pub sessions: HashSet<i32>,
}

pub struct Incoming<'a> {
    pub listener: &'a mut LrcpListener,
}

pub struct LrcpStream {
    pub session_id: i32,
    pub socket: UdpSocket,
    pub src: SocketAddr,
    pub ack: RefCell<i32>,
    pub received: RefCell<String>,
    pub sent: RefCell<String>,
    pub open: RefCell<bool>,
    pub last_seen: RefCell<i32>,
}

impl LrcpListener {
    pub fn bind(address: &str) -> io::Result<LrcpListener> {
        let socket = UdpSocket::bind(address)?;

        Ok(LrcpListener{socket, sessions: HashSet::new()})
    }

    pub fn incoming(&mut self) -> Incoming {
        Incoming { listener: self }
    }

    pub fn accept(&mut self) -> io::Result<LrcpStream> {
        loop {
            thread::sleep(Duration::from_millis(250));
            let mut buf = vec![0; 1000];
            let (amt, src) = self.socket.peek_from(&mut buf)?;
            
            match parse_message(&buf[..amt], 0) {
                Ok(LcrpMessage::Connect{ session_id }) => {
                    self.socket.recv_from(&mut buf)?;

                    let response = format!("/ack/{}/0/", session_id);
                    self.socket.send_to(response.to_string().as_bytes(), src)?;
                    self.socket.set_read_timeout(Some(Duration::from_secs(10)))?;
                    
                    let clone = self.socket.try_clone()?;

                    if self.sessions.contains(&session_id) {
                        continue;
                    }

                    self.sessions.insert(session_id);

                    println!("Creating new stream. Streams: {:?}", self.sessions);

                    let stream = LrcpStream{
                        session_id,
                        src,
                        socket: clone,
                        last_seen: RefCell::new(0),
                        ack: RefCell::new(0),
                        open: RefCell::new(true),
                        received: RefCell::new("".to_string()),
                        sent: RefCell::new("".to_string()),
                    };

                    return Ok(stream)
                }
                Ok(LcrpMessage::Close{ ref session_id }) if self.sessions.contains(session_id) => {
                    self.sessions.remove(session_id);
                }
                Ok(LcrpMessage::Ack { ref session_id, .. }
                    | LcrpMessage::Close { ref session_id }
                    | LcrpMessage::Data { ref session_id, .. }) if !self.sessions.contains(session_id) => {
                    self.socket.recv_from(&mut buf)?;
                    let response = format!("/close/{}/", session_id);
                    self.socket.send_to(response.to_string().as_bytes(), src)?;
                }
                _ => {}
            }
        }
    }
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<LrcpStream>;
    fn next(&mut self) -> Option<io::Result<LrcpStream>> {
        Some(self.listener.accept())
    }
}

impl LrcpStream {
    fn poll(&self) -> io::Result<()> {
        thread::sleep(Duration::from_millis(100));

        let mut last_seen = self.last_seen.borrow_mut();

        if *last_seen >= 100 {
            return Err(Error::from(ErrorKind::ConnectionAborted));
        } else {
            *last_seen += 1;
        }
        println!("last_seen: {}", last_seen);

        let mut lrcp_buf = vec![0; 1000];
        let (amt, _) = self.socket.peek_from(&mut lrcp_buf)?;

        match parse_message(&lrcp_buf[..amt], self.session_id) {
            Ok(LcrpMessage::Data{session_id, data, pos}) if session_id == self.session_id => {
                self.socket.recv_from(&mut lrcp_buf)?;
                *last_seen = 0;

                let mut ack = self.ack.borrow_mut();

                if pos > *ack {
                    let response = format!("/ack/{}/{}/", session_id, ack);
                    self.socket.send_to(response.to_string().as_bytes(), self.src)?;
                }
                
                if pos == *ack {
                    self.received.borrow_mut().push_str(&str::from_utf8(&data).unwrap());
                    *ack += data.len() as i32;

                    let response = format!("/ack/{}/{}/", session_id, ack);
                    self.socket.send_to(response.to_string().as_bytes(), self.src)?;
                }

                Ok(())
            }
            Ok(LcrpMessage::Ack{session_id, len}) if session_id == self.session_id => {
                self.socket.recv_from(&mut lrcp_buf)?;
                *last_seen = 0;

                println!("Client {} acknowledges receipt up to {}.", self.session_id, len);

                if len > self.sent.borrow().len() as i32 {
                    return self.close();
                }

                if len < self.sent.borrow().len() as i32 {
                    println!("Client {} needs resend from position {}", self.session_id, len);
                    let response = format!(
                        "/data/{}/{}/{}/",
                        self.session_id,
                        len,
                        &self.sent.borrow()[(len as usize)..],
                    );
                    self.socket.send_to(response.to_string().as_bytes(), self.src)?;
                }

                Ok(())
            }
            Ok(LcrpMessage::Close{session_id}) if session_id == self.session_id => {
                self.socket.recv_from(&mut lrcp_buf)?;
                self.close()
            }
            _ => Ok(())
        }
    }

    fn close(&self) -> io::Result<()> {
        let response = format!("/close/{}/", self.session_id);
        self.socket.send_to(response.to_string().as_bytes(), self.src)?;

        *self.open.borrow_mut() = false;

        Err(Error::from(ErrorKind::ConnectionAborted))
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
                    _ => {
                        Err(Error::from(ErrorKind::WouldBlock))
                    }
                }
            }
            Err(_) => {
                Ok(0)
            }
        }
    }
}

impl Write for &LrcpStream {
    fn write(&mut self, write_buf: &[u8]) -> io::Result<usize> {
        let data = str::from_utf8(write_buf).unwrap();
        let response = format!("/data/{}/{}/{}/", self.session_id, self.sent.borrow().len(), data);
        println!("Attempting to send to client {}: {}", self.session_id, response);

        self.socket.send_to(response.to_string().as_bytes(), self.src)?;
        self.sent.borrow_mut().push_str(data);
        println!("The target ack for client {} is now {}", self.session_id, self.sent.borrow().len());

        return Ok(write_buf.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
