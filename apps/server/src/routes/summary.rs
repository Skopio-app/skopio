use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use common::models::inputs::{BucketSummaryInput, SummaryQueryInput};
use db::{
    models::{BucketTimeSummary, GroupedTimeSummary},
    server::summary::SummaryQueryBuilder,
    DBContext,
};
use serde_qs::axum::QsQuery;
use tokio::sync::Mutex;

use crate::error::ServerResult;

pub async fn total_time_handler(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<SummaryQueryInput>,
) -> ServerResult<Json<i64>> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = payload.into();
    let time = builder.execute_total_time(&db).await?;

    Ok(Json(time))
}

pub async fn execute_range_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<SummaryQueryInput>,
) -> ServerResult<Json<Vec<GroupedTimeSummary>>> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = payload.into();
    let range = builder.execute_range_summary(&db).await?;

    Ok(Json(range))
}

pub async fn get_bucketed_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<BucketSummaryInput>,
) -> ServerResult<Json<Vec<BucketTimeSummary>>> {
    let db = db.lock().await;
    let builder: SummaryQueryBuilder = payload.into();

    let records = builder.execute_range_summary_with_bucket(&db).await?;

    Ok(Json(records))
}

pub fn summary_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/summary/total", get(total_time_handler))
        .route("/summary/buckets", get(get_bucketed_summary))
        .route("/summary/range", get(execute_range_summary))
        .with_state(db)
}
