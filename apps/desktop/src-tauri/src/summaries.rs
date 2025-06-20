use std::{sync::LazyLock, time::Duration};

use common::models::inputs::SummaryQueryInput;
use reqwest::Client;
use serde::{Deserialize, Serialize};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to create request client")
});

const SERVER_URL: &str = "http://localhost:8080";

// TODO: Find a way to reuse this struct
#[derive(Serialize, Deserialize, specta::Type)]
pub struct GroupedTimeSummary {
    pub group_key: String,
    pub total_seconds: i64,
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_app_summary(
    query: SummaryQueryInput,
) -> Result<Vec<GroupedTimeSummary>, String> {
    let res = HTTP_CLIENT
        .post(format!("{}/summary/apps", SERVER_URL))
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let data = res
            .json::<Vec<GroupedTimeSummary>>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string()))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_projects_summary(
    query: SummaryQueryInput,
) -> Result<Vec<GroupedTimeSummary>, String> {
    let res = HTTP_CLIENT
        .post(format!("{}/summary/projects", SERVER_URL))
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let data = res
            .json::<Vec<GroupedTimeSummary>>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string()))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_activity_types_summary(
    query: SummaryQueryInput,
) -> Result<Vec<GroupedTimeSummary>, String> {
    let res = HTTP_CLIENT
        .post(format!("{}/summary/activity-types", SERVER_URL))
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let data = res
            .json::<Vec<GroupedTimeSummary>>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string()))
    }
}
