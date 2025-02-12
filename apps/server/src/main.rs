use std::path::PathBuf;

use db::DBContext;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, EnvFilter};

mod routes;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = utils::get_application_support_path();
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db_context = DBContext::new(&db_url)
        .await
        .expect("Failed to initialize the database");

    let app = routes::create_routes(db_context);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
