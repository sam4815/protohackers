use std::{collections::HashMap, i32, io::Result, net::UdpSocket, str};

fn main() -> Result<()> {
    let mut sessions: HashMap<i32, Vec<u8>> = HashMap::new();

    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    let mut buf = vec![0; 1000];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let buf = &buf[..amt];

        let escaped = str::from_utf8(buf).unwrap();
        let mut unescaped = str::replace(escaped, "//", "/");
        unescaped = str::replace(unescaped.as_str(), r"\\", r"\");

        println!("{}", unescaped);
        // println!("{:?}", unescaped.split('/'));

        match unescaped.split('/').collect::<Vec<&str>>()[..] {
            ["", "connect", session_str, ""] => {
                match session_str.parse::<i32>() {
                    Ok(session) if session >= 0 => {
                        if !sessions.contains_key(&session) {
                            sessions.insert(session, vec![0; 2048]);
                        }

                        let response = format!("/ack/{}/0/", session);
                        socket.send_to(response.to_string().as_bytes(), src)?;
                    }
                    _ => {}
                }
            }
            ["", "data", session_str, pos_str, data, ""] => {
                match (session_str.parse::<i32>(), pos_str.parse::<i32>()) {
                    (Ok(session), Ok(pos)) if sessions.contains_key(&session) && pos > 0 => {
                        let response = format!("/ack/{}/0/", session);
                        socket.send_to(response.to_string().as_bytes(), src)?;
                    }
                    (Ok(session), _) => {
                        let response = format!("/close/{}/", session);
                        socket.send_to(response.to_string().as_bytes(), src)?;
                    }
                    _ => {}
                }
            }
            ["", "close", session_str, ""] => {
                match session_str.parse::<i32>() {
                    Ok(session) => {
                        let response = format!("/close/{}/", session);
                        socket.send_to(response.to_string().as_bytes(), src)?;
                    }
                    _ => {}
                }
            }
            _ => {}
        };
    }
}
