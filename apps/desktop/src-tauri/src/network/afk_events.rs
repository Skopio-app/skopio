use common::models::{inputs::BucketSummaryInput, outputs::FullEvent};
use db::models::BucketTimeSummary;

use crate::network::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_afk_events(query: BucketSummaryInput) -> Result<Vec<FullEvent>, String> {
    req_json("afk", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_bucketed_afk_summary(
    query: BucketSummaryInput,
) -> Result<Vec<BucketTimeSummary>, String> {
    req_json("afk/buckets", Some(&query)).await
}
