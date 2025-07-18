use std::{sync::LazyLock, time::Duration};

use common::models::outputs::PaginatedProjects;
use db::models::{App, Category};
use reqwest::Client;
use serde::Deserialize;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create request client")
});

const SERVER_URL: &str = "http://localhost:8080";

async fn req_json<TRes>(path: &str) -> Result<TRes, String>
where
    // TReq: Serialize + ?Sized,
    TRes: for<'de> Deserialize<'de>,
{
    let res = HTTP_CLIENT
        .get(format!("{}/{}", SERVER_URL, path))
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
pub async fn fetch_apps() -> Result<Vec<App>, String> {
    req_json("apps").await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_categories() -> Result<Vec<Category>, String> {
    req_json("categories").await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_projects() -> Result<Vec<PaginatedProjects>, String> {
    req_json("projects").await
}
