use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use types::Project;

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedProjects {
    pub data: Vec<Project>,
    pub total_pages: Option<u32>,
    pub cursors: Vec<Option<i64>>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InsightResult {
    ActiveYears(Vec<i32>),
    TopN(Vec<(String, i64)>),
    MostActiveDay { date: String, total_duration: i64 },
    AggregatedAverage(BTreeMap<String, Vec<(String, f64)>>),
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct FullEvent {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: String,
    pub app: Option<String>,
    pub entity: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct EventGroup {
    pub group: String,
    pub events: Vec<FullEvent>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub enum EventGroupResult {
    Flat(Vec<FullEvent>),
    Grouped(Vec<EventGroup>),
}
