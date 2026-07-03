use std::sync::Arc;

use async_trait::async_trait;
use common::models::inputs::{AFKEventInput, EventInput};
use db::DBContext;
use db::desktop::{afk_events::AFKEvent, events::Event};
use db::error::DBError;
use tokio::sync::{Mutex, mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant, interval};
use tracing::{error, info};

use crate::network::post_json;
use crate::tracking_service::TrackingService;

enum TrackingStats {
    Event(Box<Event>),
    Afk(Box<AFKEvent>),
}

pub struct BufferedTrackingService {
    sender: mpsc::Sender<TrackingStats>,
    flush_shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    flush_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    sync_shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    sync_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl BufferedTrackingService {
    pub fn new(
        inner: Arc<dyn TrackingService>,
        db: Arc<DBContext>,
        flush_interval_rx: watch::Receiver<u64>,
        sync_interval_rx: watch::Receiver<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TrackingStats>(100);

        let (flush_shutdown_tx, flush_shutdown_rx) = oneshot::channel::<()>();
        let flush_shutdown_tx = Arc::new(Mutex::new(Some(flush_shutdown_tx)));
        let (sync_shutdown_tx, sync_shutdown_rx) = oneshot::channel::<()>();
        let sync_shutdown_tx = Arc::new(Mutex::new(Some(sync_shutdown_tx)));

        let inner_clone = Arc::clone(&inner);
        let db_clone = Arc::clone(&db);

        let flush_handle = tokio::spawn(run_buffer_flush_loop(
            rx,
            flush_shutdown_rx,
            inner_clone,
            flush_interval_rx,
        ));

        let sync_handle = tokio::spawn(run_sync_loop(db_clone, sync_interval_rx, sync_shutdown_rx));

        Self {
            sender: tx,
            flush_shutdown_tx,
            flush_handle: Arc::new(Mutex::new(Some(flush_handle))),
            sync_shutdown_tx,
            sync_handle: Arc::new(Mutex::new(Some(sync_handle))),
        }
    }

    pub async fn shutdown(&self) {
        let mut tx_guard = self.flush_shutdown_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }
        drop(tx_guard);

        let mut tx_guard = self.sync_shutdown_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }
        drop(tx_guard);

        if let Some(handle) = self.flush_handle.lock().await.take()
            && let Err(err) = handle.await
        {
            error!("Flush loop task panicked or failed to join: {}", err);
        }

        if let Some(handle) = self.sync_handle.lock().await.take()
            && let Err(err) = handle.await
        {
            error!("Sync loop task panicked or failed to join: {}", err);
        }
    }
}

#[async_trait]
impl TrackingService for BufferedTrackingService {
    async fn insert_event(&self, event: &Event) -> Result<(), DBError> {
        self.sender
            .send(TrackingStats::Event(Box::new(event.clone())))
            .await
            .map_err(|err| {
                DBError::Internal(format!("buffered tracking service is shut down: {err}"))
            })?;
        Ok(())
    }

    async fn insert_afk(&self, afk: &AFKEvent) -> Result<(), DBError> {
        self.sender
            .send(TrackingStats::Afk(Box::new(afk.clone())))
            .await
            .map_err(|err| {
                DBError::Internal(format!("buffered tracking service is shut down: {err}"))
            })?;
        Ok(())
    }
}

async fn run_buffer_flush_loop(
    mut rx: mpsc::Receiver<TrackingStats>,
    mut shutdown_rx: oneshot::Receiver<()>,
    inner: Arc<dyn TrackingService>,
    mut flush_interval_rx: watch::Receiver<u64>,
) {
    let mut flush_interval = Duration::from_secs(*flush_interval_rx.borrow_and_update());
    let mut buffer: Vec<TrackingStats> = Vec::with_capacity(20);
    let mut retry_queue: Vec<TrackingStats> = Vec::with_capacity(20);
    let mut last_flush = Instant::now();

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                        buffer.push(msg);
                        if buffer.len() >= 10 || last_flush.elapsed() >= flush_interval {
                            let mut flush_data = buffer.split_off(0);
                            let mut retry_data = retry_queue.split_off(0);

                            flush(&inner, &mut flush_data, &mut retry_data).await;
                            retry_queue.append(&mut retry_data);

                            last_flush = Instant::now();
                        }
                    }
            _ = tokio::time::sleep_until(last_flush + flush_interval) => {
                if !buffer.is_empty() {
                   let mut flush_data = buffer.split_off(0);
                   let mut retry_data = retry_queue.split_off(0);

                    flush(&inner, &mut flush_data, &mut retry_data).await;
                    retry_queue.append(&mut retry_data);

                    last_flush = Instant::now();
                }
            }
            changed = flush_interval_rx.changed() => {
                if changed.is_ok() {
                    flush_interval = Duration::from_secs(*flush_interval_rx.borrow_and_update());
                }
            }
            _ = &mut shutdown_rx => {
                rx.close();
                while let Some(msg) = rx.recv().await {
                    buffer.push(msg);
                }

                let mut flush_data = buffer.split_off(0);
                let mut retry_data = retry_queue.split_off(0);
                let shutdown_item_count = flush_data.len() + retry_data.len();

                if shutdown_item_count > 0 {
                    info!("Flushing buffer before shutdown ({}) items...", shutdown_item_count);
                    flush(&inner, &mut flush_data, &mut retry_data).await;
                }

                if !retry_data.is_empty() {
                    error!(
                        "Buffer service shut down with {} item(s) still queued after retries.",
                        retry_data.len()
                    );
                }

                info!("Buffer service shut down gracefully.");
                break;
            }
        }
    }
}

async fn run_sync_loop(
    db: Arc<DBContext>,
    mut sync_interval_rx: watch::Receiver<u64>,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    let mut interval = interval(Duration::from_secs(*sync_interval_rx.borrow_and_update()));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = sync_with_server(&db).await {
                    error!("Sync with server failed: {}", e);
                }
            }
            changed = sync_interval_rx.changed() => {
                if changed.is_err() {
                    break;
                }

                interval =
                    tokio::time::interval(Duration::from_secs(*sync_interval_rx.borrow_and_update()));
            }
            _ = &mut shutdown_rx => {
                break;
            }
        }
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
