use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct BucketTimeSummary {
    pub bucket: String,
    pub grouped_values: HashMap<String, i64>,
    /// Optional per-group metadata (e.g. entity type when grouping by Entity)
    pub group_meta: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct GroupedTimeSummary {
    pub group_key: String,
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
