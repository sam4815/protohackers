use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Deserialize)]
pub struct Request {
    pub method: String,
    pub number: Number,
}

#[derive(Serialize)]
pub struct Response {
    pub method: String,
    pub prime: bool,
}
