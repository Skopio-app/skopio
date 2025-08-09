use rusqlite::Connection;

use crate::{
    cli::Commands,
    heartbeat::{self, HeartbeatData},
    utils::CliError,
};

pub fn handle_heartbeat(conn: &Connection, command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Heartbeat {
            project,
            timestamp,
            entity,
            entity_type,
            app,
            lines,
            cursorpos,
            is_write,
        } => {
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

            heartbeat::save_heartbeat(conn, hb_data)?;
            Ok(())
        }
        _ => Err(CliError::VariantMismatch("Heartbeat".to_string())),
    }
}
