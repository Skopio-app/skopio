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
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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

        #[arg(long, short)]
        /// The name of the extension/plugin that has generated the event
        source: String,

        #[arg(long)]
        /// The end timestamp of the event
        end_timestamp: i32,
    },

    /// Sync stored data to the main server
    Sync,
}
