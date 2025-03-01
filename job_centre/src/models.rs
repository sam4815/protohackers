use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "lowercase")]
pub enum Request {
    Get {
        queues: Vec<String>,
        #[serde(default)]
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
#[serde(untagged)]
pub enum Response {
    Get {
        status: String,
        id: u32,
        job: Value,
        pri: u32,
        queue: String,
    },
    Put {
        status: String,
        id: u32,
    },
    Delete {
        status: String,
    },
    Abort {
        status: String,
    },
    Error {
        status: String,
        error: String,
    },
}
