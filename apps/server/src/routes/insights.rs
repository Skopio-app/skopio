use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use db::{
    server::insights::{InsightProvider, InsightQuery, InsightResult, InsightType, Insights},
    DBContext,
};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub async fn fetch_active_years(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<i32>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;
    let query = InsightQuery {
        insight_type: InsightType::ActiveYears,
        year: None,
        month: None,
        week: None,
        day: None,
    };

    match Insights::execute(&db, query).await {
        Ok(InsightResult::ActiveYears(years)) => Ok(Json(years)),
        _ => Err(error_response("Unexpected result type for ActiveYears")),
    }
}

pub fn insights_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/insights/years", get(fetch_active_years))
        .with_state(db)
}
