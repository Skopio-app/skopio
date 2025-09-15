use refinery::embed_migrations;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

use crate::utils::{setup_keyring, CliError};

embed_migrations!("./migrations");

pub fn get_connection<P: AsRef<Path>>(
    path: P,
    encryption_key: Option<String>,
) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    if let Some(key) = encryption_key {
        conn.pragma_update(None, "key", &key)?;
    }
    Ok(conn)
}

pub fn init_db() -> Result<Connection, CliError> {
    let cli_dir = dirs::home_dir().unwrap_or_default().join(".skopio");

    if !cli_dir.exists() {
        fs::create_dir_all(&cli_dir)?;
    }

    let key_opt = setup_keyring()?;

    let db_path = cli_dir.join("cli.db");

    let mut conn = get_connection(db_path, key_opt)?;
    migrations::runner().run(&mut conn)?;
    Ok(conn)
}
