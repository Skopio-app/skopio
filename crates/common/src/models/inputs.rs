use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    models::{Group, InsightBucket, InsightType},
    time::TimeRangePreset,
};

/// Input payload for inserting a new event.
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

/// Input payload for AFK (Away From Keyboard) events
#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEventInput {
    pub afk_start: DateTime<Utc>,
    pub afk_end: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

/// Query input for requesting summaries over a range of time
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

/// Query input for bucketed summaries (based on a preset time range)
#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BucketSummaryInput {
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
#[serde(rename_all = "camelCase")]
pub struct ProjectListQuery {
    pub after: Option<String>,
    pub limit: Option<u32>,
    pub query: Option<String>,
}

/// Query payload for insights
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
