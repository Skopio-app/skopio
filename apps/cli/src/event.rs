use common::git::find_git_branch;
use log::info;
use rusqlite::{params, Connection};

pub struct EventData {
    pub timestamp: i32,
    pub category: String,
    pub app: String,
    pub entity: String,
    pub entity_type: String,
    pub duration: i32,
    pub project: String,
    pub language: Option<String>,
    pub end_timestamp: i32,
}

pub fn log_event(
    conn: &Connection,
    event_data: EventData,
) -> Result<(), Box<dyn std::error::Error>> {
    let branch = find_git_branch(&event_data.project);

    conn.execute(
        "INSERT INTO events (timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, end_timestamp, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            event_data.timestamp,
            event_data.category,
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
        event_data.category, event_data.app, event_data.duration
    );

    Ok(())
}
