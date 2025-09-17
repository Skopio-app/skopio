use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use common::models::{
    inputs::{PaginationQuery, ProjectQuery, ProjectSearchQuery},
    outputs::PaginatedProjects,
    Project,
};
use db::{server::projects::ServerProject, DBContext};
use tokio::sync::Mutex;

use crate::error::ServerResult;

pub async fn get_projects(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(query): Query<PaginationQuery>,
) -> ServerResult<Json<PaginatedProjects>> {
    let db = db.lock().await;

    let limit = query.limit.unwrap_or(20).min(100);

    let projects = ServerProject::fetch_paginated(&db, query.after, limit).await?;

    let cursors = ServerProject::get_all_cursors(&db, limit).await?;

    Ok(Json(PaginatedProjects {
        data: projects,
        total_pages: Some(cursors.len() as u32),
        cursors,
    }))
}

pub async fn fetch_project(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(query): Query<ProjectQuery>,
) -> ServerResult<Json<Option<Project>>> {
    let db = db.lock().await;

    let project = ServerProject::find_by_id(&db, query.id).await?;

    Ok(Json(project))
}

pub async fn search_projects(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(query): Query<ProjectSearchQuery>,
) -> ServerResult<Json<Vec<Project>>> {
    let db = db.lock().await;

    let projects = ServerProject::search_project(&db, &query.name, query.limit).await?;

    Ok(Json(projects))
}

// TODO: Fix route naming
pub fn project_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/projects", get(get_projects))
        .route("/project", get(fetch_project))
        .route("/projects/search", get(search_projects))
        .with_state(db)
}
