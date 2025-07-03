use axum::http::StatusCode;
use axum::Json;
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
