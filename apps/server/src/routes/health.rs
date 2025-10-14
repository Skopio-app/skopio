use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use common::models::outputs::HealthStatus;

pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthStatus {
            status: "ok".into(),
        }),
    )
}

pub fn health_routes() -> Router {
    Router::new().route("/health", get(health_check))
}
