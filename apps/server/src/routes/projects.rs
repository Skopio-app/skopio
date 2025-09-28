use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use common::models::{inputs::ProjectListQuery, outputs::PaginatedProjects, Project};
use db::{server::projects::ServerProject, DBContext};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::ServerResult;

pub async fn get_projects(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(query): Query<ProjectListQuery>,
) -> ServerResult<Json<PaginatedProjects>> {
    let db = db.lock().await;
    let limit = query.limit.unwrap_or(20).min(100);

    if let Some(term) = query.query.as_deref() {
        let data = ServerProject::search_project(&db, term, limit).await?;
        return Ok(Json(PaginatedProjects {
            data,
            total_pages: None,
            cursors: vec![],
        }));
    }

    let data = ServerProject::fetch_paginated(&db, query.after, limit).await?;
    let cursors = ServerProject::get_all_cursors(&db, limit).await?;

    Ok(Json(PaginatedProjects {
        data,
        total_pages: Some(cursors.len() as u32),
        cursors,
    }))
}

pub async fn get_project_by_id(
    State(db): State<Arc<Mutex<DBContext>>>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<Option<Project>>> {
    let db = db.lock().await;
    let project = ServerProject::find_by_id(&db, id).await?;
    Ok(Json(project))
}

pub fn project_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/projects", get(get_projects))
        .route("/projects/{id}", get(get_project_by_id))
        .with_state(db)
}
