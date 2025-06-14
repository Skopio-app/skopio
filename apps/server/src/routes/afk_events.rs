use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use common::models::inputs::AFKEventInput;
use db::{
    server::afk_events::{fetch_recent, AFKEvent},
    DBContext,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use tracing::{debug, error, info, warn};

use crate::{models::DurationRequest, utils::error_response};

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

pub async fn afk_ws_handler(
    ws: WebSocketUpgrade,
    State(db): State<Arc<Mutex<DBContext>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_afk_ws(socket, db))
}

async fn handle_afk_ws(mut socket: WebSocket, db: Arc<Mutex<DBContext>>) {
    let mut duration = chrono::Duration::minutes(15);
    let mut last_check = Utc::now().naive_utc() - duration;

    loop {
        tokio::select! {
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(parsed) = serde_json::from_str::<DurationRequest>(&text) {
                                duration = chrono::Duration::minutes(parsed.minutes);
                                last_check = Utc::now().naive_utc() - duration;
                                debug!("Updated duration to {} minutes", parsed.minutes);
                            } else {
                                warn!("Invalid duration payload: {}", text);
                            }
                        }
                        Some(Ok(_)) => {} // Ignore non-text messages
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => break, // client closed connection
                    }
                }

        _ = sleep(Duration::from_secs(5)) => {
            let db = db.lock().await;
            match fetch_recent(&db, last_check).await {
                Ok(afk_events) if !afk_events.is_empty() => {
                    last_check = afk_events.last().unwrap().afk_start;
                    let json = serde_json::to_string(&afk_events).unwrap_or_else(|_| "[]".into());
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {},
                Err(e) => {
                    error!("AFK event stream fetch failed: {}", e);
                    break;
                }
            }
                }
            }
    }
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .route("/ws/afk", get(afk_ws_handler))
        .with_state(db)
}
