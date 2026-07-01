use crate::monitored_app::{BundleIdExt, Category, Entity};
use crate::trackers::SOURCE;
use crate::tracking_service::TrackingService;
use crate::utils::ax::cache::AxSnapshotCache;
use crate::utils::ax::provider::SystemAxProvider;
use crate::utils::config::TrackedApp;
use chrono::{DateTime, Utc};
use common::git::find_git_branch;
use db::desktop::events::Event as DBEvent;

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast, watch};
use tracing::{error, info};

use super::afk_tracker::AfkState;
use super::power_monitor::PowerEvent;
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

impl From<Event> for DBEvent {
    fn from(value: Event) -> Self {
        Self {
            id: None,
            timestamp: value.timestamp,
            duration: value.duration,
            category: Some(value.category.to_string()),
            app_name: value.app_name,
            entity_name: value.entity_name,
            entity_type: value.entity_type.map(|t| t.to_string()),
            project_name: value.project_name,
            project_path: value.project_path,
            branch_name: value.branch_name,
            language_name: value.language_name,
            source_name: SOURCE.to_string(),
            end_timestamp: value.end_timestamp,
        }
    }
}

pub struct EventTracker {
    active_event: Arc<Mutex<Option<Event>>>,
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
        let now = Utc::now();

        if app_bundle_id.is_ignored_bundle() {
            self.end_active_event_at(now).await;
            return;
        }

        {
            let allowed = self.allowed_ids.read().await;
            if !allowed.contains(app_bundle_id) {
                self.end_active_event_at(now).await;
                return;
            }
        }

        let snapshot = self.ax_cache.snapshot().await;
        let app_details =
            app_bundle_id.resolve_app_details(app_name, app_path, entity, &snapshot, pid);

        let branch_name = if app_bundle_id.is_xcode_bundle() {
            app_details.project_path.as_ref().and_then(find_git_branch)
        } else {
            None
        };

        let new_event = Event {
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
        };

        let event_to_insert = {
            let mut active = self.active_event.lock().await;
            if let Some(prev_event) = active.as_ref()
                && prev_event.app_name == app_name
                && prev_event.entity_name.as_deref() == Some(app_details.entity.as_str())
            {
                return;
            }

            let event_to_insert = active
                .take()
                .and_then(|prev_event| Self::finish_event(prev_event, now));

            *active = Some(new_event);
            event_to_insert
        };

        if let Some((db_event, ended_event)) = event_to_insert {
            self.insert_ended_event(db_event, &ended_event).await;
        }
    }

    pub fn start_tracking(
        self: Arc<Self>,
        mut window_rx: watch::Receiver<Option<Window>>,
        mut afk_state_rx: watch::Receiver<AfkState>,
        mut power_rx: broadcast::Receiver<PowerEvent>,
    ) -> tokio::task::JoinHandle<()> {
        let mut tracked_rx = self.tracked_apps_rx.clone();
        let allowed_ids = Arc::clone(&self.allowed_ids);

        tokio::spawn(async move {
            let mut last_state: Option<(Arc<str>, Arc<str>)> = None;

            loop {
                tokio::select! {
                    // AFK state changed: if AFK started, end right away.
                    changed = afk_state_rx.changed() => {
                        if changed.is_err() { break; }
                        let afk_state = *afk_state_rx.borrow_and_update();
                        match afk_state {
                            AfkState::Afk { started_at } => {
                                self.end_active_event_at(started_at).await;
                                last_state = None;
                            }
                            AfkState::Active => {
                                self.track_current_window(&window_rx, &mut last_state).await;
                            }
                        }
                    }
                    power_event = power_rx.recv() => {
                        match power_event {
                            Ok(PowerEvent::WillSleep { at }) => {
                                self.end_active_event_at(at).await;
                                last_state = None;
                            }
                            Ok(PowerEvent::DidWake { .. }) => {
                                let is_active = *afk_state_rx.borrow() == AfkState::Active;
                                if is_active {
                                    self.track_current_window(&window_rx, &mut last_state).await;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                self.end_active_event_at(Utc::now()).await;
                                last_state = None;
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    changed = tracked_rx.changed() => {
                        if changed.is_ok() {
                            let latest = tracked_rx.borrow().clone();
                            let mut w = allowed_ids.write().await;
                            w.clear();
                            w.extend(latest.into_iter().map(|t| t.bundle_id));
                            drop(w);

                            last_state = None;
                            self.track_current_window(&window_rx, &mut last_state).await;
                        }
                    }
                    changed = window_rx.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        let current_window = window_rx.borrow_and_update().clone();
                        let window = match current_window {
                            Some(w) => w,
                            None => {
                                self.end_active_event_at(Utc::now()).await;
                                last_state = None;
                                continue;
                            }
                        };

                        self.track_window(window, &mut last_state).await;
                    }
                }
            }
        })
    }

    async fn track_current_window(
        &self,
        window_rx: &watch::Receiver<Option<Window>>,
        last_state: &mut Option<(Arc<str>, Arc<str>)>,
    ) {
        let current_window = window_rx.borrow().clone();
        if let Some(window) = current_window {
            self.track_window(window, last_state).await;
        }
    }

    async fn track_window(&self, window: Window, last_state: &mut Option<(Arc<str>, Arc<str>)>) {
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
            *last_state = Some((app_name.clone(), file.clone()));
            self.track_event(&app_name, &bundle_id, &app_path, &file, pid)
                .await;
        }
    }

    pub async fn end_active_event_at(&self, end_at: DateTime<Utc>) {
        let event_to_insert = {
            let mut active = self.active_event.lock().await;
            active
                .take()
                .and_then(|prev_event| Self::finish_event(prev_event, end_at))
        };

        if let Some((db_event, ended_event)) = event_to_insert {
            self.insert_ended_event(db_event, &ended_event).await;
        }
    }

    fn finish_event(prev_event: Event, end_at: DateTime<Utc>) -> Option<(DBEvent, Event)> {
        let start_at = prev_event.timestamp?;
        let event_duration = (end_at - start_at).num_seconds().max(0);

        if event_duration == 0 {
            return None;
        }

        let mut ended_event = prev_event;
        ended_event.duration = Some(event_duration);
        ended_event.end_timestamp = Some(end_at);

        let db_event = ended_event.clone().into();
        Some((db_event, ended_event))
    }

    async fn insert_ended_event(&self, db_event: DBEvent, ended_event: &Event) {
        self.tracker
            .insert_event(&db_event)
            .await
            .unwrap_or_else(|error| error!("Failed to batch event: {}", error));

        info!(
            "App={}, Entity={:?}, Activity={}, Duration={}s",
            ended_event.app_name,
            ended_event.entity_name,
            ended_event.category,
            ended_event.duration.unwrap_or_default()
        );
    }

    async fn end_active_event(&self) {
        self.end_active_event_at(Utc::now()).await;
    }

    pub async fn stop_tracking(&self) {
        self.end_active_event().await;
        info!("Event tracker stopped");
    }
}
