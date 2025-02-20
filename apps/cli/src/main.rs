use crate::cli::Cli;
use crate::db::init_db;
use clap::Parser;

mod cli;
mod db;
mod events;
mod heartbeat;
mod models;
mod sync;

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(|| "cli_data.db".to_string());
    let conn = init_db(&db_path);

    match cli.command {
        cli::Commands::Heartbeat {
            project,
            full_path,
            branch,
            file,
            language,
            app,
            is_write,
        } => heartbeat::log_heartbeat(&conn, project, full_path, branch, file, language, app, is_write),

        cli::Commands::Event {
            project,
            full_path,
            activity_type,
            app,
            duration,
        } => events::log_event(&conn, project, full_path, activity_type, app, duration),

        cli::Commands::Sync => sync::sync_data(&conn),
    }
}
