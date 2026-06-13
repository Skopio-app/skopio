use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use db::{DBContext, models::App};

use crate::error::ServerResult;

async fn fetch_apps(State(db): State<Arc<DBContext>>) -> ServerResult<Json<Vec<App>>> {
    let apps = App::get_all(&db).await?;
    Ok(Json(apps))
}

pub fn app_routes(db: Arc<DBContext>) -> Router {
    Router::new().route("/apps", get(fetch_apps)).with_state(db)
}
