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
use chrono::{DateTime, NaiveDateTime, Utc};
use common::models::inputs::AFKEventInput;
use db::{
    server::afk_events::{fetch_range, fetch_recent, AFKEvent},
    DBContext,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use tracing::{debug, error, info, warn};

use crate::{models::ClientMessage, utils::error_response};

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
    let duration = chrono::Duration::minutes(15);
    let mut current_start: NaiveDateTime = Utc::now().naive_utc() - duration;
    let mut current_end: NaiveDateTime = Utc::now().naive_utc();
    let mut last_event_timestamp: NaiveDateTime = current_start;

    loop {
        tokio::select! {
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                                match client_msg {
                                    ClientMessage::Duration(req) => {
                                        let duration = chrono::Duration::minutes(req.minutes);
                                        current_start = current_end - duration;
                                        last_event_timestamp = current_start;
                                        debug!("Updated duration to {} minutes", req.minutes);

                                        if let Err(e) = send_range_data(&mut socket, &db, current_start, current_end).await {
                                            error!("Error sending initial duration data: {}", e);
                                            break;
                                        }
                                    }
                                    ClientMessage::Range(req) => {
                                        let parse_date = |s: &str| {
                                            DateTime::parse_from_rfc3339(s)
                                                .map_err(|e| format!("RFC3339 parse error: {}", e))
                                                .map(|dt_utc| dt_utc.naive_utc())
                                        };

                                        if let (Ok(start_ts), Ok(end_ts)) = (parse_date(&req.start_timestamp), parse_date(&req.end_timestamp)) {
                                            debug!("Received range request: {} to {}", start_ts, end_ts);
                                            current_start = start_ts;
                                            current_end = end_ts;

                                            if let Err(e) = send_range_data(&mut socket, &db, current_start, current_end).await {
                                                error!("Error sending range data: {}", e);
                                                break;
                                            }
                                        } else {
                                            warn!("Invalid range timestamp: {} - {}", req.start_timestamp, req.end_timestamp);
                                        }
                                    }
                                }
                            } else {
                                warn!("Invalid client payload: {}", text);
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
            match fetch_recent(&db, last_event_timestamp).await {
                Ok(afk_events) if !afk_events.is_empty() => {
                    last_event_timestamp = afk_events.last().map(|e| e.afk_start).unwrap_or(last_event_timestamp);
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

async fn send_range_data(
    socket: &mut WebSocket,
    db: &Arc<Mutex<DBContext>>,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<(), axum::Error> {
    let db_guard = db.lock().await;
    match fetch_range(&db_guard, start, end).await {
        Ok(afk_events) => {
            let json = serde_json::to_string(&afk_events).unwrap_or_else(|_| "[]".into());
            socket.send(Message::Text(json.into())).await?;
            Ok(())
        }
        Err(e) => {
            error!("Error fetching afk events for range: {}", e);
            Err(axum::Error::new(e))
        }
    }
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .route("/ws/afk", get(afk_ws_handler))
        .with_state(db)
}
