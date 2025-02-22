use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "timestack-cli", version = "1.0", about = "Timestack editor plugin CLI helper app")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Optional database path
    #[arg(long)]
    pub db: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Log a heartbeat (continuous coding activity)
    Heartbeat {
        project: String,
        entity: String,
        language: String,
        app: String,
        lines: Option<i64>,
        cursorpos: Option<i64>,
        #[arg(short, long)]
        is_write: bool,
    },

    /// Log a coding event (like debugging, reviewing)
    Event {
        timestamp: String,
        activity_type: String,
        app: String,
        entity: String,
        entity_type: String,
        duration: i32,
        project: String,
        language: String,
        end_timestamp: String,
    },

    /// Sync stored data to the remote server
    Sync,
}