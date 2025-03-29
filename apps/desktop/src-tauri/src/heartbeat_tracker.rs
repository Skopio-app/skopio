use crate::cursor_tracker::CursorTracker;
use crate::helpers::app::{get_browser_active_tab, get_terminal_process};
use crate::monitored_app::MonitoredApp;
use crate::window_tracker::{Window, WindowTracker};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, thread};
use tokei::{Config, Languages};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Heartbeat {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    project_name: Option<String>,
    project_path: Option<String>,
    entity_name: String,
    entity_type: String,
    branch_name: Option<String>,
    language_name: Option<String>,
    app_name: String,
    is_write: bool,
    lines: Option<i64>,
    cursor_x: Option<f64>,
    cursor_y: Option<f64>,
}

pub struct HeartbeatTracker {
    last_heartbeat: Arc<Mutex<Option<Heartbeat>>>,
    last_heartbeats: Arc<DashMap<(String, String), Instant>>, // (app_name, entity_name)
}

impl HeartbeatTracker {
    pub fn new() -> Self {
        let tracker = Self {
            last_heartbeat: Arc::new(Mutex::new(None)),
            last_heartbeats: Arc::new(DashMap::new()),
        };

        let last_heartbeats_ref = Arc::clone(&tracker.last_heartbeats);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(30));
            Self::cleanup_old_entries(&last_heartbeats_ref);
        });

        tracker
    }

    fn cleanup_old_entries(last_heartbeats: &Arc<DashMap<(String, String), Instant>>) {
        let threshold = Instant::now() - Duration::from_secs(300); // 5 minutes
        let mut removed = 0;

        last_heartbeats.retain(|_, &mut timestamp| {
            let keep = timestamp > threshold;
            if !keep {
                removed += 1;
            }
            keep
        });

        debug!(
            "Cleanup up {} old heartbeats, remaining: {}",
            removed,
            last_heartbeats.len()
        );
    }

    fn should_log_heartbeat(&self, app: &str, entity: &str) -> bool {
        let now = Instant::now();
        let min_interval = Duration::from_secs(10);

        let key = (app.to_string(), entity.to_string());
        let should_log = match self.last_heartbeats.get(&key) {
            Some(entry) => now.duration_since(*entry.value()) >= min_interval,
            None => true,
        };

        if should_log {
            self.last_heartbeats.insert(key, now);
        }

        should_log
    }

    /// Dynamically logs a heartbeat when user activity changes
    pub fn track_heartbeat(
        &self,
        app: &str,
        file: &str,
        cursor_x: f64,
        cursor_y: f64,
        is_write: bool,
    ) {
        if !self.should_log_heartbeat(app, file) {
            return;
        }

        let (project_name, project_path, file_path, language_name) = match app {
            "Xcode" => get_xcode_project_details(),
            "Terminal" => (None, None, get_terminal_process(), None),
            "Safari" | "Google Chrome" | "Firefox" => (
                None,
                None,
                get_browser_active_tab(
                    &app.parse::<MonitoredApp>().unwrap_or(MonitoredApp::Unknown),
                ),
                None,
            ),
            _ => (None, None, file.to_string(), None),
        };

        let entity_name = file_path.clone();

        let lines_edited = if app == "Xcode" {
            detect_lines_edited_xcode(&entity_name)
        } else {
            detect_lines_edited(&entity_name)
        };

        let branch_name = if app == "Xcode" {
            project_path.as_deref().map(get_git_branch)
        } else {
            None
        };

        let heartbeat = Heartbeat {
            timestamp: Some(Utc::now()),
            project_name,
            project_path,
            entity_name,
            entity_type: if app == "Xcode" {
                "file".to_string()
            } else {
                "window".to_string()
            },
            branch_name,
            language_name,
            app_name: app.to_string(),
            is_write,
            lines: Some(lines_edited),
            cursor_x: Some(cursor_x),
            cursor_y: Some(cursor_y),
        };

        info!("Heartbeat logged: {:?}", heartbeat);
        *self.last_heartbeat.lock().unwrap() = Some(heartbeat);
    }

    pub fn start_tracking(
        self: Arc<Self>,
        cursor_tracker: Arc<CursorTracker>,
        window_tracker: Arc<WindowTracker>,
    ) {
        let heartbeat_tracker = Arc::clone(&self);
        let cursor_tracker_ref = Arc::clone(&cursor_tracker);

        cursor_tracker_ref.start_tracking({
            let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
            move |app_name, file, x, y| {
                heartbeat_tracker.track_heartbeat(app_name, file, x, y, false);
            }
        });

        let cursor_tracker_ref = Arc::clone(&cursor_tracker);
        window_tracker.start_tracking(Arc::new({
            let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
            move |window: Window| {
                let cursor_position = cursor_tracker_ref.get_global_cursor_position();
                heartbeat_tracker.track_heartbeat(
                    &window.app_name,
                    &window.title,
                    cursor_position.0,
                    cursor_position.1,
                    false,
                );
            }
        }) as Arc<dyn Fn(Window) + Send + Sync>);
    }
}

pub fn get_git_branch(project: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    output.unwrap_or_else(|| "unknown".to_string())
}

pub fn get_xcode_project_details() -> (Option<String>, Option<String>, String, Option<String>) {
    let project_path_output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Xcode\" to get path of active workspace document")
        .output();

    let project_path = project_path_output
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|p| !p.is_empty());

    let project_name = project_path
        .as_ref()
        .map(|p| p.split('/').last().unwrap_or("Unknown").to_string());

    let active_file_path_output = Command::new("osascript")
        .arg("-e")
        .arg(
            r#"
            tell application "System Events"
                tell process "Xcode"
                    try
                        return value of attribute "AXDocument" of window 1
                    on error
                        return "No active document"
                    end try
                end tell
            end tell
        "#,
        )
        .output();

    let active_file_path = active_file_path_output
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    let entity_name = match active_file_path.as_deref() {
        Some(path) if path.starts_with("file://") => path.trim_start_matches("file://").to_string(),
        Some(path) if path != "No active document" => path.to_string(),
        _ => {
            warn!("No active document detected in Xcode.");
            "".to_string()
        }
    };

    let language_name = detect_language(&entity_name);

    info!(
        "Detected Xcode project details: project_name={:?}, project_path={:?}, entity_name={}, language={:?}",
        project_name, project_path, entity_name, language_name
    );

    (project_name, project_path, entity_name, language_name)
}

fn detect_language(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    let mut languages = Languages::new();
    let config = Config::default();

    languages.get_statistics(&[path], &[], &config);

    let detected_language = languages
        .iter()
        .max_by_key(|(_, stats)| stats.code)
        .map(|(lang, _)| *lang);

    detected_language.map(|lang| lang.name().to_string())
}

pub fn detect_lines_edited_xcode(file: &str) -> i64 {
    if file.is_empty() {
        return 0;
    }

    let script =
        "tell application \"Xcode\" to tell front document to get difference count of lines"
            .to_string();

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    match output {
        Some(diff) if diff.parse::<i64>().is_ok() => diff.parse::<i64>().unwrap(),
        _ => 0,
    }
}

pub fn detect_lines_edited(file: &str) -> i64 {
    if file.is_empty() {
        return 0;
    }

    let metadata = fs::metadata(file);
    if let Ok(metadata) = metadata {
        let modified_time = metadata.modified().ok();
        if let Some(modified) = modified_time {
            let elapsed = modified.elapsed().unwrap_or_default();
            if elapsed.as_secs() < 10 {
                return 1;
            }
        }
    }

    0
}
