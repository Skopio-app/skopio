use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::NaiveDateTime;
use db::{server::afk_events::AFKEvent, DBContext};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

use crate::utils::error_response;

#[derive(Serialize, Deserialize, Debug)]
struct AFKEventInput {
    afk_start: NaiveDateTime,
    afk_end: Option<NaiveDateTime>,
    duration: Option<i64>,
}

async fn handle_afk_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<AFKEventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    info!("Handling {} afk events", payload.len());

    for afk in payload {
        let afk_event = AFKEvent {
            id: None,
            afk_start: afk.afk_start,
            afk_end: afk.afk_end,
            duration: afk.duration,
        };

        afk_event.create(&db).await.map_err(error_response)?;
    }

    info!("AFK event details stored successfully");
    Ok(Json("AFK events recorded".to_owned()))
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .with_state(db)
}
