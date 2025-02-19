use crate::models::{Event, Heartbeat};
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde_json::json;

const SERVER_URL: &str = "";

pub fn sync_data(conn: &Connection) {
    let client = Client::new();

    let heartbeats: Vec<Heartbeat> = conn
        .prepare("SELECT id, timestamp, project, branch, file, language, app, is_write FROM heartbeats WHERE synced = 0")
        .unwrap()
        .query_map([], |row| {
            Ok(Heartbeat {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                project: row.get(2)?,
                branch: row.get(3)?,
                file: row.get(4)?,
                language: row.get(5)?,
                app: row.get(6)?,
                is_write: row.get(7)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let events: Vec<Event> = conn
        .prepare("SELECT id, timestamp, activity_type, app, duration FROM events where synced = 0")
        .unwrap()
        .query_map([], |row| {
            Ok(Event {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                activity_type: row.get(2)?,
                app: row.get(3)?,
                duration: row.get(4)?,
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

    match client.post(SERVER_URL).json(&payload).send() {
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
