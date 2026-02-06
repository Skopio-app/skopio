use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use common::models::{
    inputs::{AFKEventInput, BucketSummaryInput},
    outputs::FullEvent,
};
use db::{
    server::{afk_events::AFKEvent, summary::SummaryQueryBuilder},
    DBContext,
};
use serde_qs::axum::QsQuery;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::error::ServerResult;

async fn handle_afk_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<AFKEventInput>>,
) -> ServerResult<()> {
    info!("Handling {} afk events", payload.len());

    let mut events: Vec<AFKEvent> = payload
        .into_iter()
        .map(|afk| {
            let key = format!(
                "{}|{}|{}",
                afk.afk_start,
                afk.afk_end.unwrap_or_default(),
                afk.duration.unwrap_or_default(),
            );

            let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes());

            AFKEvent {
                id,
                afk_start: afk.afk_start,
                afk_end: afk.afk_end,
                duration: afk.duration,
            }
        })
        .collect();

    events.sort_by_key(|e| e.id);
    events.dedup_by_key(|e| e.id);

    let db = db.lock().await;
    let inserted = AFKEvent::bulk_create(&db, &events).await?;

    info!("Inserted {} AFK events", inserted);
    Ok(())
}

async fn fetch_afk_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<BucketSummaryInput>,
) -> ServerResult<Json<Vec<FullEvent>>> {
    let db = db.lock().await;
    let builder = SummaryQueryBuilder::from(payload);
    let events = builder.fetch_afk_event_range(&db).await?;

    Ok(Json(events))
}

pub fn afk_event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/afk", post(handle_afk_events))
        .route("/afk", get(fetch_afk_events))
        .with_state(db)
}
