use common::models::{
    inputs::{PaginationQuery, ProjectQuery, ProjectSearchQuery},
    outputs::PaginatedProjects,
};
use db::models::{App, Category};
use types::Project;

use crate::network::utils::req_json;

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
pub async fn fetch_projects(query: PaginationQuery) -> Result<PaginatedProjects, String> {
    req_json("projects", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_project(query: ProjectQuery) -> Result<Option<Project>, String> {
    req_json("project", Some(&query)).await
}

#[tauri::command]
#[specta::specta]
pub async fn search_projects(query: ProjectSearchQuery) -> Result<Vec<Project>, String> {
    req_json("projects/search", Some(&query)).await
}
