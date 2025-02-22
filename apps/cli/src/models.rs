use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Heartbeat {
    pub timestamp: String,
    pub project: String,
    pub branch: String,
    pub file: String,
    pub language: String,
    pub app: String,
    pub is_write: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub timestamp: String,
    pub activity_type: String,
    pub app: String,
    pub duration: i32,
    pub project: String,
}