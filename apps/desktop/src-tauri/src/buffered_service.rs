use std::sync::Arc;

use async_trait::async_trait;
use db::desktop::{afk_events::AFKEvent, events::Event, heartbeats::Heartbeat};
use log::info;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{Duration, Instant};

use crate::tracking_service::TrackingService;

pub enum TrackingMessage {
    Heartbeat(Heartbeat),
    Event(Event),
    Afk(AFKEvent),
}

pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingMessage>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl BufferedTrackingService {
    pub fn new(inner: Arc<dyn TrackingService>) -> Self {
        let (tx, mut rx) = mpsc::channel::<TrackingMessage>(100);
        let flush_interval = Duration::from_secs(120);
        let mut buffer: Vec<TrackingMessage> = Vec::new();
        let mut last_flush = Instant::now();

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let shutdown_tx_arc = Arc::new(Mutex::new(Some(shutdown_tx)));

        let inner_clone = Arc::clone(&inner);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        buffer.push(msg);
                        if buffer.len() >= 10 || last_flush.elapsed() >= flush_interval {
                            flush(&inner_clone, &mut buffer).await;
                            last_flush = Instant::now();
                        }
                    }
                    _ = tokio::time::sleep_until(last_flush + flush_interval) => {
                        if !buffer.is_empty() {
                            flush(&inner_clone, &mut buffer).await;
                            last_flush = Instant::now();
                        }
                    }
                    _ = &mut shutdown_rx => {
                        if !buffer.is_empty() {
                            info!("Flushing buffer before shutdown ({} items)...", buffer.len());
                            flush(&inner_clone, &mut buffer).await;
                        }
                        info!("Buffer service shut down gracefully.");
                        break;
                    }
                }
            }
        });

        Self {
            sender: tx,
            shutdown_tx: shutdown_tx_arc,
        }
    }

    pub async fn shutdown(&self) {
        let mut tx_guard = self.shutdown_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }
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
