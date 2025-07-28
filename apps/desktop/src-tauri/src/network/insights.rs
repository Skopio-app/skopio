use common::models::{inputs::InsightQueryPayload, outputs::InsightResult};

use crate::network::utils::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_insights(query: InsightQueryPayload) -> Result<InsightResult, String> {
    req_json("insights", Some(&query)).await
}
