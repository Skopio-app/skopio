use crate::afk_tracker::AFKTracker;
use crate::cursor_tracker::CursorTracker;
use crate::event_tracker::EventTracker;
use crate::heartbeat_tracker::HeartbeatTracker;
use crate::window_tracker::WindowTracker;
use chrono::Local;
use db::DBContext;
use helpers::{
    config::{AppConfig, CONFIG},
    db::get_db_path,
};
use keyboard_tracker::KeyboardTracker;
use log::error;
use std::sync::Arc;
use tauri::AppHandle;
use window_tracker::Window;

mod afk_tracker;
mod cursor_tracker;
mod event_tracker;
mod heartbeat_tracker;
mod helpers;
mod keyboard_tracker;
mod monitored_app;
mod window_tracker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
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
                if let Err(e) = async_setup(&app_handle_clone).await {
                    error!("Failed async setup: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::helpers::config::get_config,
            crate::helpers::config::set_theme,
            crate::helpers::config::set_afk_timeout,
            crate::helpers::config::set_heartbeat_interval,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

async fn async_setup(app_handle: &AppHandle) -> Result<(), anyhow::Error> {
    let config = AppConfig::load(app_handle)?;
    *CONFIG.lock().unwrap() = config.clone();

    let db_path = get_db_path(app_handle);
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db = match DBContext::new(&db_url).await {
        Ok(db) => Arc::new(db),
        Err(err) => {
            error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    let window_tracker = Arc::new(WindowTracker::new());
    let cursor_tracker = Arc::new(CursorTracker::new());
    let keyboard_tracker = Arc::new(KeyboardTracker::new());
    let afk_tracker = Arc::new(AFKTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&keyboard_tracker),
        config.afk_timeout,
        Arc::clone(&db),
    ));
    let heartbeat_tracker = Arc::new(HeartbeatTracker::new(
        config.heartbeat_interval,
        Arc::clone(&db),
    ));
    let event_tracker = Arc::new(EventTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&heartbeat_tracker),
        Arc::clone(&keyboard_tracker),
    ));

    tauri::async_runtime::spawn({
        let afk_tracker = Arc::clone(&afk_tracker);
        async move {
            afk_tracker.start_tracking();
        }
    });

    tauri::async_runtime::spawn({
        let keyboard_tracker = Arc::clone(&keyboard_tracker);
        async move {
            keyboard_tracker.start_tracking();
        }
    });

    tauri::async_runtime::spawn({
        let cursor_tracker = Arc::clone(&cursor_tracker);
        let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
        async move {
            cursor_tracker.start_tracking(move |app_name, bundle_id, app_path, file, x, y| {
                let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
                let app_name = app_name.to_string();
                let bundle_id = bundle_id.to_string();
                let app_path = app_path.to_string();
                let file = file.to_string();

                tauri::async_runtime::spawn(async move {
                    heartbeat_tracker
                        .track_heartbeat(&app_name, &bundle_id, &app_path, &file, x, y)
                        .await;
                });
            });
        }
    });

    tauri::async_runtime::spawn({
        let window_tracker = Arc::clone(&window_tracker);
        let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
        let cursor_tracker = Arc::clone(&cursor_tracker);
        async move {
            window_tracker.start_tracking(Arc::new(move |window: Window| {
                let cursor_position = cursor_tracker.get_global_cursor_position();
                let heartbeat_tracker = Arc::clone(&heartbeat_tracker);

                tauri::async_runtime::spawn(async move {
                    heartbeat_tracker
                        .track_heartbeat(
                            &window.app_name,
                            &window.bundle_id,
                            &window.path,
                            &window.title,
                            cursor_position.0,
                            cursor_position.1,
                        )
                        .await;
                });
            }));
        }
    });

    tauri::async_runtime::spawn({
        let event_tracker = Arc::clone(&event_tracker);
        async move {
            event_tracker.start_tracking();
        }
    });

    tauri::async_runtime::spawn({
        let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
        let cursor_tracker = Arc::clone(&cursor_tracker);
        let window_tracker = Arc::clone(&window_tracker);
        async move {
            heartbeat_tracker.start_tracking(cursor_tracker, window_tracker);
        }
    });

    Ok(())
}
