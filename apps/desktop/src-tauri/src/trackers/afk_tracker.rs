use chrono::{DateTime, Utc};
use db::desktop::afk_events::AFKEvent;
use std::{
    sync::{Arc, Mutex as StdMutex},
    time::Instant,
};
use tokio::sync::{Mutex, RwLock, broadcast, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration, interval};
use tracing::{error, info};

use crate::trackers::TrackerLifecycle;
use crate::trackers::input_activity::InputActivity;
use crate::trackers::power_monitor::PowerEvent;
use crate::tracking_service::TrackingService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfkState {
    Active,
    Afk { started_at: DateTime<Utc> },
}

pub struct AFKTracker {
    last_activity: Arc<RwLock<DateTime<Utc>>>,
    last_activity_instant: Arc<RwLock<Instant>>,
    afk_start: Arc<Mutex<Option<DateTime<Utc>>>>,
    afk_timeout_rx: watch::Receiver<u64>,
    tracker: Arc<dyn TrackingService>,
    afk_state_tx: watch::Sender<AfkState>,
    afk_state_rx: watch::Receiver<AfkState>,
    shutdown_tx: watch::Sender<bool>,
    task: StdMutex<Option<JoinHandle<()>>>,
}

impl AFKTracker {
    pub fn new(afk_timeout_rx: watch::Receiver<u64>, tracker: Arc<dyn TrackingService>) -> Self {
        let (afk_state_tx, afk_state_rx) = watch::channel(AfkState::Active);
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            last_activity: Arc::new(RwLock::new(Utc::now())),
            last_activity_instant: Arc::new(RwLock::new(Instant::now())),
            afk_start: Arc::new(Mutex::new(None)),
            afk_timeout_rx,
            tracker,
            afk_state_tx,
            afk_state_rx,
            shutdown_tx,
            task: StdMutex::new(None),
        }
    }

    pub fn start_tracking(
        self: Arc<Self>,
        mut power_rx: broadcast::Receiver<PowerEvent>,
        mut input_rx: broadcast::Receiver<InputActivity>,
    ) {
        if let Some(existing) = self.task.lock().unwrap().take() {
            existing.abort();
        }

        let _ = self.shutdown_tx.send(false);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let tracker = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    changed = shutdown_rx.changed() => {
                        if changed.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                    }

                    _ = interval.tick() => {
                        tracker.mark_afk_idle_at(Instant::now()).await;
                    }

                    input_event = input_rx.recv() => {
                        match input_event {
                            Ok(activity) => {
                                tracker.handle_input_activity(activity.at, activity.monotonic_at).await;
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {}
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }

                    power_event = power_rx.recv() => {
                        match power_event {
                            Ok(PowerEvent::WillSleep { at }) => {
                                tracker.mark_afk_at(at).await;
                            }
                            Ok(PowerEvent::DidWake { .. }) => {}
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                tracker.mark_afk_idle_at(Instant::now()).await;
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        });

        let mut task = self.task.lock().unwrap();
        *task = Some(handle);
    }

    pub async fn stop_tracking(&self) {
        self.stop_task().await;
        self.flush_open_afk_event().await;
    }

    async fn stop_task(&self) {
        let _ = self.shutdown_tx.send(true);
        let task = self.task.lock().unwrap().take();
        if let Some(task) = task
            && let Err(error) = task.await
            && !error.is_cancelled()
        {
            error!("AFK tracker task failed during shutdown: {}", error);
        }
    }

    async fn flush_open_afk_event(&self) {
        let mut afk_time = self.afk_start.lock().await;

        if let Some(afk_start_time) = *afk_time {
            let now = Utc::now();
            let afk_duration = (now - afk_start_time).num_seconds().max(0);

            info!(
                "AFK tracker stopping. Flushing AFK event from {} to {} ({}s)",
                afk_start_time, now, afk_duration
            );

            let afk_event = AFKEvent {
                id: None,
                afk_start: Some(afk_start_time),
                afk_end: Some(now),
                duration: Some(afk_duration),
            };

            if let Err(err) = self.tracker.insert_afk(&afk_event).await {
                error!("Failed to flush AFK event on stop: {}", err);
            }

            *afk_time = None;
        } else {
            info!("AFK tracker stopping. No AFK event to flush.");
        }
    }

    async fn mark_afk_idle_at(&self, now: Instant) {
        let last_activity_instant = *self.last_activity_instant.read().await;
        let timeout_secs = *self.afk_timeout_rx.borrow();
        let timeout = Duration::from_secs(timeout_secs);

        if now.duration_since(last_activity_instant) < timeout {
            return;
        }

        let afk_time = self.afk_start.lock().await;
        if afk_time.is_some() {
            return;
        }

        let last_activity_time = *self.last_activity.read().await;
        let afk_start_at = self.afk_start_time(last_activity_time, timeout_secs);
        drop(afk_time);
        self.mark_afk_at(afk_start_at).await;
    }

    async fn mark_afk_at(&self, afk_start_at: DateTime<Utc>) {
        let mut afk_time = self.afk_start.lock().await;
        if afk_time.is_some() {
            return;
        }

        info!("User went AFK at: {}", afk_start_at);
        *afk_time = Some(afk_start_at);
        self.update_state(AfkState::Afk {
            started_at: afk_start_at,
        });
    }

    async fn handle_input_activity(&self, now: DateTime<Utc>, monotonic_now: Instant) {
        self.mark_afk_idle_at(monotonic_now).await;

        let afk_start_time = {
            let mut afk_time = self.afk_start.lock().await;
            afk_time.take()
        };

        if let Some(afk_start_time) = afk_start_time {
            self.record_afk_return(afk_start_time, now).await;
        }

        *self.last_activity.write().await = now;
        *self.last_activity_instant.write().await = monotonic_now;
        self.update_state(AfkState::Active);
    }

    async fn record_afk_return(&self, afk_start_time: DateTime<Utc>, now: DateTime<Utc>) {
        let afk_duration = (now - afk_start_time).num_seconds().max(0);
        info!(
            "User returned at: {} (AFK Duration: {}s)",
            now, afk_duration
        );

        if afk_duration == 0 {
            return;
        }

        let afk_event = AFKEvent {
            id: None,
            afk_start: Some(afk_start_time),
            afk_end: Some(now),
            duration: Some(afk_duration),
        };

        self.tracker
            .insert_afk(&afk_event)
            .await
            .unwrap_or_else(|error| error!("Failed to batch afk event: {}", error));
    }

    fn update_state(&self, state: AfkState) {
        if *self.afk_state_tx.borrow() == state {
            return;
        }

        if let Err(e) = self.afk_state_tx.send(state) {
            error!("Error sending AFK state: {}", e);
        }
    }

    fn afk_start_time(&self, last_activity: DateTime<Utc>, afk_timeout_secs: u64) -> DateTime<Utc> {
        last_activity + chrono::Duration::seconds(afk_timeout_secs as i64)
    }

    pub fn subscribe_state(&self) -> watch::Receiver<AfkState> {
        self.afk_state_rx.clone()
    }
}

#[async_trait::async_trait]
impl TrackerLifecycle for AFKTracker {
    type StartArgs = (
        broadcast::Receiver<PowerEvent>,
        broadcast::Receiver<InputActivity>,
    );

    fn start_tracking(self: Arc<Self>, (power_rx, input_rx): Self::StartArgs) {
        AFKTracker::start_tracking(self, power_rx, input_rx);
    }

    async fn shutdown(&self) {
        self.stop_task().await;
    }

    async fn flush(&self) {
        self.flush_open_afk_event().await;
    }
}
