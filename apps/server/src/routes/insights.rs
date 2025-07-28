use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use common::models::{inputs::InsightQueryPayload, outputs::InsightResult};
use db::{
    server::insights::{InsightProvider, InsightQuery, Insights},
    DBContext,
};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub async fn fetch_insight(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<InsightQueryPayload>,
) -> Result<Json<InsightResult>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let insight_range = match payload.range {
        Some(ref r) => Some(r.clone().try_into().map_err(error_response)?),
        None => None,
    };

    let query = InsightQuery {
        insight_type: payload.insight_type,
        insight_range,
    };

    let result = Insights::execute(&db, query)
        .await
        .map_err(error_response)?;

    Ok(Json(result))
}

pub fn insights_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/insights", get(fetch_insight))
        .with_state(db)
}
