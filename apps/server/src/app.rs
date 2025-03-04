use crate::routes::events::event_routes;
use crate::routes::health::health_routes;
use crate::routes::heartbeats::heartbeat_routes;
use axum::Router;
use db::DBContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

pub async fn create_app(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .merge(heartbeat_routes(db.clone()))
        .merge(event_routes(db.clone()))
        .merge(health_routes())
        .layer(CorsLayer::permissive())
}
