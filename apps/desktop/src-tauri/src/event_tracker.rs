use crate::cursor_tracker::CursorTracker;
use crate::heartbeat_tracker::HeartbeatTracker;
use crate::helpers::git::get_git_branch;
use crate::keyboard_tracker::KeyboardTracker;
use crate::monitored_app::{resolve_app_details, Category, Entity, MonitoredApp, IGNORED_APPS};
use crate::window_tracker::WindowTracker;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Event {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    duration: Option<i64>,
    activity_type: Category,
    app_name: String,
    entity_name: Option<String>,
    entity_type: Option<Entity>,
    project_name: Option<String>,
    project_path: Option<String>,
    branch_name: Option<String>,
    language_name: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    end_timestamp: Option<DateTime<Utc>>,
}

pub struct EventTracker {
    active_event: Arc<Mutex<Option<Event>>>,
    last_activity: Arc<Mutex<Instant>>,
    cursor_tracker: Arc<CursorTracker>,
    heartbeat_tracker: Arc<HeartbeatTracker>,
    keyboard_tracker: Arc<KeyboardTracker>,
}

impl EventTracker {
    pub fn new(
        cursor_tracker: Arc<CursorTracker>,
        heartbeat_tracker: Arc<HeartbeatTracker>,
        keyboard_tracker: Arc<KeyboardTracker>,
    ) -> Self {
        Self {
            active_event: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            cursor_tracker,
            heartbeat_tracker,
            keyboard_tracker,
        }
    }

    pub async fn track_event(&self, app_name: &str, app_bundle_id: &str, entity: &str) {
        let bundle_id = app_bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);

        if IGNORED_APPS.contains(&bundle_id) {
            return;
        }

        let now = Utc::now();
        let (project_name, project_path, entity_name, language_name, entity_type, category) =
            resolve_app_details(&bundle_id, entity);

        let branch_name = if app_name == "Xcode" {
            project_path.as_deref().map(get_git_branch)
        } else {
            None
        };

        let mut should_reset_event = false;

        {
            let active = self.active_event.lock().unwrap();
            if let Some(prev_event) = active.as_ref() {
                let duration = (now - prev_event.timestamp.unwrap()).num_seconds();
                if prev_event.app_name != app_name
                    || prev_event.entity_name.as_deref() != Some(entity_name.as_str())
                {
                    info!(
                        "Event Ended: App={}, Entity={}, Activity={}, Duration={}s",
                        prev_event.app_name,
                        prev_event
                            .entity_name
                            .clone()
                            .unwrap_or("unknown".to_string()),
                        prev_event.activity_type,
                        duration
                    );
                    should_reset_event = true;
                }
            }
        }

        if should_reset_event {
            *self.active_event.lock().unwrap() = None;
        }

        *self.active_event.lock().unwrap() = Some(Event {
            timestamp: Some(now),
            duration: None,
            activity_type: category,
            app_name: app_name.to_string(),
            entity_name: Some(entity_name.clone()),
            entity_type: Some(entity_type),
            project_name,
            project_path,
            branch_name,
            language_name,
            end_timestamp: None,
        });

        *self.last_activity.lock().unwrap() = Instant::now();
        let cursor_position = self.cursor_tracker.get_global_cursor_position();
        let (cursor_x, cursor_y) = cursor_position;

        self.heartbeat_tracker
            .track_heartbeat(app_name, app_bundle_id, &entity_name, cursor_x, cursor_y)
            .await;
    }

    pub fn start_tracking(self: Arc<Self>) {
        let last_activity = Arc::clone(&self.last_activity);
        let active_event = Arc::clone(&self.active_event);

        thread::spawn(move || {
            let mut last_check = Instant::now();
            let mut last_state: Option<(String, String)> = None;
            let timeout_duration = Duration::from_secs(120);

            loop {
                if last_check.elapsed() >= Duration::from_millis(500) {
                    let now = Instant::now();

                    if let Some(window) = WindowTracker::get_active_window() {
                        let app_name = window.app_name.clone();
                        let bundle_id = window.bundle_id;
                        let file = window.title.clone();

                        let mouse_buttons = self.cursor_tracker.get_pressed_mouse_buttons();
                        let keys_pressed = self.keyboard_tracker.get_pressed_keys();

                        let mouse_active = self.cursor_tracker.has_mouse_moved();

                        let mouse_clicked = mouse_buttons.left
                            || mouse_buttons.right
                            || mouse_buttons.middle
                            || mouse_buttons.other;
                        let keyboard_active = !keys_pressed.is_empty();

                        let activity_detected = mouse_active || mouse_clicked || keyboard_active;

                        if activity_detected {
                            *last_activity.lock().unwrap() = now;
                            if last_state.as_ref() != Some(&(app_name.clone(), file.clone())) {
                                last_state = Some((app_name.clone(), file.clone()));
                                let tracker = Arc::clone(&self);
                                let app_name = app_name.clone();
                                let bundle_id = bundle_id.clone();
                                let file = file.clone();
                                tauri::async_runtime::spawn(async move {
                                    tracker.track_event(&app_name, &bundle_id, &file).await;
                                });
                            }
                        } else {
                            let last_active_time = *last_activity.lock().unwrap();
                            if now.duration_since(last_active_time) >= timeout_duration {
                                if let Some(prev_event) = active_event.lock().unwrap().take() {
                                    let event_duration =
                                        (Utc::now() - prev_event.timestamp.unwrap()).num_seconds();
                                    info!(
                                        "Auto-ending inactive event: App: {}, Entity: {:?}, Activity: {}, Duration: {}s",
                                        prev_event.app_name,
                                        prev_event.entity_name,
                                        prev_event.activity_type,
                                        event_duration
                                    )
                                }
                            }
                        }
                    }
                    last_check = Instant::now();
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
