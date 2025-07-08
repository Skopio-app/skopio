use std::sync::Arc;

use async_trait::async_trait;
use common::models::inputs::{AFKEventInput, EventInput, HeartbeatInput};
use db::desktop::{afk_events::AFKEvent, events::Event, heartbeats::Heartbeat};
use db::DBContext;
use log::{error, info};
use reqwest::Client;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::time::{interval, Duration, Instant};

use crate::tracking_service::TrackingService;

const SERVER_URL: &str = "http://localhost:8080";

enum TrackingStats {
    Heartbeat(Heartbeat),
    Event(Event),
    Afk(AFKEvent),
}

pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingStats>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl BufferedTrackingService {
    pub fn new(
        inner: Arc<dyn TrackingService>,
        db: Arc<DBContext>,
        flush_interval_rx: watch::Receiver<u64>,
        sync_interval_rx: watch::Receiver<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TrackingStats>(100);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let shutdown_tx_arc = Arc::new(Mutex::new(Some(shutdown_tx)));

        let inner_clone = Arc::clone(&inner);
        let db_clone = Arc::clone(&db);

        tokio::spawn(run_buffer_flush_loop(
            rx,
            shutdown_rx,
            inner_clone,
            flush_interval_rx,
        ));

        tokio::spawn(run_sync_loop(db_clone, sync_interval_rx));

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
        let _ = self.sender.send(TrackingStats::Heartbeat(hb)).await;
        Ok(())
    }

    async fn insert_event(&self, event: Event) -> Result<(), anyhow::Error> {
        let _ = self.sender.send(TrackingStats::Event(event)).await;
        Ok(())
    }

    async fn insert_afk(&self, afk: AFKEvent) -> Result<(), anyhow::Error> {
        let _ = self.sender.send(TrackingStats::Afk(afk)).await;
        Ok(())
    }
}

async fn run_buffer_flush_loop(
    mut rx: mpsc::Receiver<TrackingStats>,
    mut shutdown_rx: oneshot::Receiver<()>,
    inner: Arc<dyn TrackingService>,
    mut flush_interval_rx: watch::Receiver<u64>,
) {
    let flush_interval = Duration::from_secs(*flush_interval_rx.borrow_and_update());
    let mut buffer: Vec<TrackingStats> = Vec::with_capacity(20);
    let mut retry_queue: Vec<TrackingStats> = Vec::with_capacity(20);
    let mut last_flush = Instant::now();

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                        buffer.push(msg);
                        if buffer.len() >= 10 || last_flush.elapsed() >= flush_interval {
                            let inner = Arc::clone(&inner);
                            let mut flush_data = buffer.split_off(0);
                            let mut retry_data = retry_queue.split_off(0);

                            tokio::spawn(async move {
                                flush(&inner, &mut flush_data, &mut retry_data).await;
                            });

                            last_flush = Instant::now();
                        }
                    }
            _ = tokio::time::sleep_until(last_flush + flush_interval) => {
                if !buffer.is_empty() {
                   let inner = Arc::clone(&inner);
                   let mut flush_data = buffer.split_off(0);
                   let mut retry_data = retry_queue.split_off(0);

                    tokio::spawn(async move {
                        flush(&inner, &mut flush_data, &mut retry_data).await;
                    });

                    last_flush = Instant::now();
                }
            }
            _ = &mut shutdown_rx => {
                if !buffer.is_empty() {
                    let inner = Arc::clone(&inner);
                    let mut flush_data = buffer.split_off(0);
                    let mut retry_data = retry_queue.split_off(0);

                    tokio::spawn(async move {
                        info!("Flushing buffer before shutdown ({}) items...", flush_data.len());
                        flush(&inner, &mut flush_data, &mut retry_data).await;
                        info!("Buffer service shut down gracefully.");
                    });
                }
                break;
            }
        }
    }
}

async fn run_sync_loop(db: Arc<DBContext>, mut sync_interval_rx: watch::Receiver<u64>) {
    let mut interval = interval(Duration::from_secs(*sync_interval_rx.borrow_and_update()));
    loop {
        interval.tick().await;
        let db_clone = Arc::clone(&db);
        tokio::spawn(async move {
            if let Err(e) = sync_with_server(&db_clone).await {
                error!("Sync with server failed: {}", e);
            }
        });
    }
}

async fn flush(
    inner: &Arc<dyn TrackingService>,
    buffer: &mut Vec<TrackingStats>,
    retry_queue: &mut Vec<TrackingStats>,
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
                TrackingStats::Heartbeat(hb) => inner.insert_heartbeat(hb.clone()).await,
                TrackingStats::Event(ev) => inner.insert_event(ev.clone()).await,
                TrackingStats::Afk(afk) => inner.insert_afk(afk.clone()).await,
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
                timestamp: hb.timestamp,
                project_name: hb.project_name.clone().unwrap_or_default(),
                project_path: hb.project_path.clone().unwrap_or_default(),
                entity_name: hb.entity_name.clone(),
                entity_type: hb.entity_type.clone(),
                branch_name: hb.branch_name.clone(),
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
                timestamp: ev.timestamp,
                duration: ev.duration,
                category: ev.category.clone().unwrap_or_default(),
                app_name: ev.app_name.clone(),
                entity_name: ev.entity_name.clone().unwrap_or_default(),
                entity_type: ev.entity_type.clone().unwrap_or_default(),
                project_name: ev.project_name.clone().unwrap_or_default(),
                project_path: ev.project_path.clone().unwrap_or_default(),
                branch_name: Some(ev.branch_name.clone().unwrap_or_default()),
                language_name: Some(ev.language_name.clone().unwrap_or_default()),
                end_timestamp: ev.end_timestamp,
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

    let afk_events = AFKEvent::unsynced(db_context).await?;
    if !afk_events.is_empty() {
        let payload: Vec<AFKEventInput> = afk_events
            .iter()
            .map(|afk| AFKEventInput {
                afk_start: afk.afk_start.unwrap_or_default(),
                afk_end: afk.afk_end,
                duration: afk.duration,
            })
            .collect();

        let res = client
            .post(format!("{}/afk", SERVER_URL))
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            AFKEvent::mark_as_synced(db_context, &afk_events).await?;
            info!("Synced {} afk events", afk_events.len());
        } else {
            error!(
                "Something went wrong trying to sync AFK events: {:?}",
                res.text().await
            )
        }
    }

    Ok(())
}
