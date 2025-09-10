use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use db::{models::Category, DBContext};
use tokio::sync::Mutex;

use crate::error::AppResult;

async fn fetch_categories(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> AppResult<Json<Vec<Category>>> {
    let db = db.lock().await;

    let categories = Category::get_all(&db).await?;
    Ok(Json(categories))
}

pub fn category_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/categories", get(fetch_categories))
        .with_state(db)
}
