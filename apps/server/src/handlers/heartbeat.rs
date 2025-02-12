use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse, Json};
use db::{DBContext, events::Event};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use db::afk_events::AfkEvent;

#[derive(Deserialize)]
pub struct HeartbeatPayload {
    pub timestamp: String,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app_name: String,
    pub file_name: Option<String>,
    pub branch_name: Option<String>,
    pub language: Option<String>,
    pub status: Option<String>,
    pub afk: bool,
}

/// Handle incoming heartbeat requests and insert the event into the database.
pub async fn handle_heartbeat(
    Json(payload): Json<HeartbeatPayload>,
    db_context: Arc<DBContext>,
) -> impl IntoResponse {
    if payload.afk {
        // Process an afk event
        let afk_event = AfkEvent {
            id: None,
            afk_start: DateTime::from_naive_utc_and_offset(NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%d %H:%M:%S")
                .expect("Invalid timestamp format"),
            Utc),
            afk_end: None,
            duration: payload.duration,
        };

        match afk_event.insert(&db_context).await {
            Ok(_) => (StatusCode::OK, String::from("AFK event successfully processed")),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to process AFK event: {}", e))
        }
    } else {
        let event = Event {
            id: 1,
            timestamp: NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%d %H:%M:%S")
                .expect("Invalid timestamp format"),
            duration: payload.duration,
            activity_type: payload.activity_type,
            app_name: payload.app_name,
            file_name: payload.file_name,
            project_id: None,
            branch_name: payload.branch_name,
            language: payload.language,
            metadata: None,
            status: payload.status,
            end_timestamp: None,
        };

        // Insert the normal event into the database using `DbContext`
        match event.insert(&db_context).await {
            Ok(_) => (StatusCode::OK, String::from("Heartbeat successfully processed")),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to process heartbeat: {}", e))
        }
    }
}
