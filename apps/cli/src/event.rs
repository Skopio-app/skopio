use common::git::find_git_branch;
use log::info;
use rusqlite::{params, Connection};

use crate::error::CliError;

pub struct EventData {
    pub timestamp: i32,
    pub category: String,
    pub app: String,
    pub entity: String,
    pub entity_type: String,
    pub duration: i32,
    pub project: String,
    pub language: Option<String>,
    pub source: String,
    pub end_timestamp: i32,
}

pub fn save_event(conn: &Connection, event_data: EventData) -> Result<(), CliError> {
    let branch = find_git_branch(&event_data.project);

    conn.execute(
        "INSERT INTO events (timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, source, end_timestamp, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0)",
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
            event_data.source,
            event_data.end_timestamp,
        ],
    )?;

    info!(
        "Event '{}' saved for {} ({} sec)",
        event_data.category, event_data.app, event_data.duration
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::setup_test_conn;

    #[test]
    fn test_save_event_inserts_into_db() {
        let conn = setup_test_conn();

        let test_event = EventData {
            timestamp: 1720,
            category: "Coding".into(),
            app: "Code".into(),
            entity: "main.rs".into(),
            entity_type: "File".into(),
            duration: 300,
            project: "/tmp/my-project".into(),
            language: Some("Rust".into()),
            source: "skopio-vsode".into(),
            end_timestamp: 2020,
        };

        save_event(&conn, test_event).unwrap();

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM events").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }
}
