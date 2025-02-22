use rusqlite::Connection;

/// Initialize the database connection
pub fn init_db(db_path: &str) -> Connection {
    let conn = Connection::open(db_path).expect("Failed to open database");

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS heartbeats (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        project TEXT NOT NULL,
        branch TEXT,
        file TEXT NOT NULL,
        language TEXT NOT NULL,
        app TEXT NOT NULL,
        is_write BOOLEAN DEFAULT FALSE,
        synced BOOLEAN DEFAULT FALSE
        );

        CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        activity_type TEXT NOT NULL,
        app TEXT NOT NULL,
        duration INTEGER NOT NULL,
        project TEXT NOT NULL,
        synced BOOLEAN DEFAULT FALSE,
        );
        ",
    )
    .expect("Failed to create tables");

    conn
}
