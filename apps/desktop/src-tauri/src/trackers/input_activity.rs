use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputActivityKind {
    MouseMoved,
    MouseButton,
    KeyPressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputActivity {
    pub kind: InputActivityKind,
    pub at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct InputActivityBus {
    tx: broadcast::Sender<InputActivity>,
}

impl InputActivityBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InputActivity> {
        self.tx.subscribe()
    }

    pub fn publish(&self, kind: InputActivityKind) {
        let _ = self.tx.send(InputActivity {
            kind,
            at: Utc::now(),
        });
    }
}

impl Default for InputActivityBus {
    fn default() -> Self {
        Self::new()
    }
}
