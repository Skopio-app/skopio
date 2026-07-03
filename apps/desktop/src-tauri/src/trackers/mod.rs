pub mod afk_tracker;
pub mod event_tracker;
pub mod input_activity;
pub mod keyboard_tracker;
pub mod mouse_tracker;
pub mod power_monitor;
pub mod window_tracker;

use std::sync::Arc;

#[async_trait::async_trait]
pub trait TrackerLifecycle: Send + Sync {
    type StartArgs: Send + 'static;

    fn start_tracking(self: Arc<Self>, args: Self::StartArgs);

    async fn shutdown(&self);

    async fn flush(&self) {}
}

pub const SOURCE: &str = "skopio-desktop";
