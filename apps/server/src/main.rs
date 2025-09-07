use crate::{
    app::create_app,
    auth::{bearer_auth, AuthCfg},
};
use axum::middleware;
use common::keyring::Keyring;
use db::DBContext;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

mod app;
mod auth;
mod error;
mod net;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    let log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_timer(fmt::time::ChronoLocal::rfc_3339())
        .init();

    info!("🚀 Starting server...");

    let db_path = utils::get_db_path();
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db = match DBContext::new(&db_url).await {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(err) => {
            tracing::error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    let mut app = create_app(db.clone()).await;
    let token =
        Keyring::get_password("skopio", "bearer_token").unwrap_or_else(|e| Some(e.to_string()));
    let auth = AuthCfg {
        bearer: Arc::from(token.unwrap_or_default()),
    };
    app = app.layer(middleware::from_fn_with_state(auth, bearer_auth));

    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
        info!("Shutdown signal received");
    };

    if cfg!(debug_assertions) {
        let listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("Failed to start server");
        let local = listener.local_addr().unwrap();
        info!("Listening (dev) on http://{local}");

        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
        {
            error!("Server failure: {e}")
        }
        return;
    }

    #[cfg(not(debug_assertions))]
    {
        use crate::net::{bind_uds, ensure_dir_mode, mac_run_dir};

        let run_dir = mac_run_dir().expect("Could not resolve run dir");
        ensure_dir_mode(&run_dir, 0o700).expect("Could not set dir perms");

        let sock_path = run_dir.join("skopio.sock");
        let uds = bind_uds(&sock_path, 0o600).expect("Could not bind uds");

        info!("Listening (prod) on unix:://{}", sock_path.display());

        if let Err(e) = axum::serve(uds, app).await {
            error!("UDS server failure: {e}");
        }
        return;
    }

    #[allow(unreachable_code)]
    {
        let listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("Unable to bind fallback tcp listener");
        let local = listener.local_addr().unwrap();
        info!("Listening (fallback) on http://{local}");

        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
        {
            error!("Server failure: {e}");
        }
    }
}
