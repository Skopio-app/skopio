use std::sync::Arc;

use async_trait::async_trait;
use common::models::inputs::{AFKEventInput, EventInput};
use db::desktop::{afk_events::AFKEvent, events::Event};
use db::DBContext;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration, Instant};
use tracing::{error, info};

use crate::network::post_json;
use crate::tracking_service::TrackingService;

enum TrackingStats {
    Event(Box<Event>),
    Afk(Box<AFKEvent>),
}

pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingStats>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    flush_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
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

        let flush_handle = tokio::spawn(run_buffer_flush_loop(
            rx,
            shutdown_rx,
            inner_clone,
            flush_interval_rx,
        ));

        tokio::spawn(run_sync_loop(db_clone, sync_interval_rx));

        Self {
            sender: tx,
            shutdown_tx: shutdown_tx_arc,
            flush_handle: Arc::new(Mutex::new(Some(flush_handle))),
        }
    }

    pub async fn shutdown(&self) {
        let mut tx_guard = self.shutdown_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.flush_handle.lock().await.take() {
            if let Err(err) = handle.await {
                error!("Flush loop task panicked or failed to join: {}", err);
            }
        }
    }
}

#[async_trait]
impl TrackingService for BufferedTrackingService {
    async fn insert_event(&self, event: &Event) -> Result<(), anyhow::Error> {
        let _ = self
            .sender
            .send(TrackingStats::Event(Box::new(event.clone())))
            .await;
        Ok(())
    }

    async fn insert_afk(&self, afk: &AFKEvent) -> Result<(), anyhow::Error> {
        let _ = self
            .sender
            .send(TrackingStats::Afk(Box::new(afk.clone())))
            .await;
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

                    info!("Flushing buffer before shutdown ({}) items...", flush_data.len());
                    flush(&inner, &mut flush_data, &mut retry_data).await;
                    info!("Buffer service shut down gracefully.");
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
                TrackingStats::Event(ev) => inner.insert_event(ev).await,
                TrackingStats::Afk(afk) => inner.insert_afk(afk).await,
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
                branch_name: ev.branch_name.clone(),
                language_name: ev.language_name.clone(),
                source_name: ev.source_name.clone(),
                end_timestamp: ev.end_timestamp,
            })
            .collect();

        match post_json::<Vec<EventInput>, ()>("/events", &payload).await {
            Ok(_) => {
                Event::mark_as_synced(db_context, &events).await?;
                info!("Synced {} events", events.len());
                Event::delete_synced(db_context).await?;
            }
            Err(e) => {
                error!("Something went wrong trying to sync events: {e}");
            }
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

        match post_json::<Vec<AFKEventInput>, ()>("/afk", &payload).await {
            Ok(_) => {
                AFKEvent::mark_as_synced(db_context, &afk_events).await?;
                info!("Synced {} afk events", afk_events.len());
                AFKEvent::delete_synced(db_context).await?;
            }
            Err(e) => {
                error!("Something went wrong trying to sync AFK events: {e}")
            }
        }
    }

    Ok(())
}
