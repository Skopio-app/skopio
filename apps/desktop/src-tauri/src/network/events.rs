use common::models::{inputs::BucketSummaryInput, outputs::EventGroupResult};

use crate::network::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_events(query: BucketSummaryInput) -> Result<EventGroupResult, String> {
    req_json("events", Some(&query)).await
}
