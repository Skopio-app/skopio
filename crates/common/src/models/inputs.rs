use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EventInput {
    pub timestamp: Option<NaiveDateTime>,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app_name: String,
    pub entity_name: String,
    pub entity_type: String,
    pub project_name: String,
    pub project_path: String,
    pub branch_name: String,
    pub language_name: String,
    pub end_timestamp: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatInput {
    pub timestamp: Option<NaiveDateTime>,
    pub project_name: String,
    pub project_path: String,
    pub entity_name: String,
    pub entity_type: String,
    pub branch_name: String,
    pub language_name: Option<String>,
    pub app_name: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEventInput {
    pub afk_start: NaiveDateTime,
    pub afk_end: Option<NaiveDateTime>,
    pub duration: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SummaryQueryInput {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub app_names: Option<Vec<String>>,
    pub project_names: Option<Vec<String>>,
    pub activity_types: Option<Vec<String>>,
    pub entity_names: Option<Vec<String>>,
    pub branch_names: Option<Vec<String>>,
    pub language_names: Option<Vec<String>>,
    pub include_afk: bool,
}
