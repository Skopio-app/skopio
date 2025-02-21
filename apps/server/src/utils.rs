use axum::http::StatusCode;
use axum::Json;
use dirs::data_dir;
use std::path::PathBuf;

pub fn get_application_support_path() -> PathBuf {
    let base_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    let db_path = base_dir.join("Timestack").join("timestack.db");

    db_path
}

pub fn error_response<E: std::fmt::Display>(err: E) -> (StatusCode, Json<String>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(format!("Error: {}", err)),
    )
}
