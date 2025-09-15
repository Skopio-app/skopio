use crate::utils::{extract_project_name, CliError};
use chrono::{Duration, TimeZone, Utc};
use common::{client::Transport, models::inputs::EventInput};
use log::{debug, info};
use rusqlite::{Connection, Row};

pub async fn sync_data(conn: &Connection) -> Result<(), CliError> {
    let transport = Transport::new()?;
    let events = fetch_unsynced_events(conn)?;

    if events.is_empty() {
        debug!("No data to sync");
        return Ok(());
    }

    if !events.is_empty() {
        sync_to_server(&transport, "events", &events).await?;
        conn.execute("UPDATE events SET synced = 1 WHERE synced = 0", [])?;
        info!("{} events synced successfully!", events.len())
    }

    delete_synced_data(conn)?;

    Ok(())
}

fn fetch_unsynced_events(conn: &Connection) -> Result<Vec<EventInput>, rusqlite::Error> {
    let mut stmt = conn
        .prepare(
            "SELECT timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, source, end_timestamp
                 FROM events WHERE synced = 0"
                )?;
    let rows = stmt.query_map([], |row| parse_event(row))?;
    Ok(rows.flatten().collect())
}

fn parse_event(row: &Row) -> rusqlite::Result<EventInput> {
    let ts: Option<i64> = row.get(0)?;
    let end_ts: Option<i64> = row.get(10)?;
    let project_path: String = row.get(6)?;

    Ok(EventInput {
        timestamp: ts.map(|t| Utc.timestamp_opt(t, 0).single().unwrap_or_default()),
        category: row.get(1)?,
        app_name: row.get(2)?,
        entity_name: row.get(3)?,
        entity_type: row.get(4)?,
        duration: row.get(5)?,
        project_name: extract_project_name(&project_path),
        project_path,
        branch_name: row.get(7)?,
        language_name: row.get(8)?,
        source_name: row.get(9)?,
        end_timestamp: end_ts.map(|t| Utc.timestamp_opt(t, 0).single().unwrap_or_default()),
    })
}

async fn sync_to_server<T: serde::Serialize>(
    transport: &Transport,
    path: &str,
    data: &T,
) -> Result<(), CliError> {
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    let json = serde_json::to_string(data)?;

    let _ = transport.post_json(&path, &json).await?;
    Ok(())
}

fn delete_synced_data(conn: &Connection) -> Result<(), CliError> {
    let cutoff = Utc::now() - Duration::days(15);
    let cutoff_unix = cutoff.timestamp();

    let deleted_events = conn.execute(
        "DELETE FROM events WHERE synced = 1 AND timestamp < ?1",
        [cutoff_unix],
    )?;

    debug!("Deleted {} old synced events", deleted_events);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        event::{save_event, EventData},
        utils::setup_test_conn,
    };
    use rusqlite::params;

    #[test]
    fn test_fetch_unsynced_events() {
        let conn = setup_test_conn();
        let now = Utc::now().timestamp();

        let test_event = EventData {
            timestamp: now as i32,
            category: "Coding".into(),
            app: "Code".into(),
            source: "skopio-vscode".into(),
            entity: "main.rs".into(),
            entity_type: "File".into(),
            duration: 300,
            project: "/tmp/project".into(),
            language: Some("Rust".into()),
            end_timestamp: (now + 100) as i32,
        };

        save_event(&conn, test_event).unwrap();

        let events = fetch_unsynced_events(&conn).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category, "Coding");
        assert_eq!(events[0].project_path, "/tmp/project");
    }

    #[test]
    fn test_delete_synced_data() {
        let conn = setup_test_conn();
        let old_ts = (Utc::now() - Duration::days(20)).timestamp();

        conn.execute(
            "INSERT INTO events (timestamp, category, app, entity_name, entity_type, duration, project_path, branch, language, source, end_timestamp, synced)
             VALUES (?1, 'Coding', 'VSCode', 'main.rs', 'file', 100, '/tmp/project', 'main', 'Rust', 'skopio-vscode', ?2, 1)",
            params![old_ts, old_ts + 100],
        ).unwrap();

        delete_synced_data(&conn).unwrap();

        let remaining_events: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();

        assert_eq!(remaining_events, 0);
    }
}
