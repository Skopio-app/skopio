use common::{git::find_git_branch, language::detect_language};
use log::info;
use rusqlite::{params, Connection};

use crate::utils::CliError;

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

pub fn save_heartbeat(conn: &Connection, hb_data: HeartbeatData) -> Result<(), CliError> {
    let branch_name = find_git_branch(&hb_data.entity);
    let language = detect_language(&hb_data.entity);

    conn.execute(
        "INSERT INTO heartbeats (timestamp, project_path, branch, entity_name, entity_type, language, app, is_write, lines, cursorpos, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
        params![
            hb_data.timestamp,
            hb_data.project,
            branch_name,
            hb_data.entity,
            hb_data.entity_type,
            language,
            hb_data.app,
            hb_data.is_write,
            hb_data.lines,
            hb_data.cursorpos,
        ],
    )?;

    info!("Heartbeat saved for {}", hb_data.entity);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::setup_test_conn;

    #[test]
    fn test_save_heartbeat_inserts_into_db() {
        let conn = setup_test_conn();

        let test_heartbeat = HeartbeatData {
            timestamp: 1720,
            project: "/tmp/test-project".into(),
            entity: "main.rs".into(),
            entity_type: "File".into(),
            app: "Code".into(),
            is_write: false,
            lines: Some(10),
            cursorpos: Some(62),
        };

        save_heartbeat(&conn, test_heartbeat).unwrap();

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM heartbeats").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }
}
