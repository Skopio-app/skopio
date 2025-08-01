use chrono::{DateTime, Utc};
use db::desktop::afk_events::AFKEvent;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};
use tokio::time::{interval, Duration};

use crate::tracking_service::TrackingService;

use super::keyboard_tracker::KeyboardTracker;
use super::mouse_tracker::MouseTracker;

pub struct AFKTracker {
    last_activity: Arc<RwLock<DateTime<Utc>>>,
    afk_start: Arc<Mutex<Option<DateTime<Utc>>>>,
    afk_timeout_rx: watch::Receiver<u64>,
    cursor_tracker: Arc<MouseTracker>,
    keyboard_tracker: Arc<KeyboardTracker>,
    tracker: Arc<dyn TrackingService>,
}

impl AFKTracker {
    pub fn new(
        cursor_tracker: Arc<MouseTracker>,
        keyboard_tracker: Arc<KeyboardTracker>,
        afk_timeout_rx: watch::Receiver<u64>,
        tracker: Arc<dyn TrackingService>,
    ) -> Self {
        Self {
            last_activity: Arc::new(RwLock::new(Utc::now())),
            afk_start: Arc::new(Mutex::new(None)),
            afk_timeout_rx,
            cursor_tracker,
            keyboard_tracker,
            tracker,
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let last_activity = Arc::clone(&self.last_activity);
        let afk_start = Arc::clone(&self.afk_start);
        let cursor_tracker = Arc::clone(&self.cursor_tracker);
        let keyboard_tracker = Arc::clone(&self.keyboard_tracker);
        let buffer_tracker = Arc::clone(&self.tracker);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                let now = Utc::now();
                let last_activity_time = *last_activity.read().await;
                let mut afk_time = afk_start.lock().await;

                // Detect user activity (mouse/keyboard)
                let mouse_buttons = cursor_tracker.get_pressed_mouse_buttons();
                let keys_pressed = keyboard_tracker.get_pressed_keys();

                let mouse_active = cursor_tracker.has_mouse_moved();
                let mouse_clicked = mouse_buttons.left
                    || mouse_buttons.right
                    || mouse_buttons.middle
                    || mouse_buttons.other;
                let keyboard_active = !keys_pressed.is_empty();

                let activity_detected = mouse_active || mouse_clicked || keyboard_active;

                if activity_detected {
                    *last_activity.write().await = now;

                    if let Some(afk_start_time) = *afk_time {
                        let afk_duration = (now - afk_start_time).num_seconds();
                        info!(
                            "User returned at: {} (AFK Duration: {}s)",
                            now, afk_duration
                        );

                        let afk_event = AFKEvent {
                            id: None,
                            afk_start: Some(afk_start_time),
                            afk_end: Some(now),
                            duration: Some(afk_duration),
                        };

                        buffer_tracker
                            .insert_afk(afk_event)
                            .await
                            .unwrap_or_else(|error| error!("Failed to batch afk event: {}", error));
                    }
                    *afk_time = None;
                } else {
                    // Dynamically retrieve afk timeout from app settings config
                    let afk_threshold = Duration::from_secs(*self.afk_timeout_rx.borrow());
                    let idle_duration = (now - last_activity_time).num_seconds();
                    if idle_duration >= afk_threshold.as_secs() as i64 && afk_time.is_none() {
                        info!("User went AFK at: {}", now);
                        *afk_time = Some(now);
                    }
                }
            }
        });
    }

    pub async fn stop_tracking(&self) {
        let mut afk_time = self.afk_start.lock().await;

        if let Some(afk_start_time) = *afk_time {
            let now = Utc::now();
            let afk_duration = (now - afk_start_time).num_seconds();

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

            if let Err(err) = self.tracker.insert_afk(afk_event).await {
                error!("Failed to flush AFK event on stop: {}", err);
            }

            *afk_time = None;
        } else {
            info!("AFK tracker stopping. No AFK event to flush.");
        }
    }
}
