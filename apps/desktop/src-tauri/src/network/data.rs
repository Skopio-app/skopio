use common::models::{inputs::ProjectListQuery, outputs::PaginatedProjects, Project};
use db::models::{App, Category};
use uuid::Uuid;

use crate::network::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_apps() -> Result<Vec<App>, String> {
    req_json::<_, ()>("apps", None).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_categories() -> Result<Vec<Category>, String> {
    req_json::<_, ()>("categories", None).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_projects(query: ProjectListQuery) -> Result<PaginatedProjects, String> {
    req_json("projects", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_project(id: Uuid) -> Result<Option<Project>, String> {
    let path = format!("projects/{}", id);
    req_json::<Option<Project>, ()>(&path, None::<&()>).await
}
