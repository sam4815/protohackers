use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{prelude::*, BufReader};

use serde_json::Value;

pub struct QueueManager {
    pub queues: HashMap<String, Vec<Job>>,
    pub allocated: HashSet<u32>,
    pub deleted: HashSet<u32>,
}

pub struct Job {
    pub id: u32,
    pub queue_id: String,
    pub pri: u32,
    pub job: Value,
}

impl QueueManager {
    pub fn new() -> QueueManager {
        QueueManager {
            queues: HashMap::new(),
            allocated: HashSet::new(),
            deleted: HashSet::new(),
        }
    }

    fn generate_random_u32(&mut self) -> u32 {
        let urandom = File::open("/dev/urandom").unwrap();
        let mut reader = BufReader::new(urandom);

        let mut buffer = [0; 4];
        let _ = reader.read_exact(&mut buffer);

        u32::from_be_bytes(buffer)
    }

    pub fn add_job(&mut self, queue_id: String, job: Value, pri: u32) -> u32 {
        let id = self.generate_random_u32();
        let job = Job {
            id,
            queue_id: queue_id.clone(),
            pri,
            job,
        };

        if let Some(queue) = self.queues.get_mut(&queue_id) {
            queue.push(job);
            queue.sort_by(|a, b| a.pri.cmp(&b.pri));
        } else {
            self.queues.insert(queue_id, vec![job]);
        }

        id
    }

    pub fn abort_job(&mut self, job: Job) -> String {
        if self.deleted.contains(&job.id) {
            return "no-job".to_string();
        }

        self.queues.get_mut(&job.queue_id).unwrap().push(job);

        return "ok".to_string();
    }

    pub fn delete_job(&mut self, job_id: u32) -> String {
        if self.allocated.contains(&job_id) {
            self.allocated.remove(&job_id);
            self.deleted.insert(job_id);

            return "ok".to_string();
        }

        for queue in self.queues.values_mut() {
            if let Some(position) = queue.iter().position(|job| job.id == job_id) {
                queue.remove(position);
                self.deleted.insert(job_id);

                return "ok".to_string();
            }
        }

        return "no-job".to_string();
    }

    pub fn pop_job(&mut self, queue_id: String) -> Option<Job> {
        Some(self.queues.get_mut(&queue_id).unwrap().remove(0))
    }

    pub fn get_highest_priority_job(&mut self, queue_ids: Vec<String>) -> Option<Job> {
        let highest_priority_job = queue_ids
            .iter()
            .filter_map(|id| self.queues.get(id))
            .filter_map(|queue| queue.first())
            .max_by_key(|job| job.pri);

        if let Some(job) = highest_priority_job {
            return self.pop_job(job.queue_id.clone());
        } else {
            return None;
        }
    }
}
