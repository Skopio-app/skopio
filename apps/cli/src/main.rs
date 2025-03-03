use crate::cli::{get_or_store_db_path, Cli};
use crate::db::init_db;
use clap::Parser;
use env_logger::Builder;
use log::{debug, error, info, LevelFilter};
use std::io::{stderr, stdout, Write};

mod cli;
mod db;
mod event;
mod heartbeat;
mod models;
mod sync;
mod utils;

fn main() {
    Builder::new()
        .format(|_buf, record| {
            // Prevent normal logs from appearing as warnings in plugin
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

    let cli = Cli::parse();
    let db_path = get_or_store_db_path(cli.db);

    info!("Using database path: {}", db_path);

    let conn = match init_db(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            error!("Error initializing database: {}", err);
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(cli::Commands::Heartbeat {
            timestamp,
            project,
            entity,
            entity_type,
            language,
            app,
            lines,
            cursorpos,
            is_write,
        }) => match heartbeat::log_heartbeat(
            &conn,
            timestamp,
            project,
            entity,
            entity_type,
            language,
            app,
            is_write,
            lines,
            cursorpos,
        ) {
            Ok(_) => debug!("Heartbeat logged successfully."),
            Err(err) => error!("Error logging heartbeat: {}", err),
        },

        Some(cli::Commands::Event {
            timestamp,
            activity_type,
            app,
            entity,
            entity_type,
            duration,
            project,
            language,
            end_timestamp,
        }) => match event::log_event(
            &conn,
            timestamp,
            activity_type,
            app,
            entity,
            entity_type,
            duration,
            project,
            language,
            end_timestamp,
        ) {
            Ok(_) => debug!("Event logged successfully."),
            Err(err) => error!("Error logging event: {}", err),
        },

        Some(cli::Commands::Sync) => {
            if let Err(err) = sync::sync_data(&conn) {
                error!("Sync failed: {}", err);
                std::process::exit(1);
            }
        }

        None => {
            info!("Database initialized at {}", db_path);
        }
    }
}
