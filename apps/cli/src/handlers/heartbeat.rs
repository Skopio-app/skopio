use log::{debug, error};
use rusqlite::Connection;

use crate::{
    cli::Commands,
    heartbeat::{self, HeartbeatData},
};

pub fn handle_heartbeat(conn: &Connection, command: Commands) {
    if let Commands::Heartbeat {
        project,
        timestamp,
        entity,
        entity_type,
        app,
        lines,
        cursorpos,
        is_write,
    } = command
    {
        let hb_data = HeartbeatData {
            timestamp,
            project,
            entity,
            entity_type,
            app,
            lines,
            cursorpos,
            is_write,
        };

        match heartbeat::log_heartbeat(conn, hb_data) {
            Ok(_) => debug!("Heartbeat logged successfully."),
            Err(err) => error!("Error logging heartbeat: {}", err),
        }
    } else {
        error!("Expected Heartbeat command, but received a different variant");
    }
}
