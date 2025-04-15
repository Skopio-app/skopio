use crate::cursor_tracker::CursorActivity;
use crate::helpers::git::get_git_branch;
use crate::monitored_app::{resolve_app_details, Entity, MonitoredApp, IGNORED_APPS};
use crate::window_tracker::Window;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio::time::{interval, Duration as TokioDuration};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Heartbeat {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub entity_name: String,
    pub entity_type: Entity,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub app_name: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursor_x: Option<f64>,
    pub cursor_y: Option<f64>,
}

pub struct HeartbeatTracker {
    last_heartbeat: Arc<Mutex<Option<Heartbeat>>>,
    last_heartbeats: Arc<DashMap<(String, String), Instant>>, // (app_name, entity_name)
    heartbeat_interval: Duration,
    // db: Arc<DBContext>,
}

impl HeartbeatTracker {
    pub fn new(
        heartbeat_interval: u64,
        //  db: Arc<DBContext>,
    ) -> Self {
        let tracker = Self {
            last_heartbeat: Arc::new(Mutex::new(None)),
            last_heartbeats: Arc::new(DashMap::new()),
            heartbeat_interval: Duration::from_secs(heartbeat_interval),
            // db,
        };

        let last_heartbeats_ref = Arc::clone(&tracker.last_heartbeats);
        let mut interval = interval(TokioDuration::from_secs(30));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                Self::cleanup_old_entries(&last_heartbeats_ref);
            }
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

        let key = (app.to_string(), entity.to_string());
        let should_log = match self.last_heartbeats.get(&key) {
            Some(entry) => now.duration_since(*entry.value()) >= self.heartbeat_interval,
            None => true,
        };

        if should_log {
            self.last_heartbeats.insert(key, now);
        }

        should_log
    }

    /// Dynamically logs a heartbeat when user activity changes
    pub async fn track_heartbeat(
        &self,
        app_name: &str,
        app_bundle_id: &str,
        app_path: &str,
        entity: &str,
        cursor_x: f64,
        cursor_y: f64,
    ) {
        let bundle_id = app_bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);
        // let db = Arc::clone(&self.db);

        if IGNORED_APPS.contains(&bundle_id) {
            return;
        }

        let (project_name, project_path, entity_name, language_name, entity_type, _category) =
            resolve_app_details(&bundle_id, app_name, app_path, entity);

        if !self.should_log_heartbeat(app_name, entity) {
            return;
        }

        // TODO: Find a reliable means of retrieving xcode line edit count
        // let lines_edited: Option<i64> = if app_name == "Xcode" { Some(0) } else { None };

        let branch_name = if app_name == "Xcode" {
            project_path.as_deref().map(get_git_branch)
        } else {
            None
        };

        let is_write: bool = if app_name == "Xcode" {
            entity.contains("Edited")
        } else {
            false
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
            lines: None,
            cursor_x: Some(cursor_x),
            cursor_y: Some(cursor_y),
        };

        info!(
            "Logged heartbeat for {:?} and entity {}",
            heartbeat.app_name, heartbeat.entity_name
        );

        // let resolved = resolve_heartbeat_ids(&db, &heartbeat)
        //     .await
        //     .unwrap_or_default();

        // let db_heartbeat = DBHeartbeat {
        //     id: None,
        //     project_id: resolved.project_id,
        //     entity_id: resolved.entity_id,
        //     branch_id: resolved.branch_id,
        //     language_id: resolved.language_id,
        //     app_id: resolved.app_id,
        //     timestamp: heartbeat.timestamp.unwrap_or_else(Utc::now),
        //     is_write: Some(heartbeat.is_write),
        //     lines: heartbeat.lines,
        //     cursorpos: heartbeat
        //         .cursor_x
        //         .map(|x| x as i64)
        //         .or(heartbeat.cursor_y.map(|y| y as i64)),
        // };

        // if let Err(e) = db_heartbeat.create(&db).await {
        //     error!("Failed to insert heartbeat: {}", e);
        // }

        *self.last_heartbeat.lock().unwrap() = Some(heartbeat);
    }

    pub async fn start_tracking(
        self: Arc<Self>,
        cursor_rx: watch::Receiver<Option<CursorActivity>>,
        mut window_rx: watch::Receiver<Option<Window>>,
    ) {
        let tracker = Arc::clone(&self);

        tokio::spawn(async move {
            loop {
                if window_rx.changed().await.is_ok() {
                    let maybe_window = window_rx.borrow_and_update().clone();
                    let maybe_cursor = cursor_rx.borrow().clone();
                    if let (Some(window), Some(cursor)) = (maybe_window, maybe_cursor) {
                        let app = window.app_name;
                        let bundle = window.bundle_id;
                        let path = window.path;
                        let file = window.title;

                        tracker
                            .track_heartbeat(&app, &bundle, &path, &file, cursor.x, cursor.y)
                            .await;
                    }
                }
            }
        });
    }
}
