use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct BucketTimeSummary {
    pub bucket: String,
    pub grouped_values: HashMap<String, i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RawBucketRow {
    pub bucket: String,
    pub group_key: String,
    pub total_seconds: i64,
}
