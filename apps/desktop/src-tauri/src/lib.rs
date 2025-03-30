use crate::afk_tracker::AFKTracker;
use crate::cursor_tracker::CursorTracker;
use crate::event_tracker::EventTracker;
use crate::heartbeat_tracker::HeartbeatTracker;
use crate::window_tracker::WindowTracker;
use keyboard_tracker::KeyboardTracker;
use std::sync::Arc;
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
            // Enable logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }

            let window_tracker = Arc::new(WindowTracker::new());
            let cursor_tracker = Arc::new(CursorTracker::new());
            let keyboard_tracker = Arc::new(KeyboardTracker::new());
            let afk_tracker = Arc::new(AFKTracker::new(
                Arc::clone(&cursor_tracker),
                Arc::clone(&keyboard_tracker),
            ));
            let heartbeat_tracker = Arc::new(HeartbeatTracker::new());
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
                    cursor_tracker.start_tracking(move |app_name, bundle_id, file, x, y| {
                        heartbeat_tracker.track_heartbeat(app_name, bundle_id, file, x, y, false);
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
                        heartbeat_tracker.track_heartbeat(
                            &window.app_name,
                            &window.bundle_id,
                            &window.title,
                            cursor_position.0,
                            cursor_position.1,
                            false,
                        );
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
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
