use crate::utils::find_git_branch;
use log::info;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct EventData {
    pub timestamp: i32,
    pub activity_type: String,
    pub app: String,
    pub entity: String,
    pub entity_type: String,
    pub duration: i32,
    pub project: String,
    pub language: String,
    pub end_timestamp: i32,
}

pub fn log_event(
    conn: &Connection,
    event_data: EventData,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_path = Path::new(&event_data.project);
    let branch = find_git_branch(project_path);

    conn.execute(
        "INSERT INTO events (timestamp, activity_type, app, entity_name, entity_type, duration, project_path, branch, language, end_timestamp, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            event_data.timestamp,
            event_data.activity_type,
            event_data.app,
            event_data.entity,
            event_data.entity_type,
            event_data.duration,
            event_data.project,
            branch,
            event_data.language,
            event_data.end_timestamp,
        ],
    )
    .map_err(|e| format!( "Failed to insert event: {}", e))?;

    info!(
        "Event '{}' logged for {} ({} sec)",
        event_data.activity_type, event_data.app, event_data.duration
    );

    Ok(())
}
