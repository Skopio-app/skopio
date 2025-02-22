use crate::utils::{extract_project_name, find_git_branch, find_git_project_root};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;

pub fn log_heartbeat(
    conn: &Connection,
    project: String,
    entity: String,
    language: String,
    app: String,
    is_write: bool,
    lines: Option<i64>,
    cursorpos: Option<i64>,
) {
    let file_path = Path::new(&entity);
    // let project_path = find_git_project_root(file_path);
    // let project_name = extract_project_name(&project_path);
    let branch_name = find_git_branch(file_path);

    let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO heartbeats (timestamp, project_path, branch, entity, language, app, is_write, lines, cursorpos, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0)",
        params![
            timestamp,
            project,
            branch_name,
            entity,
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
