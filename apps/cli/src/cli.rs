use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "skopio-cli",
    version = "1.0",
    about = "Skopio editor plugin CLI helper app"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Optional database path
    #[arg(long)]
    pub db: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Log a heartbeat (continuous coding activity)
    Heartbeat {
        #[arg(long)]
        project: String,

        #[arg(long)]
        timestamp: i32,

        #[arg(long)]
        entity: String,

        #[arg(long)]
        entity_type: String,

        #[arg(long)]
        app: String,

        #[arg(long)]
        lines: Option<i64>,

        #[arg(long)]
        cursorpos: Option<i64>,

        #[arg(short, long)]
        is_write: bool,
    },

    /// Log a coding event (like debugging, reviewing)
    Event {
        #[arg(long)]
        timestamp: i32,

        #[arg(long)]
        activity_type: String,

        #[arg(long)]
        app: String,

        #[arg(long)]
        entity: String,

        #[arg(long)]
        entity_type: String,

        #[arg(long)]
        duration: i32,

        #[arg(long)]
        project: String,

        #[arg(long)]
        language: String,

        #[arg(long)]
        end_timestamp: i32,
    },

    /// Sync stored data to the remote server
    Sync,
}

#[derive(Serialize, Deserialize)]
struct Config {
    db_path: String,
}

fn get_config_path() -> String {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    format!("{}/.skopio/config.json", home_dir.display())
}

pub fn get_or_store_db_path(cli_db: Option<String>) -> String {
    let config_path = get_config_path();

    if let Some(db) = cli_db {
        let config = Config {
            db_path: db.clone(),
        };
        fs::create_dir_all(Path::new(&config_path).parent().unwrap())
            .expect("Failed to create config directory");
        fs::write(config_path, serde_json::to_string(&config).unwrap())
            .expect("Failed to write config");
        return db;
    }

    // If no `--db` is passed, read from config
    if Path::new(&config_path).exists() {
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&config_str) {
                return config.db_path;
            }
        }
    }

    "skopio-cli-data.db".to_string()
}
