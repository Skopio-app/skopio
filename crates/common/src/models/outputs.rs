use std::collections::BTreeMap;

use super::Project;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedProjects {
    pub data: Vec<Project>,
    pub total_pages: Option<u32>,
    pub cursors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InsightResult {
    ActiveYears(Vec<i32>),
    TopN(Vec<(String, i64)>),
    MostActiveDay { date: String, total_duration: i64 },
    AggregatedAverage(BTreeMap<String, Vec<(String, f64)>>),
}

/// A fully materialized event row
#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FullEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: String,
    pub app: Option<String>,
    pub entity: Option<String>,
    pub entity_type: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
    pub source: String,
}

/// A collection of events that share a common grouping key.
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct EventGroup {
    /// The group key value
    pub group: String,
    /// The list of `FullEvent` rows belonging to this group
    pub events: Vec<FullEvent>,
}

/// The result of fetching events for a time/window query
///
/// - `Flat(Vec<FullEvent>)` - returned when no `group_by` is set. Contains every
///   matching event row.
/// - `Grouped(Vec<EventGroup>)` - returned when a `group_by` dimension is set.
///   Each `EventGroup` holds a `group` key (e.g., a category name) and **all**
///   events that belong to that group.
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub enum EventGroupResult {
    /// Ungrouped list of events.
    Flat(Vec<FullEvent>),
    /// Events grouped by a group key (e.g., category, app, project, source, etc.)
    Grouped(Vec<EventGroup>),
}
