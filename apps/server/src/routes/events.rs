use crate::models::{ClientMessage, EventOutput};
use crate::utils::error_response;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use common::models::inputs::EventInput;
use db::models::{App, Category};
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::events::{fetch_range, fetch_recent, Event};
use db::server::languages::Language;
use db::server::projects::Project;
use db::DBContext;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, warn};

async fn handle_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<EventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    debug!("Handling {} events", payload.len());

    for event in payload {
        let app_id = App::find_or_insert(&db, &event.app_name)
            .await
            .map_err(error_response)?;
        let project_id = Project::find_or_insert(&db, &event.project_name, &event.project_path)
            .await
            .map_err(error_response)?;
        let branch_id = Branch::find_or_insert(&db, project_id, &event.branch_name)
            .await
            .map_err(error_response)?;
        let entity_id =
            Entity::find_or_insert(&db, project_id, &event.entity_name, &event.entity_type)
                .await
                .map_err(error_response)?;
        let language_id = Language::find_or_insert(&db, &event.language_name)
            .await
            .map_err(error_response)?;
        let category_id = Category::find_or_insert(&db, &event.category)
            .await
            .map_err(error_response)?;

        let event = Event {
            id: None,
            timestamp: event.timestamp.unwrap_or_default(),
            duration: event.duration,
            category_id,
            app_id,
            entity_id: Some(entity_id),
            project_id: Some(project_id),
            branch_id: Some(branch_id),
            language_id: Some(language_id),
            end_timestamp: event.end_timestamp,
        };

        event.create(&db).await.map_err(error_response)?;
    }
    Ok(Json("Events saved".to_string()))
}

pub async fn event_ws_handler(
    ws: WebSocketUpgrade,
    State(db): State<Arc<Mutex<DBContext>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_ws(socket, db))
}

async fn handle_event_ws(mut socket: WebSocket, db: Arc<Mutex<DBContext>>) {
    let duration = chrono::Duration::minutes(15);
    let mut current_start: DateTime<Utc> = Utc::now() - duration;
    let mut current_end: DateTime<Utc> = Utc::now();
    let mut last_event_timestamp: DateTime<Utc> = current_start;

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

                                    if send_range_data(&mut socket, &db, current_start, current_end).await.is_err() {
                                        break;
                                    }
                                }

                                ClientMessage::Range(req) => {
                                    let parse_utc_time = |s: &str| {
                                        DateTime::parse_from_rfc3339(s)
                                            .map_err(|e| format!("RFC3339 parse error: {}", e))
                                            .map(|dt_utc| dt_utc.to_utc())
                                    };

                                    if let (Ok(start_ts), Ok(end_ts)) = (parse_utc_time(&req.start_timestamp), parse_utc_time(&req.end_timestamp)) {
                                        debug!("Received range request: {} to {}", start_ts, end_ts);
                                        current_start = start_ts;
                                        current_end = end_ts;

                                        if send_range_data(&mut socket, &db, current_start, current_end).await.is_err() {
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
                    Ok(events) if !events.is_empty() => {
                        last_event_timestamp = events.last().map(|e| e.timestamp).unwrap_or(last_event_timestamp);
                        let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".into());
                        if let Err(e) = socket.send(Message::Text(json.into())).await {
                        error!("Error sending new events: {}", e);
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    error!("Event stream fetch failed: {}", e);
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
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), axum::Error> {
    let db_guard = db.lock().await;
    match fetch_range(&db_guard, start, end).await {
        Ok(events) => {
            let serialized = serde_json::to_string(
                &events
                    .into_iter()
                    .map(EventOutput::from)
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            // let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".into());
            socket.send(Message::Text(serialized.into())).await?;
            Ok(())
        }
        Err(e) => {
            error!("Error fetching events for range: {}", e);
            Err(axum::Error::new(e))
        }
    }
}

pub fn event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/events", post(handle_events))
        .route("/ws/events", get(event_ws_handler))
        .with_state(db)
}
