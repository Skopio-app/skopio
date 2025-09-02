use crate::routes::afk_events::afk_event_routes;
use crate::routes::apps::app_routes;
use crate::routes::categories::category_routes;
use crate::routes::events::event_routes;
use crate::routes::health::health_routes;
use crate::routes::heartbeats::heartbeat_routes;
use crate::routes::insights::insights_routes;
use crate::routes::projects::project_routes;
use crate::routes::summary::summary_routes;
use axum::Router;
use db::DBContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

pub async fn create_app(db: Arc<Mutex<DBContext>>) -> Router {
    let router = Router::new()
        .merge(heartbeat_routes(db.clone()))
        .merge(event_routes(db.clone()))
        .merge(afk_event_routes(db.clone()))
        .merge(health_routes())
        .merge(summary_routes(db.clone()))
        .merge(app_routes(db.clone()))
        .merge(category_routes(db.clone()))
        .merge(project_routes(db.clone()))
        .merge(insights_routes(db.clone()));

    let dev_mode = cfg!(debug_assertions)
        || std::env::var("SKOPIO_DEV")
            .map(|v| v == "1")
            .unwrap_or(false);

    if dev_mode {
        router.layer(CorsLayer::permissive())
    } else {
        router
    }
}
