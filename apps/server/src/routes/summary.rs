use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::NaiveDateTime;
use db::{
    server::summary::{GroupedTimeSummary, SummaryQueryBuilder},
    DBContext,
};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::utils::error_response;

#[derive(Debug, Deserialize)]
pub struct SummaryQueryInput {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub app_names: Option<Vec<String>>,
    pub project_names: Option<Vec<String>>,
    pub activity_types: Option<Vec<String>>,
    pub entity_names: Option<Vec<String>>,
    pub branch_names: Option<Vec<String>>,
    pub language_names: Option<Vec<String>>,
    pub include_afk: bool,
}

impl From<SummaryQueryInput> for SummaryQueryBuilder {
    fn from(input: SummaryQueryInput) -> Self {
        let mut builder = SummaryQueryBuilder::default();

        if let Some(start) = input.start {
            builder = builder.start(start);
        }

        if let Some(end) = input.end {
            builder = builder.end(end);
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
}

pub async fn total_time_handler(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<SummaryQueryInput>,
) -> Result<Json<i64>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let builder: SummaryQueryBuilder = payload.into();
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

    let builder: SummaryQueryBuilder = payload.into();
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

    let builder: SummaryQueryBuilder = payload.into();
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

    let builder: SummaryQueryBuilder = payload.into();
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

    let builder: SummaryQueryBuilder = payload.into();
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

    let builder: SummaryQueryBuilder = payload.into();
    builder
        .execute_activity_type_summary(&db)
        .await
        .map(Json)
        .map_err(error_response)
}

pub fn summary_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/summary/total-time", post(total_time_handler))
        .route("/summary/apps", post(summary_by_apps))
        .route("/summary/projects", post(summary_by_projects))
        .route("/summary/entities", post(summary_by_entities))
        .route("/summary/branches", post(summary_by_branches))
        .route("/summary/activity-types", post(summary_by_activity_types))
        .with_state(db)
}
