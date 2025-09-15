use crate::cli::Cli;
use crate::db::init_db;
use crate::error::CliError;
use crate::handlers::event::handle_event;
use crate::handlers::sync::handle_sync;
use crate::utils::init_logger;
use clap::Parser;
use log::error;

mod cli;
mod db;
mod error;
mod event;
mod handlers;
mod sync;
mod utils;

#[tokio::main]
async fn main() {
    init_logger();

    if let Err(err) = run().await {
        error!("Fatal error: {:#}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    let conn = init_db()?;

    match cli.command {
        Some(cmd @ cli::Commands::Event { .. }) => handle_event(&conn, cmd),
        Some(cmd @ cli::Commands::Sync) => handle_sync(&conn, cmd).await,
        None => Ok(()),
    }
}
