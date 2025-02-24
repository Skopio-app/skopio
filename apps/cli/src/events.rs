use crate::utils::find_git_branch;
use rusqlite::{params, Connection};
use std::path::Path;

pub fn log_event(
    conn: &Connection,
    timestamp: String,
    activity_type: String,
    app: String,
    entity: String,
    entity_type: String,
    duration: i32,
    project: String,
    language: String,
    end_timestamp: String,
) {
    let project_path = Path::new(&project);
    let branch = find_git_branch(&project_path);
    // let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO events (timestamp, activity_type, app, entity_name, entity_type, duration, project, branch, language, end_timestamp, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            timestamp,
            activity_type,
            app,
            entity,
            entity_type,
            duration,
            project,
            branch,
            language,
            end_timestamp,
        ],
    )
    .expect("Failed to insert event");

    println!(
        "Event '{}' logged for {} ({} sec)",
        activity_type, app, duration
    );
}
