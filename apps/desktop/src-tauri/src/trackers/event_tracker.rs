use crate::monitored_app::{resolve_app_details, BundleIdExt, Category, Entity};
use crate::trackers::SOURCE;
use crate::tracking_service::TrackingService;
use crate::utils::ax::cache::AxSnapshotCache;
use crate::utils::ax::provider::SystemAxProvider;
use crate::utils::config::TrackedApp;
use chrono::{DateTime, Utc};
use common::git::find_git_branch;
use db::desktop::events::Event as DBEvent;

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};
use tokio::time::{self, Duration, Instant, Sleep};
use tracing::{error, info};

use super::window_tracker::Window;

#[derive(Debug, Clone)]
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
    tracker: Arc<dyn TrackingService>,
    tracked_apps_rx: watch::Receiver<Vec<TrackedApp>>,
    allowed_ids: Arc<RwLock<HashSet<String>>>,
    ax_cache: Arc<AxSnapshotCache<SystemAxProvider>>,
}

impl EventTracker {
    pub fn new(
        tracker: Arc<dyn TrackingService>,
        tracked_apps_rx: watch::Receiver<Vec<TrackedApp>>,
        ax_cache: Arc<AxSnapshotCache<SystemAxProvider>>,
    ) -> Self {
        let initial_allowed: HashSet<String> = tracked_apps_rx
            .borrow()
            .iter()
            .map(|t| t.bundle_id.clone())
            .collect();

        Self {
            active_event: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            tracker,
            tracked_apps_rx,
            allowed_ids: Arc::new(RwLock::new(initial_allowed)),
            ax_cache,
        }
    }

    pub async fn track_event(
        &self,
        app_name: &str,
        app_bundle_id: &str,
        app_path: &str,
        entity: &str,
        pid: i32,
    ) {
        if app_bundle_id.is_ignored_bundle() {
            return;
        }

        {
            let allowed = self.allowed_ids.read().await;
            if !allowed.contains(app_bundle_id) {
                return;
            }
        }

        let now = Utc::now();
        let snapshot = self.ax_cache.snapshot().await;
        let app_details =
            resolve_app_details(app_bundle_id, app_name, app_path, entity, &snapshot, pid);

        let branch_name = if app_name == "Xcode" {
            app_details.project_path.as_ref().and_then(find_git_branch)
        } else {
            None
        };

        let mut should_reset_event = false;

        {
            let mut active = self.active_event.lock().await;
            if let Some(prev_event) = active.as_ref() {
                let duration = (now - prev_event.timestamp.unwrap()).num_seconds();
                if prev_event.app_name != app_name
                    || prev_event.entity_name.as_deref() != Some(app_details.entity.as_str())
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
                        .insert_event(&db_event)
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
                category: app_details.category,
                app_name: app_name.to_string(),
                entity_name: Some(app_details.entity.clone()),
                entity_type: Some(app_details.entity_type),
                project_name: app_details.project_name,
                project_path: app_details.project_path,
                branch_name,
                language_name: app_details.language,
                end_timestamp: None,
            });
        }

        *self.last_activity.lock().await = Instant::now();
    }

    pub fn start_tracking(
        self: Arc<Self>,
        mut window_rx: watch::Receiver<Option<Window>>,
        mut afk_timeout_rx: watch::Receiver<u64>,
        mut afk_state_rx: watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        let mut tracked_rx = self.tracked_apps_rx.clone();
        let allowed_ids = Arc::clone(&self.allowed_ids);

        tokio::spawn(async move {
            let mut last_state = None;
            let afk_timeout_secs = *afk_timeout_rx.borrow_and_update();
            let mut afk_threshold = Duration::from_secs(afk_timeout_secs);
            let mut sleep: Pin<Box<Sleep>> = Box::pin(time::sleep(afk_threshold));

            sleep.as_mut().reset(Instant::now() + afk_threshold);

            loop {
                tokio::select! {
                    // Config changed: update AFK threshold and reset the timer.
                    changed = afk_timeout_rx.changed() => {
                        if changed.is_err() { break; }
                        afk_threshold = Duration::from_secs(*afk_timeout_rx.borrow());
                        sleep.as_mut().reset(Instant::now() + afk_threshold);
                    }
                    // AFK state changed: if AFK started, end right away.
                    changed = afk_state_rx.changed() => {
                        if changed.is_err() { break; }
                        if *afk_state_rx.borrow() {
                            self.end_active_event().await;
                            last_state = None;
                        } else {
                            sleep.as_mut().reset(Instant::now() + afk_threshold);
                        }
                    }
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

                        let app_name = window.app_name;
                        let bundle_id = window.bundle_id;
                        let file = window.title;
                        let app_path = window.path;
                        let pid = window.pid;

                         let changed = last_state
                                .as_ref()
                                .map(|(prev_app, prev_file)| prev_app != &app_name || prev_file != &file)
                                .unwrap_or(true);

                        if changed {
                            last_state = Some((app_name.clone(), file.clone()));
                            self.track_event(&app_name, &bundle_id, &app_path, &file, pid).await;
                        }
                    }

                    _ = &mut sleep => {
                        let last_active_time = *self.last_activity.lock().await;
                        if Instant::now().duration_since(last_active_time) >= afk_threshold {
                            self.end_active_event().await;
                            last_state = None;
                        }
                        sleep.as_mut().reset(Instant::now() + afk_threshold);
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
                .insert_event(&db_event)
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
