use std::{
    io::{stderr, stdout, Write},
    path::Path,
};

use env_logger::Builder;
use log::{error, LevelFilter};
use thiserror::Error;

use crate::sync::SyncError;

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
        .filter(None, LevelFilter::Debug)
        .init();
}
