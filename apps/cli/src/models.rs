use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Heartbeat {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub project_name: String,
    pub project_path: String,
    pub entity_name: String,
    pub entity_type: String,
    pub branch_name: String,
    pub language_name: String,
    pub app_name: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: i32,
    pub activity_type: String,
    pub app_name: String,
    pub entity_name: String,
    pub entity_type: String,
    pub project_name: String,
    pub project_path: String,
    pub branch_name: String,
    pub language_name: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end_timestamp: Option<DateTime<Utc>>,
}
