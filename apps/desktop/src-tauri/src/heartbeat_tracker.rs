use crate::cursor_tracker::CursorTracker;
use crate::helpers::git::get_git_branch;
use crate::monitored_app::{resolve_app_details, Entity, MonitoredApp};
use crate::window_tracker::{Window, WindowTracker};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Heartbeat {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    project_name: Option<String>,
    project_path: Option<String>,
    entity_name: String,
    entity_type: Entity,
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
            "Cleaned up {} old heartbeats, remaining: {}",
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
        app_name: &str,
        app_bundle_id: &str,
        entity: &str,
        cursor_x: f64,
        cursor_y: f64,
        is_write: bool,
    ) {
        let bundle_id = app_bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);
        let (project_name, project_path, entity_name, language_name, entity_type, _category) =
            resolve_app_details(&bundle_id, entity);

        if !self.should_log_heartbeat(app_name, entity) {
            return;
        }

        let lines_edited: Option<i64> = if app_name == "Xcode" { Some(0) } else { None };

        let branch_name = if app_name == "Xcode" {
            project_path.as_deref().map(get_git_branch)
        } else {
            None
        };

        let heartbeat = Heartbeat {
            timestamp: Some(Utc::now()),
            project_name,
            project_path,
            entity_name,
            entity_type,
            branch_name,
            language_name,
            app_name: app_name.to_string(),
            is_write,
            lines: lines_edited,
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
            move |app_name, app_bundle_id, file, x, y| {
                heartbeat_tracker.track_heartbeat(app_name, app_bundle_id, file, x, y, false);
            }
        });

        let cursor_tracker_ref = Arc::clone(&cursor_tracker);
        window_tracker.start_tracking(Arc::new({
            let heartbeat_tracker = Arc::clone(&heartbeat_tracker);
            move |window: Window| {
                let cursor_position = cursor_tracker_ref.get_global_cursor_position();
                heartbeat_tracker.track_heartbeat(
                    &window.app_name,
                    &window.bundle_id,
                    &window.title,
                    cursor_position.0,
                    cursor_position.1,
                    false,
                );
            }
        }) as Arc<dyn Fn(Window) + Send + Sync>);
    }
}
