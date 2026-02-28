use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use db::{models::App, DBContext};

use crate::error::ServerResult;

async fn fetch_apps(State(db): State<Arc<DBContext>>) -> ServerResult<Json<Vec<App>>> {
    let apps = App::get_all(&db).await?;
    Ok(Json(apps))
}

pub fn app_routes(db: Arc<DBContext>) -> Router {
    Router::new().route("/apps", get(fetch_apps)).with_state(db)
}
