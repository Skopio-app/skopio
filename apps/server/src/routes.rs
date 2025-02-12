use axum::{Router, routing::post};
use db::DBContext;
use crate::handlers::heartbeat::handle_heartbeat;

/// Create HTTP routes for the server
pub fn create_routes(db_context: DBContext) -> Router {
    Router::new()
        .route("/heartbeat", post(move |body| handle_heartbeat(body, db_context.clone())))
}
