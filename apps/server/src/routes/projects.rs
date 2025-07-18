use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use common::models::{inputs::PaginationQuery, outputs::PaginatedProjects};
use db::{server::projects::ServerProject, DBContext};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub async fn get_projects(
    State(db): State<Arc<Mutex<DBContext>>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedProjects>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let limit = query.limit.unwrap_or(20).min(100);

    let projects = ServerProject::fetch_paginated(&db, query.after, limit)
        .await
        .map_err(error_response)?;

    let cursors = ServerProject::get_all_cursors(&db, limit)
        .await
        .map_err(error_response)?;

    Ok(Json(PaginatedProjects {
        data: projects,
        total_pages: Some(cursors.len() as u32),
        cursors,
    }))
}

pub fn project_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/projects", get(get_projects))
        .with_state(db)
}
