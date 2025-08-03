use std::{
    io::{stderr, stdout, Write},
    path::Path,
};

use env_logger::Builder;
use log::{error, LevelFilter};
use rusqlite::Connection;

use crate::db::init_db;

/// Extracts the project name from the project path
pub fn extract_project_name<T: AsRef<Path>>(project_path: T) -> String {
    project_path
        .as_ref()
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "UnnamedProject".to_string())
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

pub fn start_db(db_path: &str) -> Connection {
    match init_db(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            error!("Error initializing database: {}", err);
            std::process::exit(1);
        }
    }
}
