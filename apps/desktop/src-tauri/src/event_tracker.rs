use crate::cursor_tracker::CursorTracker;
use crate::heartbeat_tracker::{get_git_branch, get_xcode_project_details, HeartbeatTracker};
use crate::keyboard_tracker::KeyboardTracker;
use crate::monitored_app::MonitoredApp;
use crate::window_tracker::WindowTracker;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Event {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    duration: Option<i64>,
    activity_type: String,
    app_name: String,
    entity_name: Option<String>,
    entity_type: Option<String>,
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

    pub fn track_event(&self, app: &str, file: &str, category: &str) {
        let now = Utc::now();
        let (project_name, project_path, entity_name, language_name) = match app {
            "Xcode" => get_xcode_project_details(),
            "Terminal" => (None, None, WindowTracker::get_terminal_directory(), None),
            "Safari" | "Google Chrome" | "Firefox" => (
                None,
                None,
                WindowTracker::get_browser_active_tab(
                    &app.parse::<MonitoredApp>().unwrap_or(MonitoredApp::Unknown),
                ),
                None,
            ),
            _ => (None, None, file.to_string(), None),
        };

        let branch_name = if app == "Xcode" {
            project_path.as_deref().map(get_git_branch)
        } else {
            None
        };

        let mut active = self.active_event.lock().unwrap();

        if let Some(prev_event) = active.clone() {
            let duration = (now - prev_event.timestamp.unwrap()).num_seconds();
            if prev_event.app_name != app
                || prev_event.entity_name.as_deref() != Some(entity_name.as_str())
            {
                info!(
                    "Event Ended: App={}, File={}, Activity={}, Duration={}s",
                    prev_event.app_name,
                    prev_event.entity_name.unwrap_or("unknown".to_string()),
                    prev_event.activity_type,
                    duration
                );
                *active = None;
            }
        }

        // Log the new event
        *active = Some(Event {
            timestamp: Some(now),
            duration: None,
            activity_type: category.to_string(),
            app_name: app.to_string(),
            entity_name: Some(entity_name.clone()),
            entity_type: Some(if app == "Xcode" { "file" } else { "window" }.to_string()),
            project_name,
            project_path,
            branch_name,
            language_name,
            end_timestamp: None,
        });

        info!("New event logged: {:?}", active);
        *self.last_activity.lock().unwrap() = Instant::now();
        let cursor_position = self.cursor_tracker.get_global_cursor_position();
        let (cursor_x, cursor_y) = cursor_position;
        self.heartbeat_tracker
            .track_heartbeat(app, &entity_name, cursor_x, cursor_y, false);
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
                        let app = window.app_name.clone();
                        let file = window.title.clone();
                        let action = detect_coding_action(&app);

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
                            if last_state.as_ref() != Some(&(app.clone(), file.clone())) {
                                last_state = Some((app.clone(), file.clone()));
                                Self::track_event(&self, &app, &file, &action);
                            }
                        } else {
                            let last_active_time = *last_activity.lock().unwrap();
                            if now.duration_since(last_active_time) >= timeout_duration {
                                if let Some(prev_event) = active_event.lock().unwrap().take() {
                                    let event_duration =
                                        (Utc::now() - prev_event.timestamp.unwrap()).num_seconds();
                                    info!(
                                        "Auto-ending inactive event: App: {}, File: {:?}, Activity: {}, Duration: {}s",
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

// TODO: Add accurate language detection and process tracking
pub fn detect_coding_action(app: &str) -> String {
    if app == "Xcode" {
        if is_compiling_xcode() {
            return "Compiling".to_string();
        } else if is_debugging_xcode() {
            return "Debugging".to_string();
        } else if is_editing_xcode() {
            return "Editing".to_string();
        }
    }
    "Focusing".to_string()
}

pub fn is_compiling_xcode() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Xcode\" to get build status")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    match output {
        Some(status) => status == "Building",
        None => false,
    }
}

pub fn is_debugging_xcode() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Xcode\" to get run state")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    match output {
        Some(state) => state == "Running",
        None => false,
    }
}

pub fn is_editing_xcode() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to tell process \"Xcode\" to get value of attribute \"AXFocusedUIElement\"")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    match output {
        Some(state) => state.contains("AXTextArea"),
        None => false,
    }
}
