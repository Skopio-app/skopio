use crate::models::{Event, Heartbeat};
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde_json::json;

const SERVER_URL: &str = "http://localhost:8080";

pub fn sync_data(conn: &Connection) {
    let client = Client::new();

    let heartbeats: Vec<Heartbeat> = conn
        .prepare("SELECT timestamp, project, branch, file, language, app, is_write FROM heartbeats WHERE synced = 0")
        .unwrap()
        .query_map([], |row| {
            Ok(Heartbeat {
                timestamp: row.get(0)?,
                project: row.get(1)?,
                branch: row.get(2)?,
                file: row.get(3)?,
                language: row.get(4)?,
                app: row.get(5)?,
                is_write: row.get(6)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let events: Vec<Event> = conn
        .prepare("SELECT timestamp, activity_type, app, duration, project FROM events where synced = 0")
        .unwrap()
        .query_map([], |row| {
            Ok(Event {
                timestamp: row.get(0)?,
                activity_type: row.get(1)?,
                app: row.get(2)?,
                duration: row.get(3)?,
                project: row.get(4)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    if heartbeats.is_empty() && events.is_empty() {
        println!("No data to sync");
        return;
    }

    let payload = json!({ "heartbeats": heartbeats, "events": events});

    match client.post(format!("{}/sync", SERVER_URL)).json(&payload).send() {
        Ok(response) if response.status().is_success() => {
            conn.execute("UPDATE heartbeats SET synced = 1 WHERE synced = 0", [])
                .unwrap();
            conn.execute("UPDATE events SET synced = 1 WHERE synced = 0", [])
                .unwrap();
            println!("Sync successful!");
        }

        _ => println!("Sync failed."),
    }
}
