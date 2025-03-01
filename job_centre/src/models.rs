use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "lowercase")]
pub enum Request {
    Get {
        queues: Vec<String>,
        wait: bool,
    },
    Put {
        queue: String,
        pri: u32,
        job: Value,
    },
    Delete {
        id: u32,
    },
    Abort {
        id: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "lowercase")]
pub enum Response {
    Get {
        status: String,
        id: u32,
    },
    Put {
        status: String,
    },
    Delete {
        status: String,
    },
    Abort {
        status: String,
    },
}
