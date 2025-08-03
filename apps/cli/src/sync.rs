use crate::utils::extract_project_name;
use chrono::{DateTime, TimeZone, Utc};
use common::models::inputs::{EventInput, HeartbeatInput};
use log::{debug, info};
use reqwest::blocking::Client;
use rusqlite::Connection;

const SERVER_URL: &str = "http://localhost:8080";

pub fn sync_data(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let heartbeats: Vec<HeartbeatInput> = conn
        .prepare("SELECT timestamp, project_path, branch, entity_name, entity_type, language, app, is_write, lines, cursorpos FROM heartbeats WHERE synced = 0")?
        .query_map([], |row| {
            let project_path: Option<String> = row.get(1)?;

            let timestamp_unix: i64 = row.get(0)?;
            let timestamp_utc: DateTime<Utc> = DateTime::from_timestamp(timestamp_unix, 0).unwrap_or_default().to_utc();

            let heartbeat = HeartbeatInput {
                timestamp: Some(timestamp_utc),
                project_name: extract_project_name(project_path.unwrap_or_default()),
                project_path: row.get(1)?,
                branch_name: row.get(2)?,
                entity_name: row.get(3)?,
                entity_type: row.get(4)?,
                language_name: row.get(5)?,
                app_name: row.get(6)?,
                is_write: row.get(7)?,
                lines: row.get(8)?,
                cursorpos: row.get(9)?
            };

            Ok(heartbeat)
        })?
        .flatten()
        .collect();

    let events: Vec<EventInput> = conn
        .prepare("SELECT timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, end_timestamp FROM events WHERE synced = 0")?
        .query_map([], |row| {
            let timestamp_unix: Option<i64> = row.get(0)?;
            let end_timestamp_unix: Option<i64> = row.get(9)?;

            let timestamp_utc: Option<DateTime<Utc>> = timestamp_unix.map(|ts| Utc.timestamp_opt(ts, 0).unwrap().to_utc());
            let end_timestamp_utc: Option<DateTime<Utc>> = end_timestamp_unix.map(|ts| Utc.timestamp_opt(ts, 0).unwrap().to_utc());

            let project_path: String = row.get(6).unwrap_or_default();

            let event = EventInput {
                timestamp: timestamp_utc,
                category: row.get(1)?,
                app_name: row.get(2)?,
                entity_name: row.get(3)?,
                entity_type: row.get(4)?,
                duration: row.get(5)?,
                project_name: extract_project_name(&project_path),
                project_path: row.get(6)?,
                branch_name: row.get(7)?,
                language_name: row.get(8)?,
                end_timestamp: end_timestamp_utc,
            };

            Ok(event)
        })?
        .flatten()
        .collect();

    if heartbeats.is_empty() && events.is_empty() {
        debug!("No data to sync");
        return Ok(());
    }

    // Send heartbeats
    if !heartbeats.is_empty() {
        match client
            .post(format!("{}/heartbeats", SERVER_URL))
            .json(&heartbeats)
            .send()
        {
            Ok(response) if response.status().is_success() => {
                conn.execute("UPDATE heartbeats SET synced = 1 WHERE synced = 0", [])
                    .unwrap();
                info!("Heartbeats synced successfully!")
            }

            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .unwrap_or_else(|_| "No response body".to_string());
                let err_msg = format!(
                    "Failed to sync heartbeats: {} - Response body: {}",
                    status, body
                );
                return Err(err_msg.into());
            }

            Err(err) => {
                let err_msg = format!("Error syncing heartbeats: {}", err);
                return Err(err_msg.into());
            }
        }
    }

    // Send events
    if !events.is_empty() {
        match client
            .post(format!("{}/events", SERVER_URL))
            .json(&events)
            .send()
        {
            Ok(response) if response.status().is_success() => {
                conn.execute("UPDATE events SET synced = 1 WHERE synced = 0", [])
                    .unwrap();
                info!("Events synced successfully!");
            }

            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .unwrap_or_else(|_| "No response body".to_string());
                let err_msg = format!(
                    "Failed to sync events: {} - Response body: {}",
                    status, body
                );
                return Err(err_msg.into());
            }

            Err(err) => {
                let err_msg = format!("Error syncing events: {}", err);
                return Err(err_msg.into());
            }
        }
    }

    Ok(())
}
