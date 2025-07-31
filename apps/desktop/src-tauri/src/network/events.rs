use common::models::{inputs::BucketedSummaryInput, outputs::EventGroupResult};

use crate::network::utils::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_events(query: BucketedSummaryInput) -> Result<EventGroupResult, String> {
    req_json("events", Some(&query)).await
}
