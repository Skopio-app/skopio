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
