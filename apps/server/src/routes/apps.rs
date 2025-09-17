use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use db::{models::App, DBContext};
use tokio::sync::Mutex;

use crate::error::ServerResult;

async fn fetch_apps(State(db): State<Arc<Mutex<DBContext>>>) -> ServerResult<Json<Vec<App>>> {
    let db = db.lock().await;

    let apps = App::get_all(&db).await?;
    Ok(Json(apps))
}

pub fn app_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new().route("/apps", get(fetch_apps)).with_state(db)
}
