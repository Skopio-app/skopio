use crate::cli::{get_or_store_db_path, Cli};
use crate::db::init_db;
use clap::Parser;

mod cli;
mod db;
mod event;
mod heartbeat;
mod models;
mod sync;
mod utils;

fn main() {
    let cli = Cli::parse();
    let db_path = get_or_store_db_path(cli.db);

    println!("Using database path: {}", db_path);
    let conn = match init_db(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error initializing database: {}", err);
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
            Ok(_) => println!("Heartbeat logged successfully."),
            Err(err) => eprintln!("Error logging heartbeat: {}", err),
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
            Ok(_) => println!("Event logged successfully."),
            Err(err) => eprintln!("Error logging event: {}", err),
        },

        Some(cli::Commands::Sync) => {
            if let Err(err) = sync::sync_data(&conn) {
                eprintln!("Sync failed: {}", err);
                std::process::exit(1);
            }
        }

        None => {
            println!("Database initialized at {}", db_path);
        }
    }
}
