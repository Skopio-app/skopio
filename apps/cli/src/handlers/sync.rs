use log::error;
use rusqlite::Connection;

use crate::{cli::Commands, sync};

pub fn handle_sync(conn: &Connection, command: Commands) {
    if let Commands::Sync = command {
        if let Err(err) = sync::sync_data(conn) {
            error!("Sync failed: {}", err);
            std::process::exit(1);
        }
    } else {
        error!("Expected Sync command, but received a different variant");
    }
}
