use crate::utils::find_git_branch;
use rusqlite::{params, Connection};
use std::path::Path;

pub fn log_heartbeat(
    conn: &Connection,
    timestamp: i32,
    project: String,
    entity: String,
    entity_type: String,
    language: String,
    app: String,
    is_write: bool,
    lines: Option<i64>,
    cursorpos: Option<i64>,
) {
    let file_path = Path::new(&entity);
    let branch_name = find_git_branch(file_path);

    conn.execute(
        "INSERT INTO heartbeats (timestamp, project_path, branch, entity_name, entity_type, language, app, is_write, lines, cursorpos, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            timestamp,
            project,
            branch_name,
            entity,
            entity_type,
            language,
            app,
            is_write,
            lines,
            cursorpos,
        ],
    )
    .expect("Failed to insert heartbeat");

    println!("Heartbeat logged for {}", entity);
}
