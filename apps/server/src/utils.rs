use axum::http::StatusCode;
use axum::Json;
use dirs::data_dir;
use std::path::PathBuf;
use tracing::error;

fn get_db_name() -> String {
    if cfg!(debug_assertions) {
        return String::from("skopio_server_test.db");
    } else {
        return String::from("skopio_server.db");
    }
}

pub fn get_db_path() -> PathBuf {
    let base_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    base_dir
        .join("com.samwahome.skopio")
        .join("server")
        .join(get_db_name())
}

pub fn error_response<E: std::fmt::Display>(err: E) -> (StatusCode, Json<String>) {
    error!("Error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(format!("Error: {}", err)),
    )
}
