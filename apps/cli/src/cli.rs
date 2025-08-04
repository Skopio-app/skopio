use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    name = "skopio-cli",
    version,
    about = "Skopio editor plugin CLI helper app"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Optional database path
    #[arg(long)]
    pub db: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Log a heartbeat (additional info, at a particular point in time)
    Heartbeat {
        #[arg(long, short)]
        /// The project path.
        project: String,

        #[arg(long, short)]
        /// The timestamp as the point the heartbeat is generated
        timestamp: i32,

        #[arg(long, short)]
        /// The entity path
        entity: String,

        #[arg(long)]
        /// The entity type, be it an app, file or URL
        entity_type: String,

        #[arg(long, short)]
        /// The app being tracked
        app: String,

        #[arg(long, short)]
        /// The number of lines edited
        lines: Option<i64>,

        #[arg(long, short)]
        /// The cursor position at the point the heartbeat is generated.
        cursorpos: Option<i64>,

        #[arg(short, long)]
        /// Whether editing is in progress at the point of heartbeat generation.
        is_write: bool,
    },

    /// Log an event (a period of activity, with a start and end timestamp)
    Event {
        #[arg(long, short)]
        /// The start of the recorded event
        timestamp: i32,

        #[arg(long, short)]
        /// The event category, eg. Coding, Debugging, etc.
        category: String,

        #[arg(long, short)]
        /// The app being tracked
        app: String,

        #[arg(long, short)]
        /// The entity path
        entity: String,

        #[arg(long)]
        /// The entity type, be it an app, file or URL
        entity_type: String,

        #[arg(long, short)]
        /// The duration of an event
        duration: i32,

        #[arg(long, short)]
        /// The full path of the currently open project
        project: String,

        #[arg(long)]
        /// The end timestamp of the event
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
