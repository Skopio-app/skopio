use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use db::desktop::{afk_events::AFKEvent, events::Event, heartbeats::Heartbeat};
use db::DBContext;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{interval, Duration, Instant};

use crate::tracking_service::TrackingService;

const SERVER_URL: &str = "http://localhost:8080";

pub enum TrackingMessage {
    Heartbeat(Heartbeat),
    Event(Event),
    Afk(AFKEvent),
}

// TODO: Add structs to `core` crate for ease of reuse across desktop, server and CLI apps
#[derive(Serialize, Deserialize, Debug)]
struct EventInput {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    duration: Option<i64>,
    activity_type: String,
    app_name: String,
    entity_name: String,
    entity_type: String,
    project_name: String,
    project_path: String,
    branch_name: String,
    language_name: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    end_timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HeartbeatInput {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    project_name: String,
    project_path: String,
    entity_name: String,
    entity_type: String,
    branch_name: String,
    language_name: Option<String>,
    app_name: String,
    is_write: bool,
    lines: Option<i64>,
    cursorpos: Option<i64>,
}

pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingMessage>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl BufferedTrackingService {
    pub fn new(inner: Arc<dyn TrackingService>, db: Arc<DBContext>) -> Self {
        let (tx, mut rx) = mpsc::channel::<TrackingMessage>(100);
        let flush_interval = Duration::from_secs(120);
        let mut buffer: Vec<TrackingMessage> = Vec::new();
        let mut retry_queue: Vec<TrackingMessage> = Vec::new();
        let mut last_flush = Instant::now();

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let shutdown_tx_arc = Arc::new(Mutex::new(Some(shutdown_tx)));

        let inner_clone = Arc::clone(&inner);
        let db_clone = Arc::clone(&db);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        buffer.push(msg);
                        if buffer.len() >= 10 || last_flush.elapsed() >= flush_interval {
                            flush(&inner_clone, &mut buffer, &mut retry_queue).await;
                            last_flush = Instant::now();
                        }
                    }
                    _ = tokio::time::sleep_until(last_flush + flush_interval) => {
                        if !buffer.is_empty() {
                            flush(&inner_clone, &mut buffer, &mut retry_queue).await;
                            last_flush = Instant::now();
                        }
                    }
                    _ = &mut shutdown_rx => {
                        if !buffer.is_empty() {
                            info!("Flushing buffer before shutdown ({} items)...", buffer.len());
                            flush(&inner_clone, &mut buffer, &mut retry_queue).await;
                        }
                        info!("Buffer service shut down gracefully.");
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = sync_with_server(&db_clone).await {
                    error!("Sync with server failed: {}", e);
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

async fn flush(
    inner: &Arc<dyn TrackingService>,
    buffer: &mut Vec<TrackingMessage>,
    retry_queue: &mut Vec<TrackingMessage>,
) {
    let start = Instant::now();
    let mut combined = Vec::new();
    combined.append(retry_queue);
    combined.append(buffer);

    let batch_size = combined.len();
    for msg in combined.drain(..) {
        let mut attempts = 0;

        let result = loop {
            let res = match &msg {
                TrackingMessage::Heartbeat(hb) => inner.insert_heartbeat(hb.clone()).await,
                TrackingMessage::Event(ev) => inner.insert_event(ev.clone()).await,
                TrackingMessage::Afk(afk) => inner.insert_afk(afk.clone()).await,
            };

            match res {
                Ok(_) => break Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        break Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(200 * attempts)).await;
                }
            }
        };

        if let Err(e) = result {
            error!("Insert failed after retries: {}", e);
            retry_queue.push(msg);
        }
    }

    info!("Flushed {} items in {:?}", batch_size, start.elapsed())
}

async fn sync_with_server(db_context: &Arc<DBContext>) -> Result<(), anyhow::Error> {
    let client = Client::new();
    let heartbeats = Heartbeat::unsynced(db_context).await?;
    if !heartbeats.is_empty() {
        let payload: Vec<HeartbeatInput> = heartbeats
            .iter()
            .map(|hb| HeartbeatInput {
                timestamp: hb
                    .timestamp
                    .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
                project_name: hb.project_name.clone().unwrap_or_default(),
                project_path: hb.project_path.clone().unwrap_or_default(),
                entity_name: hb.entity_name.clone(),
                entity_type: hb.entity_type.clone(),
                branch_name: hb.entity_name.clone(),
                language_name: hb.language_name.clone(),
                app_name: hb.app_name.clone(),
                is_write: hb.is_write.unwrap_or_default(),
                lines: hb.lines,
                cursorpos: hb.cursorpos,
            })
            .collect();

        let res = client
            .post(format!("{}/heartbeats", SERVER_URL))
            .json(&payload)
            .send()
            .await?;
        if res.status().is_success() {
            Heartbeat::mark_as_synced(db_context, &heartbeats).await?;
            info!("Synced {} heartbeats", heartbeats.len());
        } else {
            error!(
                "Something went wrong trying to sync heartbeats: {:?}",
                res.text().await
            );
        }
    }

    let events = Event::unsynced(db_context).await?;
    if !events.is_empty() {
        let payload: Vec<EventInput> = events
            .iter()
            .map(|ev| EventInput {
                timestamp: ev
                    .timestamp
                    .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
                duration: ev.duration,
                activity_type: ev.activity_type.clone().unwrap_or_default(),
                app_name: ev.app_name.clone(),
                entity_name: ev.entity_name.clone().unwrap_or_default(),
                entity_type: ev.entity_type.clone().unwrap_or_default(),
                project_name: ev.project_name.clone().unwrap_or_default(),
                project_path: ev.project_path.clone().unwrap_or_default(),
                branch_name: ev.branch_name.clone().unwrap_or_default(),
                language_name: ev.language_name.clone().unwrap_or_default(),
                end_timestamp: ev
                    .end_timestamp
                    .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
            })
            .collect();

        let res = client
            .post(format!("{}/events", SERVER_URL))
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            Event::mark_as_synced(db_context, &events).await?;
            info!("Synced {} events", events.len());
        } else {
            error!(
                "Something went wrong trying to sync events: {:?}",
                res.text().await
            );
        }
    }

    Ok(())
}
