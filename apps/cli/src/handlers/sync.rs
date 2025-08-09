use rusqlite::Connection;

use crate::{cli::Commands, sync, utils::CliError};

pub fn handle_sync(conn: &Connection, command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Sync => {
            sync::sync_data(conn)?;
            Ok(())
        }
        _ => Err(CliError::VariantMismatch("Sync".to_string())),
    }
}
