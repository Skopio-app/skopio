use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use common::{
    models::{inputs::InsightQueryPayload, outputs::InsightResult},
    time::insight::InsightRange,
};
use db::{
    server::insights::{InsightProvider, InsightQuery, Insights},
    DBContext,
};
use tokio::sync::Mutex;

use crate::error::AppResult;

pub async fn fetch_insight(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(payload): Query<InsightQueryPayload>,
) -> AppResult<Json<InsightResult>> {
    let db = db.lock().await;

    let insight_range = match &payload.insight_range {
        Some(s) => Some(InsightRange::try_from(s.clone())?),
        None => None,
    };

    let query = InsightQuery {
        insight_type: payload.insight_type,
        insight_range,
        group_by: payload.group_by,
        limit: payload.limit,
        bucket: payload.bucket,
    };

    let result = Insights::execute(&db, query).await?;

    Ok(Json(result))
}

pub fn insights_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/insights", get(fetch_insight))
        .with_state(db)
}
