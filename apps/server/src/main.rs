use std::path::PathBuf;

use db::DBContext;
use dirs::config_dir;
use tokio::{fs, net::TcpListener};
use tracing_subscriber::{fmt, EnvFilter};

mod routes;
mod handlers;

#[tokio::main]
async fn main() {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Get the database path based on the platform
    let db_path = get_database_path();
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    // Ensure the directory exists
    if let Some(parent_dir) = db_path.parent() {
        fs::create_dir_all(parent_dir).await.expect("Failed to create database directory");
    }


    let db_context = DBContext::new(Some(&db_url))
        .await
        .expect("Failed to initialize the database");

    let app = routes::create_routes(db_context);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// Dynamically construct the database path based on the platform.
fn get_database_path() -> PathBuf {
    let db_path = if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/Timestack/timestack.db
        config_dir().unwrap_or_else(|| PathBuf::from(".")).join("Timestack/timestack.db")
    } else if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\Timestack\timestack.db
        dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("Timestack/timestack.db")
    } else {
        // Linux: ~/.local/share/Timestack/timestack.db
        dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("Timestack/timestack.db")
    };

    // Ensure the path has the correct directory separators for the platform
    db_path
}
