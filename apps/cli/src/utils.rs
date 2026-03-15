use std::{
    io::{stderr, stdout},
    path::Path,
};

use common::keyring::Keyring;
use rusqlite::Connection;
use tracing::Level;
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};

use crate::{db::migrations, error::CliError};

/// Extracts the project name from the project path
pub fn extract_project_name<T: AsRef<Path>>(project_path: T) -> String {
    project_path
        .as_ref()
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn init_tracing() {
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let stdout_writer = stdout.with_max_level(Level::INFO);
    let stderr_writer = stderr.with_min_level(Level::WARN);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(stdout_writer.or_else(stderr_writer))
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();
}

#[allow(dead_code)]
pub fn setup_test_conn() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    migrations::runner().run(&mut conn).unwrap();
    conn
}

pub fn setup_keyring() -> Result<Option<String>, CliError> {
    if cfg!(debug_assertions) {
        return Ok(None);
    }
    let password = uuid::Uuid::new_v4().to_string();
    let key = Keyring::get_or_set_password("skopio-cli", "db-master-key", &password)
        .map_err(|e| std::io::Error::other(format!("keyring: {e}")))?;
    Ok(Some(key))
}
