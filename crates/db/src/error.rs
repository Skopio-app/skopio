use thiserror::Error;

/// Errors returned by the DB access layer
#[derive(Error, Debug)]
pub enum DBError {
    /// Used for low-level driver / query errors
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Input contained a timestamp that couldn't be parsed
    #[error("Timestamp parse error: {0}")]
    Parse(#[from] chrono::ParseError),

    /// A UUID string could not be parsed/validated
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    /// A required input field is missing
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    /// The supplied configuration is not supported
    #[error("Unsupported configuration: {0}")]
    Unsupported(&'static str),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}
