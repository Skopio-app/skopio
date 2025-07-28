use std::collections::BTreeMap;

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
