use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use common::models::inputs::AFKEventInput;
use db::{server::afk_events::AFKEvent, DBContext};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;

use crate::utils::error_response;

async fn handle_afk_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<AFKEventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    debug!("Handling {} afk events", payload.len());

    for afk in payload {
        let id = Uuid::now_v7();
        let afk_event = AFKEvent {
            id,
            afk_start: afk.afk_start,
            afk_end: afk.afk_end,
            duration: afk.duration,
        };

        afk_event.create(&db).await.map_err(error_response)?;
    }

    Ok(Json("AFK events saved".to_owned()))
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .with_state(db)
}
