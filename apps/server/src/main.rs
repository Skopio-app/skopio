use crate::app::create_app;
use db::DBContext;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod app;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_timer(fmt::time::ChronoLocal::rfc_3339())
        .init();

    info!("🚀 Starting server...");

    let db_path = utils::get_application_support_path();
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db = match DBContext::new(&db_url).await {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(err) => {
            tracing::error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    let app = create_app(db.clone()).await;

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("Server failure");
}
