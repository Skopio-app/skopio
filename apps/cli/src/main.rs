use clap::Parser;
use crate::cli::Cli;
use crate::commands::{execute_command, Commands};
use crate::db::init_db;

mod db;
mod tracking;
mod sync;
mod commands;
mod cli;
mod models;
mod heartbeat;
mod events;

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(|| "cli_data.db".to_string());
    let conn = init_db(&db_path);

    match cli.command {
        cli::Commands::Heartbeat {
            project,
            branch,
            file,
            language,
            app,
            is_write,
        } => heartbeat::log_heartbeat(&conn, project, branch, file, language, app, is_write),

        cli::Commands::Event {
            activity_type,
            app,
            duration,
        } => events::log_event(&conn, activity_type, app, duration),

        cli::Commands::Sync => sync::sync_data(&conn),
    }
}
