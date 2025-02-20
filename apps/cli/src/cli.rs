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
        full_path: String,
        branch: Option<String>,
        file: String,
        language: String,
        app: String,
        #[arg(short, long)]
        is_write: bool,
    },

    /// Log a coding event (like debugging, reviewing)
    Event {
        project: String,
        full_path: String,
        activity_type: String,
        app: String,
        duration: i32,
    },

    /// Sync stored data to the remote server
    Sync,
}