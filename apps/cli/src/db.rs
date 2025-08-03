use refinery::embed_migrations;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

embed_migrations!("./migrations");

/// Initialize the database connection
pub fn init_db(db_path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let db_parent = Path::new(db_path)
        .parent()
        .ok_or("Invalid database path: No parent directory")?;

    if !db_parent.exists() {
        fs::create_dir_all(db_parent)?;
    }

    let mut conn = Connection::open(db_path)?;

    migrations::runner().run(&mut conn)?;

    Ok(conn)
}
