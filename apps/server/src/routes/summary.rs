use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use common::{
    models::inputs::{BucketedSummaryInput, SummaryQueryInput},
    time::TimeRange,
};
use db::{
    server::summary::{BucketTimeSummary, GroupedTimeSummary, SummaryQueryBuilder},
    DBContext,
};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub fn summary_query_from_input(input: SummaryQueryInput) -> SummaryQueryBuilder {
    let mut builder = SummaryQueryBuilder::default();

    if let Some(start) = input.start {
        builder = builder.start(start.naive_utc());
    }

    if let Some(end) = input.end {
        builder = builder.end(end.naive_utc());
    }

    if let Some(apps) = input.app_names {
        builder = builder.app_names(apps);
    }

    if let Some(projects) = input.project_names {
        builder = builder.project_names(projects);
    }

    if let Some(types) = input.activity_types {
        builder = builder.activity_types(types);
    }

    if let Some(entities) = input.entity_names {
        builder = builder.entity_names(entities);
    }

    if let Some(branches) = input.branch_names {
        builder = builder.branch_names(branches);
    }

    if let Some(langs) = input.language_names {
        builder = builder.language_names(langs);
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

pub async fn summary_by_apps(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_apps_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn summary_by_projects(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_projects_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn summary_by_entities(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_entities_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn summary_by_branches(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_branches_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn summary_by_activity_types(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<Vec<GroupedTimeSummary>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = summary_query_from_input(payload);
    builder
        .execute_activity_type_summary(&db)
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
        .time_bucket(range.bucket())
        .include_afk(payload.include_afk);

    let builder = if let Some(names) = payload.app_names {
        builder.app_names(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.project_names {
        builder.project_names(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.entity_names {
        builder.entity_names(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.activity_types {
        builder.activity_types(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.branch_names {
        builder.branch_names(names)
    } else {
        builder
    };

    let builder = if let Some(names) = payload.language_names {
        builder.language_names(names)
    } else {
        builder
    };

    let builder = if let Some(group_by) = &payload.group_by {
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
        .route("/summary/apps", post(summary_by_apps))
        .route("/summary/projects", post(summary_by_projects))
        .route("/summary/entities", post(summary_by_entities))
        .route("/summary/branches", post(summary_by_branches))
        .route("/summary/activity-types", post(summary_by_activity_types))
        .route("/summary/buckets", post(get_bucketed_summary))
        .route("/summary/range", post(execute_range_summary))
        .with_state(db)
}
