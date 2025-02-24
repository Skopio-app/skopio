use crate::cli::Cli;
use crate::db::init_db;
use clap::Parser;

mod cli;
mod db;
mod events;
mod heartbeat;
mod models;
mod sync;
mod utils;

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(|| "cli_data.db".to_string());
    let conn = init_db(&db_path);

    match cli.command {
        cli::Commands::Heartbeat {
            project,
            entity,
            language,
            app,
            lines,
            cursorpos,
            is_write,
        } => heartbeat::log_heartbeat(&conn, project, entity, language, app, is_write, lines, cursorpos),

        cli::Commands::Event {
            timestamp,
            activity_type,
            app,
            entity,
            entity_type,
            duration,
            project,
            language,
            end_timestamp
        } => events::log_event(&conn, timestamp, activity_type, app, entity, entity_type, duration, project, language, end_timestamp),

        cli::Commands::Sync => sync::sync_data(&conn),
    }
}
