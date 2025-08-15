use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct BucketTimeSummary {
    pub bucket: String,
    pub grouped_values: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct GroupedTimeSummary {
    pub group_key: String,
    pub total_seconds: i64,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct App {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, specta::Type, sqlx::FromRow)]
pub struct Source {
    pub id: uuid::Uuid,
    pub name: String,
}
