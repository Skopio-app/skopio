use crate::utils::extract_project_name;
use chrono::{TimeZone, Utc};
use common::models::inputs::{EventInput, HeartbeatInput};
use log::{debug, info};
use reqwest::blocking::Client;
use rusqlite::{Connection, Row};
use thiserror::Error;

const SERVER_URL: &str = "http://localhost:8080";

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}

pub fn sync_data(conn: &Connection) -> Result<(), SyncError> {
    let client = Client::new();
    let heartbeats = fetch_unsynced_heartbeats(conn)?;
    let events = fetch_unsynced_events(conn)?;

    if heartbeats.is_empty() && events.is_empty() {
        debug!("No data to sync");
        return Ok(());
    }

    if !heartbeats.is_empty() {
        sync_to_server(&client, "heartbeats", &heartbeats)?;
        conn.execute("UPDATE heartbeats SET synced = 1 WHERE synced = 0", [])?;
        info!("Heartbeats synced successfully!");
    }

    if !events.is_empty() {
        sync_to_server(&client, "events", &events)?;
        conn.execute("UPDATE events SET synced = 1 WHERE synced = 0", [])?;
        info!("Events synced successfully!")
    }

    Ok(())
}

fn fetch_unsynced_heartbeats(conn: &Connection) -> Result<Vec<HeartbeatInput>, rusqlite::Error> {
    let mut stmt = conn
        .prepare(
            "SELECT timestamp, project_path, branch, entity_name, entity_type, language, app, is_write, lines, cursorpos
                  FROM heartbeats WHERE synced = 0",
                )?;
    let rows = stmt.query_map([], |row| parse_heartbeat(row))?;
    Ok(rows.flatten().collect())
}

fn parse_heartbeat(row: &Row) -> rusqlite::Result<HeartbeatInput> {
    let project_path: Option<String> = row.get(1)?;

    let timestamp: i64 = row.get(0)?;

    Ok(HeartbeatInput {
        timestamp: Some(Utc.timestamp_opt(timestamp, 0).single().unwrap_or_default()),
        project_name: extract_project_name(project_path.clone().unwrap_or_default()),
        project_path: project_path.unwrap_or_default(),
        branch_name: row.get(2)?,
        entity_name: row.get(3)?,
        entity_type: row.get(4)?,
        language_name: row.get(5)?,
        app_name: row.get(6)?,
        is_write: row.get(7)?,
        lines: row.get(8)?,
        cursorpos: row.get(9)?,
    })
}

fn fetch_unsynced_events(conn: &Connection) -> Result<Vec<EventInput>, rusqlite::Error> {
    let mut stmt = conn
        .prepare(
            "SELECT timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, end_timestamp
                 FROM events WHERE synced = 0"
                )?;
    let rows = stmt.query_map([], |row| parse_event(row))?;
    Ok(rows.flatten().collect())
}

fn parse_event(row: &Row) -> rusqlite::Result<EventInput> {
    let ts: Option<i64> = row.get(0)?;
    let end_ts: Option<i64> = row.get(9)?;
    let project_path: String = row.get(6)?;

    Ok(EventInput {
        timestamp: ts.map(|t| Utc.timestamp_opt(t, 0).single().unwrap_or_default()),
        category: row.get(1)?,
        app_name: row.get(2)?,
        entity_name: row.get(3)?,
        entity_type: row.get(4)?,
        duration: row.get(5)?,
        project_name: extract_project_name(&project_path),
        project_path,
        branch_name: row.get(7)?,
        language_name: row.get(8)?,
        end_timestamp: end_ts.map(|t| Utc.timestamp_opt(t, 0).single().unwrap_or_default()),
    })
}

fn sync_to_server<T: serde::Serialize>(
    client: &Client,
    path: &str,
    data: &T,
) -> Result<(), SyncError> {
    let res = client
        .post(format!("{}/{}", SERVER_URL, path))
        .json(data)
        .send()?;

    if res.status().is_success() {
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        Err(SyncError::UnexpectedResponse(format!(
            "Status: {}, Body: {}",
            status, body
        )))
    }
}
