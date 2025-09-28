use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

/// A single time bucket with grouped values.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BucketTimeSummary {
    /// The time bucket (e.g., "2025-08-01")
    pub bucket: String,
    /// A map of group_key: total_seconds
    pub grouped_values: HashMap<String, i64>,
    /// Optional per-group metadata (e.g. entity type when grouping by Entity)
    pub group_meta: Option<String>,
}
/// Represents an aggregated total time for a specific group.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupedTimeSummary {
    /// The group key (e.g., project name, app name)
    pub group_key: String,
    /// Total aggregated time (in seconds)
    pub total_seconds: i64,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct App {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
}
