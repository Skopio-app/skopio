use std::sync::Arc;

use async_trait::async_trait;
use db::desktop::{afk_events::AFKEvent, events::Event, heartbeats::Heartbeat};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

use crate::tracking_service::TrackingService;

pub enum TrackingMessage {
    Heartbeat(Heartbeat),
    Event(Event),
    Afk(AFKEvent),
}

#[derive(Clone)]
pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingMessage>,
}

impl BufferedTrackingService {
    pub fn new(inner: Arc<dyn TrackingService>) -> Self {
        let (tx, mut rx) = mpsc::channel::<TrackingMessage>(100);
        let flush_interval = Duration::from_secs(120);
        let mut buffer: Vec<TrackingMessage> = Vec::new();
        let mut last_flush = Instant::now();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        buffer.push(msg);
                        if buffer.len() >= 10 || last_flush.elapsed() >= flush_interval {
                            flush(&inner, &mut buffer).await;
                            last_flush = Instant::now();
                        }
                    }
                    _ = tokio::time::sleep_until(last_flush + flush_interval) => {
                        if !buffer.is_empty() {
                            flush(&inner, &mut buffer).await;
                            last_flush = Instant::now();
                        }
                    }
                }
            }
        });

        Self { sender: tx }
    }
}

#[async_trait]
impl TrackingService for BufferedTrackingService {
    async fn insert_heartbeat(&self, hb: Heartbeat) -> Result<(), anyhow::Error> {
        let _ = self.sender.send(TrackingMessage::Heartbeat(hb)).await;
        Ok(())
    }

    async fn insert_event(&self, event: Event) -> Result<(), anyhow::Error> {
        let _ = self.sender.send(TrackingMessage::Event(event)).await;
        Ok(())
    }

    async fn insert_afk(&self, afk: AFKEvent) -> Result<(), anyhow::Error> {
        let _ = self.sender.send(TrackingMessage::Afk(afk)).await;
        Ok(())
    }
}

async fn flush(inner: &Arc<dyn TrackingService>, buffer: &mut Vec<TrackingMessage>) {
    for msg in buffer.drain(..) {
        match msg {
            TrackingMessage::Heartbeat(hb) => {
                let _ = inner.insert_heartbeat(hb).await;
            }
            TrackingMessage::Event(ev) => {
                let _ = inner.insert_event(ev).await;
            }
            TrackingMessage::Afk(afk) => {
                let _ = inner.insert_afk(afk).await;
            }
        }
    }
}
