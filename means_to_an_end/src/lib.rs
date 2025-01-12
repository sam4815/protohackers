use std::collections::HashMap;
use std::io::{prelude::*, BufReader, ErrorKind, Result};

pub struct MessageIterator<R> {
    reader: BufReader<R>,
}

pub struct Message {
    pub message_type: char,
    pub a: i32,
    pub b: i32,
}

impl<R: Read> Iterator for MessageIterator<R> {
    type Item = Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 9];

        match self.reader.read_exact(&mut buffer) {
            Ok(()) => {
                let message = Message {
                    message_type: buffer[0] as char,
                    a: i32::from_be_bytes(buffer[1..5].try_into().unwrap()),
                    b: i32::from_be_bytes(buffer[5..].try_into().unwrap()),
                };

                Some(Ok(message))
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub fn consume_messages<R>(reader: BufReader<R>) -> MessageIterator<R> {
    MessageIterator { reader }
}

pub fn find_mean_price(min: i32, max: i32, prices: &HashMap<i32, i32>) -> i32 {
    let (sum, count) = prices
        .iter()
        .fold((0i64, 0), |(acc_sum, acc_count), (timestamp, price)| {
            if *timestamp >= min && *timestamp <= max {
                (acc_sum + i64::from(*price), acc_count + 1)
            } else {
                (acc_sum, acc_count)
            }
        });

    if count == 0 {
        return 0;
    }

    (sum / count).try_into().unwrap_or_default()
}
