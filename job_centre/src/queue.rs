use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use serde_json::Value;

pub struct QueueManager {
    pub queues: HashMap<String, Vec<Job>>,
}

struct Job {
    id: u32,
    priority: u32,
    job: Value,
}

impl QueueManager {
    pub fn new(self) -> QueueManager {
        QueueManager { queues: HashMap::new() }
    }

    fn generate_random_u32(&mut self) -> u32 {
        let urandom = File::open("/dev/urandom").unwrap();
        let mut reader = BufReader::new(urandom);

        let mut buffer = [0; 4];
        let _ = reader.read_exact(&mut buffer);

        u32::from_be_bytes(buffer)
    }

    pub fn add_job(&mut self, queue_id: String, job: Value, priority: u32) -> u32 {
        let id = self.generate_random_u32();
        let job = Job { id, priority, job };

        if self.queues.contains_key(&queue_id) {
            self.queues.get_mut(&queue_id).unwrap().push(job);
        } else {
            self.queues.insert(queue_id, vec![job]);
        }

        id
    }

    pub fn get_job(&mut self, queue_ids: Vec<String>) {
        let mut candidates = Vec::<Job>::new();


    }
}
