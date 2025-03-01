use std::fs;
use std::path::Path;
use rusqlite::Connection;

/// Initialize the database connection
pub fn init_db(db_path: &str) -> Connection {
    let db_parent = Path::new(db_path).parent().unwrap();

    if !db_parent.exists() {
        fs::create_dir_all(db_parent).expect("Failed to create database directory");
    }

    let conn = Connection::open(db_path).expect("Failed to open database");

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS heartbeats (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp INTEGER NOT NULL,
        project_path TEXT NOT NULL,
        branch TEXT,
        entity_name TEXT NOT NULL,
        entity_type TEXT,
        language TEXT NOT NULL,
        app TEXT NOT NULL,
        is_write BOOLEAN DEFAULT FALSE,
        lines INTEGER,
        cursorpos INTEGER,
        synced BOOLEAN DEFAULT FALSE
        );

        CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp INTEGER NOT NULL,
        activity_type TEXT NOT NULL,
        app TEXT NOT NULL,
        entity_name TEXT,
        entity_type TEXT,
        duration INTEGER NOT NULL,
        project_path TEXT NOT NULL,
        branch TEXT,
        language TEXT,
        end_timestamp INTEGER,
        synced BOOLEAN DEFAULT FALSE
        );
        ",
    )
    .expect("Failed to create tables");

    conn
}
