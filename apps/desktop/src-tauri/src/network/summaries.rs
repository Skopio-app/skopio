use std::{sync::LazyLock, time::Duration};

use common::models::inputs::{BucketedSummaryInput, SummaryQueryInput};
use db::models::{BucketTimeSummary, GroupedTimeSummary};
use reqwest::Client;
use serde::{Deserialize, Serialize};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to create request client")
});

const SERVER_URL: &str = "http://localhost:8080";

async fn post_json<TReq, TRes>(path: &str, body: &TReq) -> Result<TRes, String>
where
    TReq: Serialize + ?Sized,
    TRes: for<'de> Deserialize<'de>,
{
    let res = HTTP_CLIENT
        .post(format!("{}/{}", SERVER_URL, path))
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        res.json::<TRes>().await.map_err(|e| e.to_string())
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string()))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_total_time(query: SummaryQueryInput) -> Result<i64, String> {
    post_json("summary/total-time", &query).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_range_summary(
    query: SummaryQueryInput,
) -> Result<Vec<GroupedTimeSummary>, String> {
    post_json("summary/range", &query).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_bucketed_summary(
    query: BucketedSummaryInput,
) -> Result<Vec<BucketTimeSummary>, String> {
    post_json("summary/buckets", &query).await
}
