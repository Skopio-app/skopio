use crate::heartbeat::get_or_create_project;
use chrono::Utc;
use rusqlite::{params, Connection};

pub fn log_event(
    conn: &Connection,
    project: String,
    full_path: String,
    activity_type: String,
    app: String,
    duration: i32,
) {
    let timestamp = Utc::now().to_rfc3339();
    let project_id = get_or_create_project(conn, &project, &full_path);

    conn.execute(
        "INSERT INTO events (timestamp, activity_type, app, duration, project_id, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        params![timestamp, activity_type, app, duration, project_id],
    )
    .expect("Failed to insert event");

    println!(
        "Event '{}' logged for {} ({} sec)",
        activity_type, app, duration
    );
}
