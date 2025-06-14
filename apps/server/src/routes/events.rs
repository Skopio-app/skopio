use crate::models::DurationRequest;
use crate::utils::error_response;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use common::models::inputs::EventInput;
use db::server::apps::App;
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::events::{fetch_recent, Event};
use db::server::languages::Language;
use db::server::projects::Project;
use db::DBContext;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

async fn handle_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<EventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    info!("Handling {} events", payload.len());

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

        let event = Event {
            id: None,
            timestamp: event.timestamp.unwrap_or_default(),
            duration: event.duration,
            activity_type: event.activity_type,
            app_id,
            entity_id: Some(entity_id),
            project_id: Some(project_id),
            branch_id: Some(branch_id),
            language_id: Some(language_id),
            end_timestamp: event.end_timestamp,
        };

        event.create(&db).await.map_err(error_response)?;
    }

    info!("Event details stored successfully");
    Ok(Json("Events recorded".to_string()))
}

pub async fn event_ws_handler(
    ws: WebSocketUpgrade,
    State(db): State<Arc<Mutex<DBContext>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_ws(socket, db))
}

async fn handle_event_ws(mut socket: WebSocket, db: Arc<Mutex<DBContext>>) {
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
                    Ok(events) if !events.is_empty() => {
                        last_check = events.last().unwrap().timestamp;
                        let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".into());
                        if let Err(e) = socket.send(Message::Text(json.into())).await {
                        error!("Error sending message: {}", e);
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

pub fn event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/events", post(handle_events))
        .route("/ws/events", get(event_ws_handler))
        .with_state(db)
}
