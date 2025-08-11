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

pub fn init_db(db_path: &str, app_name: &str) -> Result<Connection, CliError> {
    let parent = Path::new(db_path).parent().ok_or(CliError::InvalidDbPath)?;

    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    let key_opt = setup_keyring(app_name)?;

    let mut conn = get_connection(db_path, key_opt)?;
    migrations::runner().run(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_invalid_path() {
        let result = init_db("", "Code");
        assert!(matches!(result, Err(CliError::InvalidDbPath)));
    }
}
