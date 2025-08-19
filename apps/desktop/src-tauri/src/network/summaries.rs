use common::models::inputs::{BucketedSummaryInput, SummaryQueryInput};
use db::models::{BucketTimeSummary, GroupedTimeSummary};

use crate::network::utils::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_total_time(query: SummaryQueryInput) -> Result<i64, String> {
    req_json("summary/total-time", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_range_summary(
    query: SummaryQueryInput,
) -> Result<Vec<GroupedTimeSummary>, String> {
    req_json("summary/range", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_bucketed_summary(
    query: BucketedSummaryInput,
) -> Result<Vec<BucketTimeSummary>, String> {
    req_json("summary/buckets", Some(&query)).await
}
