use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use db::{server::categories::Category, DBContext};
use tokio::sync::Mutex;

use crate::utils::error_response;

async fn fetch_categories(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<Category>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    match Category::get_all(&db).await {
        Ok(categories) => Ok(Json(categories)),
        Err(err) => Err(error_response(err)),
    }
}

pub fn category_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/categories", get(fetch_categories))
        .with_state(db)
}
