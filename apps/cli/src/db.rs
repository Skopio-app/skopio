use refinery::embed_migrations;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

use crate::utils::CliError;

embed_migrations!("./migrations");

/// Initialize the database connection
pub fn init_db(db_path: &str) -> Result<Connection, CliError> {
    let db_parent = Path::new(db_path).parent().ok_or(CliError::InvalidDbPath)?;

    if !db_parent.exists() {
        fs::create_dir_all(db_parent)?;
    }

    let mut conn = Connection::open(db_path)?;

    migrations::runner().run(&mut conn)?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_invalid_path() {
        let result = init_db("");
        assert!(matches!(result, Err(CliError::InvalidDbPath)));
    }
}
