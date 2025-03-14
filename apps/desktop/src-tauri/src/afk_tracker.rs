use chrono::{DateTime, Utc};
use log::info;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use crate::cursor_tracker::CursorTracker;
use crate::keyboard_tracker::KeyboardTracker;

pub struct AFKTracker {
    last_activity: Arc<RwLock<DateTime<Utc>>>,
    afk_start: Arc<Mutex<Option<DateTime<Utc>>>>,
    afk_threshold: Duration,
    last_cursor_position: Arc<Mutex<(f64, f64)>>,
    cursor_tracker: Arc<CursorTracker>,
    keyboard_tracker: Arc<KeyboardTracker>,
}

impl AFKTracker {
    pub fn new(cursor_tracker: Arc<CursorTracker>, keyboard_tracker: Arc<KeyboardTracker>) -> Self {
        Self {
            last_activity: Arc::new(RwLock::new(Utc::now())), // UTC-based last activity
            afk_start: Arc::new(Mutex::new(None)),
            afk_threshold: Duration::from_secs(60),
            last_cursor_position: Arc::new(Mutex::new(cursor_tracker.get_global_cursor_position())),
            cursor_tracker,
            keyboard_tracker,
        }
    }

    /// Other trackers call this to reset the AFK timer
    pub fn report_activity(&self) {
        let now = Utc::now();
        *self.last_activity.write().unwrap() = now;

        // If user was AFK, log the return event
        let mut afk_time = self.afk_start.lock().unwrap();
        if afk_time.is_some() {
            let duration = (now - afk_time.unwrap()).num_seconds();
            info!("User returned at: {} (AFK Duration: {}s)", now, duration);
            *afk_time = None;
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let last_activity = Arc::clone(&self.last_activity);
        let afk_start = Arc::clone(&self.afk_start);
        let afk_threshold = self.afk_threshold;
        let last_cursor_position = Arc::clone(&self.last_cursor_position);
        let cursor_tracker = Arc::clone(&self.cursor_tracker);
        let keyboard_tracker = Arc::clone(&self.keyboard_tracker);

        thread::spawn(move || {
            loop {
                let now = Utc::now();
                let last_activity_time = *last_activity.read().unwrap();
                let mut afk_time = afk_start.lock().unwrap();
                let mut last_cursor = last_cursor_position.lock().unwrap();

                // Detect user activity (mouse/keyboard)
                let cursor_position = cursor_tracker.get_global_cursor_position();
                let mouse_buttons = cursor_tracker.get_pressed_mouse_buttons();
                let keys_pressed = keyboard_tracker.get_pressed_keys();

                let dx = (cursor_position.0 - last_cursor.0).abs();
                let dy = (cursor_position.1 - last_cursor.1).abs();
                let movement_threshold = 2.0; // Ignore movemnet smaller than 2px.

                let mouse_active = dx > movement_threshold || dy > movement_threshold;
                let mouse_clicked = mouse_buttons.left
                    || mouse_buttons.right
                    || mouse_buttons.middle
                    || mouse_buttons.other;
                let keyboard_active = !keys_pressed.is_empty();

                let activity_detected = mouse_active || mouse_clicked || keyboard_active;

                info!(
                    "Activity Check - Mouse: {:?}, Clicked: {}, Keys Pressed: {:?}",
                    cursor_position, mouse_clicked, keys_pressed
                );

                if activity_detected {
                    *last_activity.write().unwrap() = now;
                    *afk_time = None;

                    last_cursor.0 = cursor_position.0;
                    last_cursor.1 = cursor_position.1;
                } else {
                    let idle_duration = (now - last_activity_time).num_seconds();
                    if idle_duration >= afk_threshold.as_secs() as i64 && afk_time.is_none() {
                        info!("User went AFK at: {}", now);
                        *afk_time = Some(now);
                    }
                }

                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}
