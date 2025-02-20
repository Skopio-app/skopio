use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;

/// Updates or inserts the project into the database if it doesn't exist, and returns the project ID.
pub fn get_or_create_project(conn: &Connection, project_name: &str, full_path: &str) -> i32 {
    let mut stmt = conn
        .prepare("SELECT id, full_path FROM projects WHERE name = ?1")
        .unwrap();
    let mut rows = stmt.query(params![project_name]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let id: i32 = row.get(0).unwrap();
        let existing_path: String = row.get(1).unwrap();

        // If path has changed, update it
        if existing_path != full_path {
            conn.execute(
                "UPDATE projects SET full_path = ?1 WHERE id = ?2",
                params![full_path, id],
            )
            .expect("Failed to update project path");
            println!("Project '{}' path updated to '{}'", project_name, full_path);
        }

        return id;
    }

    // If no existing project, insert new
    conn.execute(
        "INSERT INTO projects (name, full_path) VALUES (?1, ?2)",
        params![project_name, full_path],
    )
    .expect("Failed to insert project");

    conn.last_insert_rowid() as i32
}

/// Converts an absolute file path into a relative path based on the project root.
fn get_relative_path(full_path: &str, file_path: &str) -> String {
    let project_root = Path::new(full_path);
    let file = Path::new(file_path);

    file.strip_prefix(project_root)
        .unwrap_or(file)
        .to_string_lossy()
        .into_owned()
}

pub fn log_heartbeat(
    conn: &Connection,
    project: String,
    full_path: String,
    branch: Option<String>,
    file: String,
    language: String,
    app: String,
    is_write: bool,
) {
    let timestamp = Utc::now().to_rfc3339();
    let project_id = get_or_create_project(conn, &project, &full_path);
    let relative_file = get_relative_path(&full_path, &file);

    conn.execute(
        "INSERT INTO heartbeats (timestamp, project_id, branch, file, language, app, is_write, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
        params![timestamp, project_id, branch, relative_file, language, app, is_write]
    )
        .expect("Failed to insert heartbeat");

    println!("Heartbeat logged for {}", file);
}
