use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use common::{
    models::inputs::{BucketedSummaryInput, SummaryQueryInput},
    time::TimeRange,
};
use db::{
    models::{BucketTimeSummary, GroupedTimeSummary},
    server::summary::SummaryQueryBuilder,
    DBContext,
};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub fn summary_query_from_input(input: SummaryQueryInput) -> SummaryQueryBuilder {
    let mut builder = SummaryQueryBuilder::default();

    if let Some(start) = input.start {
        builder = builder.start(start);
    }

    if let Some(end) = input.end {
        builder = builder.end(end);
    }

    if let Some(apps) = input.apps {
        builder = builder.apps(apps);
    }

    if let Some(projects) = input.projects {
        builder = builder.projects(projects);
    }

    if let Some(types) = input.categories {
        builder = builder.categories(types);
    }

    if let Some(entities) = input.entities {
        builder = builder.entities(entities);
    }

    if let Some(branches) = input.branches {
        builder = builder.branches(branches);
    }

    if let Some(langs) = input.languages {
        builder = builder.languages(langs);
    }

    builder = builder.include_afk(input.include_afk);

    builder
}

pub async fn total_time_handler(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<i64>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_total_time(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn execute_range_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_range_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn get_bucketed_summary(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<BucketedSummaryInput>,
) -> Result<Json<Vec<BucketTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let range = TimeRange::from(payload.preset);

    let builder = SummaryQueryBuilder::default()
        .start(range.start())
        .end(range.end())
        .time_bucket(range.bucket().unwrap())
        .include_afk(payload.include_afk);

    let builder = if let Some(names) = payload.app_names {
        builder.apps(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.project_names {
        builder.projects(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.entity_names {
        builder.entities(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.activity_types {
        builder.categories(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.branch_names {
        builder.branches(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.language_names {
        builder.languages(names)
    } else {
        builder
    };

    let builder = if let Some(group_by) = payload.group_by {
        builder.group_by(group_by)
    } else {
        builder
    };

    let records = builder
        .execute_range_summary_with_bucket(&db)
        .await
        .map_err(error_response)?;

    Ok(Json(records))
}

pub fn summary_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/summary/total-time", post(total_time_handler))
        .route("/summary/buckets", post(get_bucketed_summary))
        .route("/summary/range", post(execute_range_summary))
        .with_state(db)
}
