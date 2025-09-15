use thiserror::Error;

/// This type represents all possible errors that can occur when interacting
/// with the CLI app
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    /// Errors thrown from the `common` crate
    #[error("Common error: {0}")]
    Common(#[from] common::error::CommonError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),

    /// Error thrown when a subcommand is not handled properly
    #[error("Expected {0} command, but received a different variant")]
    VariantMismatch(String),

    #[error("Serde json error: {0}")]
    Json(#[from] serde_json::Error),
}
