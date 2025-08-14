use common::language::detect_language;
use rusqlite::Connection;

use crate::{
    cli::Commands,
    event::{self, EventData},
    utils::CliError,
};

pub fn handle_event(conn: &Connection, command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Event {
            timestamp,
            category,
            app,
            entity,
            entity_type,
            duration,
            project,
            source,
            end_timestamp,
        } => {
            let language = detect_language(&entity);

            let event_data = EventData {
                timestamp,
                category,
                app,
                entity,
                entity_type,
                duration,
                project,
                language,
                source,
                end_timestamp,
            };

            event::save_event(conn, event_data)?;
            Ok(())
        }
        _ => Err(CliError::VariantMismatch("Event".to_string())),
    }
}
