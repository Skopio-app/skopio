use common::models::inputs::{BucketSummaryInput, SummaryQueryInput};
use db::models::BucketTimeSummary;

use crate::network::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_total_time(query: SummaryQueryInput) -> Result<i64, String> {
    req_json("summary/total", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_bucketed_summary(
    query: BucketSummaryInput,
) -> Result<Vec<BucketTimeSummary>, String> {
    req_json("summary/buckets", Some(&query)).await
}
