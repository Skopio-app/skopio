use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Group, InsightBucket, InsightType},
    time::TimeRangePreset,
};

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
    pub source_name: String,
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
    pub source_name: String,
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

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
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
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BucketedSummaryInput {
    pub preset: TimeRangePreset,
    #[specta(optional)]
    pub apps: Option<Vec<String>>,
    #[specta(optional)]
    pub projects: Option<Vec<String>>,
    #[specta(optional)]
    pub entities: Option<Vec<String>>,
    #[specta(optional)]
    pub categories: Option<Vec<String>>,
    #[specta(optional)]
    pub branches: Option<Vec<String>>,
    #[specta(optional)]
    pub languages: Option<Vec<String>>,
    #[specta(optional)]
    pub group_by: Option<Group>,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct PaginationQuery {
    pub after: Option<Uuid>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct ProjectQuery {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct ProjectSearchQuery {
    pub name: String,
    pub limit: u32,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InsightQueryPayload {
    pub insight_type: InsightType,
    #[specta(optional)]
    pub insight_range: Option<String>,
    #[specta(optional)]
    pub group_by: Option<Group>,
    #[specta(optional)]
    pub limit: Option<usize>,
    #[specta(optional)]
    pub bucket: Option<InsightBucket>,
}
