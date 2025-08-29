use crate::monitored_app::{resolve_app_details, Category, Entity, MonitoredApp, IGNORED_APPS};
use crate::trackers::SOURCE;
use crate::tracking_service::TrackingService;
use crate::utils::config::TrackedApp;
use chrono::{DateTime, Utc};
use common::git::find_git_branch;
use db::desktop::events::Event as DBEvent;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};
use tokio::time::{Duration, Instant};

use super::keyboard_tracker::KeyboardTracker;
use super::mouse_tracker::MouseTracker;
use super::window_tracker::Window;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: Category,
    pub app_name: String,
    pub entity_name: Option<String>,
    pub entity_type: Option<Entity>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub end_timestamp: Option<DateTime<Utc>>,
}

pub struct EventTracker {
    active_event: Arc<Mutex<Option<Event>>>,
    last_activity: Arc<Mutex<Instant>>,
    cursor_tracker: Arc<MouseTracker>,
    keyboard_tracker: Arc<KeyboardTracker>,
    tracker: Arc<dyn TrackingService>,
    tracked_apps_rx: watch::Receiver<Vec<TrackedApp>>,
    allowed_ids: Arc<RwLock<HashSet<String>>>,
}

impl EventTracker {
    pub fn new(
        cursor_tracker: Arc<MouseTracker>,
        keyboard_tracker: Arc<KeyboardTracker>,
        tracker: Arc<dyn TrackingService>,
        tracked_apps_rx: watch::Receiver<Vec<TrackedApp>>,
    ) -> Self {
        let initial_allowed: HashSet<String> = tracked_apps_rx
            .borrow()
            .iter()
            .map(|t| t.bundle_id.clone())
            .collect();

        Self {
            active_event: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            cursor_tracker,
            keyboard_tracker,
            tracker,
            tracked_apps_rx,
            allowed_ids: Arc::new(RwLock::new(initial_allowed)),
        }
    }

    pub async fn track_event(
        &self,
        app_name: &str,
        app_bundle_id: &str,
        app_path: &str,
        entity: &str,
    ) {
        let bundle_id = app_bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);

        if IGNORED_APPS.contains_key(&bundle_id) {
            return;
        }

        {
            let allowed = self.allowed_ids.read().await;
            if !allowed.contains(app_bundle_id) {
                return;
            }
        }

        let now = Utc::now();
        let (project_name, project_path, entity_name, language_name, entity_type, category) =
            resolve_app_details(&bundle_id, app_name, app_path, entity);

        let branch_name = if app_name == "Xcode" {
            project_path.as_ref().and_then(find_git_branch)
        } else {
            None
        };

        let mut should_reset_event = false;

        {
            let mut active = self.active_event.lock().await;
            if let Some(prev_event) = active.as_ref() {
                let duration = (now - prev_event.timestamp.unwrap()).num_seconds();
                if prev_event.app_name != app_name
                    || prev_event.entity_name.as_deref() != Some(entity_name.as_str())
                {
                    let mut ended_event = prev_event.clone();
                    ended_event.duration = Some(duration);
                    ended_event.end_timestamp = Some(now);

                    let db_event = DBEvent {
                        id: None,
                        timestamp: ended_event.timestamp,
                        duration: ended_event.duration,
                        category: Some(ended_event.category.to_string()),
                        app_name: ended_event.app_name,
                        entity_name: ended_event.entity_name,
                        entity_type: Some(ended_event.entity_type.unwrap().to_string()),
                        project_name: ended_event.project_name,
                        project_path: ended_event.project_path,
                        branch_name: ended_event.branch_name,
                        language_name: ended_event.language_name,
                        source_name: SOURCE.to_string(),
                        end_timestamp: ended_event.end_timestamp,
                    };

                    self.tracker
                        .insert_event(db_event)
                        .await
                        .unwrap_or_else(|error| error!("Failed to batch event: {}", error));

                    info!(
                        "Event Ended: App={}, Entity={:?}, Activity={}, Duration={}s",
                        prev_event.app_name, prev_event.entity_name, prev_event.category, duration
                    );
                    should_reset_event = true;
                }
            }

            if should_reset_event {
                *active = None;
            }

            *active = Some(Event {
                timestamp: Some(now),
                duration: None,
                category: category,
                app_name: app_name.to_string(),
                entity_name: Some(entity_name.clone()),
                entity_type: Some(entity_type),
                project_name,
                project_path,
                branch_name,
                language_name,
                end_timestamp: None,
            });
        }

        *self.last_activity.lock().await = Instant::now();
    }

    pub fn start_tracking(
        self: Arc<Self>,
        mut window_rx: watch::Receiver<Option<Window>>,
        mut afk_timeout_rx: watch::Receiver<u64>,
    ) -> tokio::task::JoinHandle<()> {
        let mut tracked_rx = self.tracked_apps_rx.clone();
        let allowed_ids = Arc::clone(&self.allowed_ids);

        tokio::spawn(async move {
            let mut last_state = None;
            let mut last_check = Instant::now();

            loop {
                let afk_timeout_secs = *afk_timeout_rx.borrow_and_update();
                let afk_threshold = Duration::from_secs(afk_timeout_secs);

                tokio::select! {
                    changed = tracked_rx.changed() => {
                        if changed.is_ok() {
                            let latest = tracked_rx.borrow().clone();
                            let mut w = allowed_ids.write().await;
                            w.clear();
                            w.extend(latest.into_iter().map(|t| t.bundle_id));
                        }
                    }
                    changed = window_rx.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        let window = match window_rx.borrow_and_update().clone() {
                            Some(w) => w,
                            None => continue,
                        };

                        let app_name = window.app_name.clone();
                        let bundle_id = window.bundle_id.clone();
                        let file = window.title.clone();
                        let app_path = window.path.clone();

                        let activity_detected = {
                            let mouse_buttons = self.cursor_tracker.get_pressed_mouse_buttons();
                            let keys_pressed = self.keyboard_tracker.get_pressed_keys();
                            let mouse_active = self.cursor_tracker.has_mouse_moved();
                            let mouse_clicked = mouse_buttons.left || mouse_buttons.right || mouse_buttons.middle || mouse_buttons.other;
                            let keyboard_active = !keys_pressed.is_empty();
                            mouse_active || mouse_clicked || keyboard_active
                        };

                         let changed = last_state
                                .as_ref()
                                .map(|(prev_app, prev_file)| prev_app != &app_name || prev_file != &file)
                                .unwrap_or(true);

                        if changed {
                            last_state = Some((app_name.clone(), file.clone()));
                            self.track_event(&app_name, &bundle_id, &app_path, &file).await;
                        }

                        if activity_detected {
                            *self.last_activity.lock().await = Instant::now();
                            last_check = Instant::now();
                        }
                    }

                    _ = tokio::time::sleep_until(last_check + afk_threshold) => {
                        let last_active_time = *self.last_activity.lock().await;
                        if Instant::now().duration_since(last_active_time) >= afk_threshold {
                            self.end_active_event().await;
                            last_state = None;
                        }
                        last_check = Instant::now();
                    }
                }
            }
        })
    }

    async fn end_active_event(&self) {
        let mut active = self.active_event.lock().await;
        if let Some(prev_event) = active.take() {
            let event_duration = (Utc::now() - prev_event.timestamp.unwrap()).num_seconds();

            let mut ended_event = prev_event.clone();
            ended_event.duration = Some(event_duration);
            ended_event.end_timestamp = Some(Utc::now());

            let db_event = DBEvent {
                id: None,
                timestamp: ended_event.timestamp,
                duration: ended_event.duration,
                category: Some(ended_event.category.to_string()),
                app_name: ended_event.app_name,
                entity_name: ended_event.entity_name,
                entity_type: Some(ended_event.entity_type.unwrap().to_string()),
                project_name: ended_event.project_name,
                project_path: ended_event.project_path,
                branch_name: ended_event.branch_name,
                language_name: ended_event.language_name,
                source_name: SOURCE.to_string(),
                end_timestamp: ended_event.end_timestamp,
            };

            self.tracker
                .insert_event(db_event)
                .await
                .unwrap_or_else(|error| error!("Failed to batch event: {}", error));

            info!(
                "Auto-ending event: App: {}, Entity: {:?}, Activity: {}, Duration: {}s",
                prev_event.app_name, prev_event.entity_name, prev_event.category, event_duration
            );
        }
    }

    pub async fn stop_tracking(&self) {
        self.end_active_event().await;
        info!("Event tracker stopped");
    }
}
