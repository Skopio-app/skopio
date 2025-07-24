use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::time::TimeRangePreset;

#[derive(Serialize, Deserialize, Debug)]
pub struct EventInput {
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: String,
    pub app_name: String,
    pub entity_name: String,
    pub entity_type: String,
    pub project_name: String,
    pub project_path: String,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub end_timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatInput {
    pub timestamp: Option<DateTime<Utc>>,
    pub project_name: String,
    pub project_path: String,
    pub entity_name: String,
    pub entity_type: String,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub app_name: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEventInput {
    pub afk_start: DateTime<Utc>,
    pub afk_end: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Group {
    App,
    Project,
    Language,
    Branch,
    Category,
    Entity,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SummaryQueryInput {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    #[specta(optional)]
    pub apps: Option<Vec<String>>,
    #[specta(optional)]
    pub projects: Option<Vec<String>>,
    #[specta(optional)]
    pub categories: Option<Vec<String>>,
    #[specta(optional)]
    pub entities: Option<Vec<String>>,
    #[specta(optional)]
    pub branches: Option<Vec<String>>,
    #[specta(optional)]
    pub languages: Option<Vec<String>>,
    pub include_afk: bool,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct BucketedSummaryInput {
    pub preset: TimeRangePreset,
    #[specta(optional)]
    pub app_names: Option<Vec<String>>,
    #[specta(optional)]
    pub project_names: Option<Vec<String>>,
    #[specta(optional)]
    pub entity_names: Option<Vec<String>>,
    #[specta(optional)]
    pub category_names: Option<Vec<String>>,
    #[specta(optional)]
    pub branch_names: Option<Vec<String>>,
    #[specta(optional)]
    pub language_names: Option<Vec<String>>,
    #[specta(optional)]
    pub group_by: Option<Group>,
    pub include_afk: bool,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct PaginationQuery {
    pub after: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct ProjectQuery {
    pub id: i64,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct ProjectSearchQuery {
    pub name: String,
    pub limit: u32,
}
