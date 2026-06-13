use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use db::{DBContext, models::Category};

use crate::error::ServerResult;

async fn fetch_categories(State(db): State<Arc<DBContext>>) -> ServerResult<Json<Vec<Category>>> {
    let categories = Category::get_all(&db).await?;
    Ok(Json(categories))
}

pub fn category_routes(db: Arc<DBContext>) -> Router {
    Router::new()
        .route("/categories", get(fetch_categories))
        .with_state(db)
}
