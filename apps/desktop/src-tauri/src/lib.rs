use crate::afk_tracker::AFKTracker;
use crate::cursor_tracker::CursorTracker;
use crate::event_tracker::EventTracker;
use crate::heartbeat_tracker::HeartbeatTracker;
use crate::window_tracker::WindowTracker;
use buffered_service::BufferedTrackingService;
use chrono::Local;
use db::DBContext;
use helpers::{
    config::{AppConfig, CONFIG},
    db::get_db_path,
};
use keyboard_tracker::KeyboardTracker;
use log::error;
use tracking_service::{DBService, TrackingService};
// use ppfileruard;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

mod afk_tracker;
mod buffered_service;
mod cursor_tracker;
mod event_tracker;
mod heartbeat_tracker;
mod helpers;
mod keyboard_tracker;
mod monitored_app;
mod tracking_service;
mod window_tracker;

#[tokio::main]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let cursor_tracker = Arc::new(CursorTracker::new());
    let keyboard_tracker = Arc::new(KeyboardTracker::new());
    let window_tracker = Arc::new(WindowTracker::new());
    tauri::Builder::default()
        .manage(Arc::clone(&cursor_tracker))
        .manage(Arc::clone(&keyboard_tracker))
        .manage(Arc::clone(&window_tracker))
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
            tokio::spawn(async move {
                if let Err(e) = async_setup(&app_handle_clone).await {
                    error!("Failed async setup: {}", e);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let cursor_tracker = window.state::<Arc<CursorTracker>>();
                let keyboard_tracker = window.state::<Arc<KeyboardTracker>>();
                cursor_tracker.stop_tracking();
                keyboard_tracker.stop_tracking();
                // force_heap_dump();
            }
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

    let raw_service = Arc::new(DBService::new(Arc::clone(&db)));
    let buffered_service: Arc<dyn TrackingService> =
        Arc::new(BufferedTrackingService::new(raw_service));

    let window_tracker = app_handle.state::<Arc<WindowTracker>>();
    let cursor_tracker = app_handle.state::<Arc<CursorTracker>>();
    let keyboard_tracker = app_handle.state::<Arc<KeyboardTracker>>();
    let afk_tracker = Arc::new(AFKTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&keyboard_tracker),
        config.afk_timeout,
        Arc::clone(&buffered_service),
    ));

    let heartbeat_tracker = Arc::new(HeartbeatTracker::new(
        config.heartbeat_interval,
        Arc::clone(&buffered_service),
    ));
    let event_tracker = Arc::new(EventTracker::new(
        Arc::clone(&cursor_tracker),
        Arc::clone(&keyboard_tracker),
        config.afk_timeout,
        Arc::clone(&buffered_service),
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

    // let guard = ProfilerGuard::new(100).unwrap();

    // tokio::spawn(async move {
    //     tokio::time::sleep(Duration::from_secs(30)).await;

    //     if let Ok(report) = guard.report().build() {
    //         let file = std::fs::File::create("pprof_flamegraph.svg").unwrap();
    //         report.flamegraph(file).unwrap();
    //         debug!("🔥 Flamegraph written to pprof_flamegraph.svg");
    //     } else {
    //         error!("Failed to build pprof report.");
    //     }
    // });

    Ok(())
}

// pub fn force_heap_dump() {
//     use std::ffi::CString;
//     use tikv_jemalloc_sys::mallctl;

//     let name = CString::new("prof.dump").unwrap();
//     unsafe {
//         let ret = mallctl(
//             name.as_ptr(),
//             std::ptr::null_mut(),
//             std::ptr::null_mut(),
//             std::ptr::null_mut(),
//             0,
//         );

//         if ret != 0 {
//             error!("⚠️ jemalloc prof.dump failed: {}", ret);
//         } else {
//             debug!("✅ jemalloc heap dump written manually.");
//         }
//     }
// }
