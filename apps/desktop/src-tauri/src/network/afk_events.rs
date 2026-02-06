use common::models::{inputs::BucketSummaryInput, outputs::FullEvent};

use crate::network::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_afk_events(query: BucketSummaryInput) -> Result<Vec<FullEvent>, String> {
    req_json("afk", Some(&query)).await
}
