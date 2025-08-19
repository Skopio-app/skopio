use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use common::{
    models::inputs::{BucketedSummaryInput, SummaryQueryInput},
    time::TimeRange,
};
use db::{
    models::{BucketTimeSummary, GroupedTimeSummary},
    server::summary::SummaryQueryBuilder,
    DBContext,
};
use serde_qs::axum::QsQuery;
use tokio::sync::Mutex;
use tracing::debug;

use crate::utils::error_response;

pub async fn total_time_handler(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<SummaryQueryInput>,
) -> Result<Json<i64>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = payload.into();
    builder
        .execute_total_time(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn execute_range_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = payload.into();
    builder
        .execute_range_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn get_bucketed_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<BucketedSummaryInput>,
) -> Result<Json<Vec<BucketTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    debug!("The payload: {:?}", payload);
    let range = TimeRange::from(payload.preset);

    let mut builder = SummaryQueryBuilder::default()
        .start(range.start())
        .end(range.end())
        .time_bucket(range.bucket().unwrap())
        .apps(payload.apps.unwrap_or_default())
        .projects(payload.projects.unwrap_or_default())
        .entities(payload.entities.unwrap_or_default())
        .categories(payload.categories.unwrap_or_default())
        .branches(payload.branches.unwrap_or_default())
        .languages(payload.languages.unwrap_or_default());

    if let Some(group) = payload.group_by {
        builder = builder.group_by(group);
    }
    debug!("The builder: {:?}", builder);

    let records = builder
        .execute_range_summary_with_bucket(&db)
        .await
        .map_err(error_response)?;

    Ok(Json(records))
}

pub fn summary_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/summary/total-time", get(total_time_handler))
        .route("/summary/buckets", get(get_bucketed_summary))
        .route("/summary/range", get(execute_range_summary))
        .with_state(db)
}
