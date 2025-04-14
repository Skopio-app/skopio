use crate::cursor_tracker::CursorTracker;
use crate::heartbeat_tracker::HeartbeatTracker;
use crate::helpers::git::get_git_branch;
use crate::keyboard_tracker::KeyboardTracker;
use crate::monitored_app::{resolve_app_details, Category, Entity, MonitoredApp, IGNORED_APPS};
use crate::window_tracker::Window;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tokio::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub activity_type: Category,
    pub app_name: String,
    pub entity_name: Option<String>,
    pub entity_type: Option<Entity>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end_timestamp: Option<DateTime<Utc>>,
}

pub struct EventTracker {
    active_event: Arc<Mutex<Option<Event>>>,
    last_activity: Arc<Mutex<Instant>>,
    cursor_tracker: Arc<CursorTracker>,
    heartbeat_tracker: Arc<HeartbeatTracker>,
    keyboard_tracker: Arc<KeyboardTracker>,
    // db: Arc<DBContext>,
}

impl EventTracker {
    pub fn new(
        cursor_tracker: Arc<CursorTracker>,
        heartbeat_tracker: Arc<HeartbeatTracker>,
        keyboard_tracker: Arc<KeyboardTracker>,
        // db: Arc<DBContext>,
    ) -> Self {
        Self {
            active_event: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            cursor_tracker,
            heartbeat_tracker,
            keyboard_tracker,
            // db,
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

        if IGNORED_APPS.contains(&bundle_id) {
            return;
        }

        let now = Utc::now();
        let (project_name, project_path, entity_name, language_name, entity_type, category) =
            resolve_app_details(&bundle_id, app_name, app_path, entity);

        let branch_name = if app_name == "Xcode" {
            project_path.as_deref().map(get_git_branch)
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

                    // let db = Arc::clone(&self.db);
                    // let resolved = resolve_event_ids(&db, ended_event.clone()).await;

                    // if let Ok(resolved) = resolved {
                    //     let db_event = DBEvent {
                    //         id: None,
                    //         timestamp: to_naive_datetime(ended_event.timestamp)
                    //             .unwrap_or_else(|| Utc::now().naive_utc()),
                    //         duration: ended_event.duration,
                    //         activity_type: ended_event.activity_type.to_string(),
                    //         app_id: resolved.app_id.unwrap_or_default(),
                    //         entity_id: resolved.entity_id,
                    //         project_id: resolved.project_id,
                    //         branch_id: resolved.branch_id,
                    //         language_id: resolved.language_id,
                    //         end_timestamp: to_naive_datetime(ended_event.end_timestamp),
                    //     };

                    //     if let Err(e) = db_event.create(&db).await {
                    //         error!("Failed to insert db event: {}", e);
                    //     };
                    // }

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

            if should_reset_event {
                *active = None;
            }

            *active = Some(Event {
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
        }

        *self.last_activity.lock().await = Instant::now();
        let cursor_position = self.cursor_tracker.get_global_cursor_position();
        let (cursor_x, cursor_y) = cursor_position;

        self.heartbeat_tracker
            .track_heartbeat(
                app_name,
                app_bundle_id,
                app_path,
                &entity_name,
                cursor_x,
                cursor_y,
            )
            .await;
    }

    pub fn start_tracking(
        self: Arc<Self>,
        mut window_rx: watch::Receiver<Option<Window>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut last_state = None;
            let timeout_duration = Duration::from_secs(120);
            let mut last_check = Instant::now();

            loop {
                tokio::select! {
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

                        if activity_detected {
                            *self.last_activity.lock().await = Instant::now();
                            last_check = Instant::now();

                            let changed = last_state
                                .as_ref()
                                .map(|(prev_app, prev_file)| prev_app != &app_name || prev_file != &file)
                                .unwrap_or(true);

                            if changed {
                                last_state = Some((app_name.clone(), file.clone()));
                                self.track_event(&app_name, &bundle_id, &app_path, &file).await;
                            }
                        }
                    }

                    _ = tokio::time::sleep_until(last_check + timeout_duration) => {
                        let last_active_time = *self.last_activity.lock().await;
                        if Instant::now().duration_since(last_active_time) >= timeout_duration {
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

            // let resolved = resolve_event_ids(&self.db, ended_event.clone()).await;

            // if let Ok(resolved) = resolved {
            //     let db_event = DBEvent {
            //         id: None,
            //         timestamp: to_naive_datetime(ended_event.timestamp)
            //             .unwrap_or_else(|| Utc::now().naive_utc()),
            //         duration: ended_event.duration,
            //         activity_type: ended_event.activity_type.to_string(),
            //         app_id: resolved.app_id.unwrap_or_default(),
            //         entity_id: resolved.entity_id,
            //         project_id: resolved.project_id,
            //         branch_id: resolved.branch_id,
            //         language_id: resolved.language_id,
            //         end_timestamp: to_naive_datetime(ended_event.end_timestamp),
            //     };

            //     if let Err(e) = db_event.create(&self.db).await {
            //         error!("Failed to insert inactive event: {}", e);
            //     } else {
            //         info!(
            //             "Auto-ending inactive event: App: {}, Entity: {:?}, Activity: {}, Duration: {}s",
            //             prev_event.app_name,
            //             prev_event.entity_name,
            //             prev_event.activity_type,
            //             event_duration
            //         );
            //     }
            // }
        }
    }
}
