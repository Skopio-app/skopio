use buffered_service::BufferedTrackingService;
use chrono::Local;
use db::DBContext;
use helpers::{
    config::{AppConfig, CONFIG},
    db::get_db_path,
};
use log::error;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use trackers::{
    afk_tracker::AFKTracker, cursor_tracker::CursorTracker, event_tracker::EventTracker,
    heartbeat_tracker::HeartbeatTracker, keyboard_tracker::KeyboardTracker,
    window_tracker::WindowTracker,
};
use tracking_service::{DBService, TrackingService};

mod buffered_service;
mod helpers;
mod monitored_app;
mod trackers;
mod tracking_service;

#[tokio::main]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let cursor_tracker = Arc::new(CursorTracker::new());
    let keyboard_tracker = Arc::new(KeyboardTracker::new());
    let window_tracker = Arc::new(WindowTracker::new());

    let specta_builder = make_specta_builder();

    tauri::Builder::default()
        .manage(Arc::clone(&cursor_tracker))
        .manage(Arc::clone(&keyboard_tracker))
        .manage(Arc::clone(&window_tracker))
        .setup(move |app| {
            let app_handle = app.handle().clone();

            specta_builder.mount_events(&app_handle);
            // Enable logging in debug mode
            if cfg!(debug_assertions) {
                app_handle.plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .format(|out, message, record| {
                            let local_time = Local::now().format("%Y-%m-%d %H:%M:%S");
                            let module = record.target();
                            let line = record.line().unwrap_or_default();
                            out.finish(format_args!(
                                "[{}][{}:{}][{}] {}",
                                local_time,
                                module,
                                line,
                                record.level(),
                                message
                            ));
                        })
                        .build(),
                )?;
            }

            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_trackers(&app_handle_clone).await {
                    error!("Failed async setup: {}", e);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let cursor_tracker = window.state::<Arc<CursorTracker>>();
                let keyboard_tracker = window.state::<Arc<KeyboardTracker>>();
                let buffered_service = window.state::<Arc<BufferedTrackingService>>();
                cursor_tracker.stop_tracking();
                keyboard_tracker.stop_tracking();

                let buffered_service = Arc::clone(&buffered_service);
                tokio::spawn(async move {
                    buffered_service.shutdown().await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

async fn setup_trackers(app_handle: &AppHandle) -> Result<(), anyhow::Error> {
    AppConfig::load(app_handle)
        .await
        .unwrap_or_else(|e| error!("Failed to load app config: {}", e));

    let config = Arc::clone(&CONFIG);

    let db_path = get_db_path(app_handle);
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db_result = tokio::spawn(async move { DBContext::new(&db_url).await })
        .await
        .expect("DB task panicked");

    let db = match db_result {
        Ok(db) => Arc::new(db),
        Err(err) => {
            error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    let raw_service = Arc::new(DBService::new(Arc::clone(&db)));
    let buffered_service = Arc::new(BufferedTrackingService::new(raw_service, Arc::clone(&db)));
    app_handle.manage(Arc::clone(&buffered_service));

    let service_trait: Arc<dyn TrackingService> = buffered_service.clone();

    let window_tracker = app_handle.state::<Arc<WindowTracker>>();
    let cursor_tracker = app_handle.state::<Arc<CursorTracker>>();
    let keyboard_tracker = app_handle.state::<Arc<KeyboardTracker>>();
    let afk_tracker = Arc::new(AFKTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&keyboard_tracker),
        Arc::clone(&config),
        Arc::clone(&service_trait),
    ));

    let heartbeat_tracker = Arc::new(HeartbeatTracker::new(
        Arc::clone(&config),
        Arc::clone(&service_trait),
    ));
    let event_tracker = Arc::new(EventTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&keyboard_tracker),
        Arc::clone(&config),
        Arc::clone(&service_trait),
    ));

    let window_tracker_ref = Arc::clone(&window_tracker);
    window_tracker_ref.start_tracking();

    let event_window_rx = window_tracker.subscribe();
    let heartbeat_window_rx = window_tracker.subscribe();

    cursor_tracker.start_tracking();

    afk_tracker.start_tracking();

    let keyboard_tracker = Arc::clone(&keyboard_tracker);
    keyboard_tracker.start_tracking();

    tokio::spawn({
        async move {
            if let Err(e) = event_tracker.start_tracking(event_window_rx).await {
                error!("Event tracker failed: {}", e);
            }
        }
    });

    tokio::spawn({
        let cursor_tracker = Arc::clone(&cursor_tracker);
        let cursor_rx = cursor_tracker.subscribe();
        async move {
            heartbeat_tracker
                .start_tracking(cursor_rx, heartbeat_window_rx)
                .await;
        }
    });

    Ok(())
}

fn make_specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    let builder = tauri_specta::Builder::<R>::new()
        .commands(tauri_specta::collect_commands![
            crate::helpers::config::get_config,
            crate::helpers::config::set_theme::<tauri::Wry>,
            crate::helpers::config::set_afk_timeout::<tauri::Wry>,
            crate::helpers::config::set_heartbeat_interval::<tauri::Wry>,
        ])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .header("/* eslint-disable */\n// @ts-nocheck\n\n"),
            "../src/types/tauri.gen.ts",
        )
        .expect("Failed to export typescript bindings");

    builder
}
