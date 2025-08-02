use common::git::find_git_branch;
use log::info;
use rusqlite::{params, Connection};

pub struct HeartbeatData {
    pub timestamp: i32,
    pub project: String,
    pub entity: String,
    pub entity_type: String,
    pub app: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

pub fn log_heartbeat(
    conn: &Connection,
    hb_data: HeartbeatData,
) -> Result<(), Box<dyn std::error::Error>> {
    let branch_name = find_git_branch(&hb_data.entity);

    conn.execute(
        "INSERT INTO heartbeats (timestamp, project_path, branch, entity_name, entity_type, language, app, is_write, lines, cursorpos, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            hb_data.timestamp,
            hb_data.project,
            branch_name,
            hb_data.entity,
            hb_data.entity_type,
            hb_data.app,
            hb_data.is_write,
            hb_data.lines,
            hb_data.cursorpos,
        ],
    )
    .map_err(|e| format!("Failed to insert heartbeat: {}", e))?;

    info!("Heartbeat logged for {}", hb_data.entity);

    Ok(())
}
