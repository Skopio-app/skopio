use axum::{extract::State, routing::post, Json, Router};
use common::models::inputs::AFKEventInput;
use db::{server::afk_events::AFKEvent, DBContext};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::AppResult;

async fn handle_afk_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<AFKEventInput>>,
) -> AppResult<()> {
    debug!("Handling {} afk events", payload.len());

    let events: Vec<AFKEvent> = payload
        .into_iter()
        .map(|afk| AFKEvent {
            id: Uuid::now_v7(),
            afk_start: afk.afk_start,
            afk_end: afk.afk_end,
            duration: afk.duration,
        })
        .collect();

    if events.is_empty() {
        return Ok(());
    }

    let db = db.lock().await;
    let inserted = AFKEvent::bulk_create(&db, &events).await?;

    info!("Inserted {} AFK events", inserted);
    Ok(())
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .with_state(db)
}
