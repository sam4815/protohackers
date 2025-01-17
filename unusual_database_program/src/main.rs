use std::{collections::HashMap, io::Result, net::UdpSocket, str};

fn main() -> Result<()> {
    let mut store = HashMap::from([(
        "version".to_string(),
        "Sam's Key-Value Store 1.0".to_string(),
    )]);

    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    let mut buf = vec![0; 1000];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let buf = &buf[..amt];

        let request = str::from_utf8(buf).unwrap();

        match request.splitn(2, '=').collect::<Vec<&str>>()[..] {
            [k] => {
                let v = store.get(k).cloned().unwrap_or("".to_string());
                let response = format!("{}={}", k, v);

                socket.send_to(response.as_bytes(), src)?;
            }
            [k, v] => {
                if k == "version" {
                    continue;
                }

                store.insert(k.to_string(), v.to_string());
            }
            _ => {}
        };
    }
}
