use crate::cli::{get_or_store_db_path, Cli};
use crate::handlers::event::handle_event;
use crate::handlers::heartbeat::handle_heartbeat;
use crate::handlers::sync::handle_sync;
use crate::utils::{init_logger, start_db};
use clap::Parser;
use log::info;

mod cli;
mod db;
mod event;
mod handlers;
mod heartbeat;
mod sync;
mod utils;

fn main() {
    init_logger();

    let cli = Cli::parse();
    let db_path = get_or_store_db_path(cli.db);

    info!("Using database path: {}", db_path);

    let conn = start_db(&db_path);

    match cli.command {
        Some(cmd @ cli::Commands::Heartbeat { .. }) => handle_heartbeat(&conn, cmd),
        Some(cmd @ cli::Commands::Event { .. }) => handle_event(&conn, cmd),
        Some(cmd @ cli::Commands::Sync) => handle_sync(&conn, cmd),
        None => {
            info!("Database initialized at {}", db_path);
        }
    }
}
