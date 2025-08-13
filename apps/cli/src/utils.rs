use std::{
    io::{stderr, stdout, Write},
    path::Path,
};

use common::keyring::Keyring;
use env_logger::Builder;
use log::{error, LevelFilter};
use rusqlite::Connection;
use thiserror::Error;

use crate::{db::migrations, sync::SyncError};

#[derive(Error, Debug)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),

    #[error("Invalid database path: No parent directory")]
    InvalidDbPath,

    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),

    #[error("Expected {0} command, but received a different variant")]
    VariantMismatch(String),

    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

/// Extracts the project name from the project path
pub fn extract_project_name<T: AsRef<Path>>(project_path: T) -> String {
    project_path
        .as_ref()
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn init_logger() {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .format(|_buf, record| {
            // Prevent normal logs from appearing as warnings in plugin debug console
            let mut target: Box<dyn Write> =
                if record.level() == LevelFilter::Info || record.level() == LevelFilter::Debug {
                    Box::new(stdout())
                } else {
                    Box::new(stderr())
                };

            writeln!(
                target,
                "[{} {}:{}] {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, log_level)
        .init();
}

#[allow(dead_code)]
pub fn setup_test_conn() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    migrations::runner().run(&mut conn).unwrap();
    conn
}

pub fn setup_keyring(app_name: &str) -> Result<Option<String>, CliError> {
    if cfg!(debug_assertions) {
        return Ok(None);
    }
    let password = uuid::Uuid::new_v4().to_string();
    let key = Keyring::get_or_set_password("skopio-cli", app_name, &password)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("keyring: {e}")))?;
    Ok(Some(key))
}
