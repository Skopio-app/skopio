use crate::cli::Cli;
use crate::config::get_or_store_db_path;
use crate::db::init_db;
use crate::handlers::event::handle_event;
use crate::handlers::heartbeat::handle_heartbeat;
use crate::handlers::sync::handle_sync;
use crate::utils::{init_logger, CliError};
use clap::Parser;
use log::{error, info};

mod cli;
mod config;
mod db;
mod event;
mod handlers;
mod heartbeat;
mod sync;
mod utils;

fn main() {
    init_logger();

    if let Err(err) = run() {
        error!("Fatal error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let db_path = get_or_store_db_path(cli.db, &cli.app);

    info!("Using database path: {}", db_path);

    let conn = init_db(&db_path, &cli.app)?;

    match cli.command {
        Some(cmd @ cli::Commands::Heartbeat { .. }) => handle_heartbeat(&conn, cmd),
        Some(cmd @ cli::Commands::Event { .. }) => handle_event(&conn, cmd),
        Some(cmd @ cli::Commands::Sync) => handle_sync(&conn, cmd),
        None => {
            info!("Database initialized at {}", db_path);
            Ok(())
        }
    }
}
