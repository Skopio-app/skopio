use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use db::{server::insights::get_active_years, DBContext};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub async fn fetch_active_years(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<i32>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;
    let total_years = get_active_years(&db).await.map_err(error_response)?;

    Ok(Json(total_years))
}

pub fn insights_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/insights/years", get(fetch_active_years))
        .with_state(db)
}
