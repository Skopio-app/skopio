use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use db::{server::apps::App, DBContext};
use tokio::sync::Mutex;

use crate::utils::error_response;

async fn fetch_apps(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<App>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    match App::get_all(&db).await {
        Ok(apps) => Ok(Json(apps)),
        Err(err) => Err(error_response(err)),
    }
}

pub fn app_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new().route("/apps", get(fetch_apps)).with_state(db)
}
