use crate::models::{Event, Heartbeat};
use crate::utils::{extract_project_name, parse_timestamp};
use reqwest::blocking::Client;
use rusqlite::Connection;
use std::path::Path;

const SERVER_URL: &str = "http://localhost:8080";

pub fn sync_data(conn: &Connection) {
    let client = Client::new();

    let heartbeats: Vec<Heartbeat> = conn
        .prepare("SELECT timestamp, project_path, branch, entity, language, app, is_write, lines, cursorpos FROM heartbeats WHERE synced = 0")
        .unwrap()
        .query_map([], |row| {
            let project_path: Option<String> = row.get(1)?;

            Ok(Heartbeat {
                timestamp: row.get(0)?,
                project_name: extract_project_name(Path::new(&project_path.unwrap_or_else(|| "unknown".to_string()))),
                project_path: row.get(1)?,
                entity_name: row.get(3)?,
                branch_name: row.get(2)?,
                language_name: row.get(4)?,
                app_name: row.get(5)?,
                is_write: row.get(6)?,
                lines: row.get(7)?,
                cursorpos: row.get(8)?
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let events: Vec<Event> = conn
        .prepare("SELECT timestamp, activity_type, app, entity_name, entity_type, duration, project_path, branch, language, end_timestamp FROM events where synced = 0")
        .unwrap()
        .query_map([], |row| {
            let timestamp_str: Option<String> = row.get(0)?;
            let end_timestamp_str: Option<String> = row.get(9)?;
            let project_path: String = row.get(6).unwrap_or_else(|_| "UnknownPath".to_string());

            Ok(Event {
                timestamp: parse_timestamp(timestamp_str),
                activity_type: row.get(1)?,
                app_name: row.get(2)?,
                entity_name: row.get(3)?,
                entity_type: row.get(4)?,
                duration: row.get(5)?,
                project_name: extract_project_name(Path::new(&project_path)),
                project_path: row.get(6)?,
                branch_name: row.get(7)?,
                language_name: row.get(8)?,
                end_timestamp: parse_timestamp(end_timestamp_str),
            })
        })
        .unwrap()
        .flatten()
        .collect();

    if heartbeats.is_empty() && events.is_empty() {
        println!("No data to sync");
        return;
    }

    // Send heartbeats
    if !heartbeats.is_empty() {
        match client
            .post(format!("{}/heartbeats", SERVER_URL))
            .json(&heartbeats)
            .send()
        {
            Ok(response) => {
                if response.status().is_success() {
                    conn.execute("UPDATE heartbeats SET synced =  1 WHERE synced = 0", [])
                        .unwrap();
                    println!("Heartbeats synced successfully!")
                }
            }
            Err(err) => println!("Failed to sync heartbeats: {}", err),
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
                println!("Events synced successfully!");
            }
            Err(err) => println!("Failed to sync events: {}", err),
            _ => println!("Sync failed."),
        }
    }
}
