use std::sync::Arc;
use axum::{Router, routing::post};
use axum::routing::get;
use db::DBContext;
use crate::handlers;


/// Create HTTP routes for the server
pub fn create_routes(db_context: DBContext) -> Router {
    let db_context = Arc::new(db_context);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/heartbeat", post(move |body| handlers::handle_heartbeat(body, db_context.clone())))
}
