use common::language::detect_language;
use log::{debug, error};
use rusqlite::Connection;

use crate::{
    cli::Commands,
    event::{self, EventData},
};

pub fn handle_event(conn: &Connection, command: Commands) {
    if let Commands::Event {
        timestamp,
        category,
        app,
        entity,
        entity_type,
        duration,
        project,
        end_timestamp,
    } = command
    {
        let language = detect_language(&project);

        let event_data = EventData {
            timestamp,
            category,
            app,
            entity,
            entity_type,
            duration,
            project,
            language,
            end_timestamp,
        };

        match event::log_event(conn, event_data) {
            Ok(_) => debug!("Event logged successfully."),
            Err(err) => error!("Error logging event: {}", err),
        }
    } else {
        error!("Expected Event command, but received a different variant");
    }
}
