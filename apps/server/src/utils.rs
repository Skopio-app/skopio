use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, NaiveDateTime, Utc};
use dirs::data_dir;
use std::path::PathBuf;
use tracing::error;

pub fn get_application_support_path() -> PathBuf {
    let base_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    base_dir.join("Skopio").join("skopio.db")
}

pub fn error_response<E: std::fmt::Display>(err: E) -> (StatusCode, Json<String>) {
    error!("Error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(format!("Error: {}", err)),
    )
}

pub fn to_utc_string(naive: NaiveDateTime) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).to_rfc3339()
}
