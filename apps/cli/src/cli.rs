use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "skopio-cli",
    version,
    about = "Skopio editor plugin CLI helper app"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Optional database directory path
    #[arg(long)]
    pub dir: Option<String>,

    /// The name of the app being tracked
    #[arg(long)]
    pub app: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Save a heartbeat (additional info, at a particular point in time)
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

    /// Save an event (a period of activity, with a start and end timestamp)
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

    /// Sync stored data to the main server
    Sync,
}
