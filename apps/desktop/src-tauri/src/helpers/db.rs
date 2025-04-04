use std::path::PathBuf;

use db::{
    apps::App, branches::Branch, entities::Entity, languages::Language, projects::Project,
    DBContext,
};
use log::debug;
use tauri::{AppHandle, Manager, Runtime};

use crate::heartbeat_tracker::Heartbeat;

const DB_NAME: &str = "skopio.db";

pub fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(DB_NAME)
}

#[derive(Default, Debug)]
pub struct ResolvedHeartbeatIDs {
    pub app_id: Option<i64>,
    pub project_id: Option<i64>,
    pub entity_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub language_id: Option<i64>,
}

pub async fn resolve_heartbeat_ids(
    db: &DBContext,
    heartbeat: Heartbeat,
) -> Result<ResolvedHeartbeatIDs, anyhow::Error> {
    debug!("The received heartbeat: {:?}", heartbeat);

    let app_id = App::find_or_insert(db, heartbeat.app_name.as_str())
        .await
        .ok();
    let project_id = match (heartbeat.project_name, heartbeat.project_path) {
        (Some(name), Some(path)) => Project::find_or_insert(db, &name, &path).await.ok(),
        _ => None,
    };
    let entity_id = match project_id {
        Some(pid) => Entity::find_or_insert(
            db,
            pid,
            &heartbeat.entity_name,
            &heartbeat.entity_type.to_string(),
        )
        .await
        .ok(),
        None => None,
    };

    let branch_id = match (project_id, heartbeat.branch_name) {
        (Some(pid), Some(branch)) => Branch::find_or_insert(db, pid, &branch).await.ok(),
        _ => None,
    };

    let language_id = match heartbeat.language_name {
        Some(lang) => Language::find_or_insert(db, &lang).await.ok(),
        None => None,
    };

    let resolved = ResolvedHeartbeatIDs {
        app_id,
        project_id,
        entity_id,
        branch_id,
        language_id,
    };

    debug!("The resolved heartbeat IDs: {:?}", resolved);

    Ok(resolved)
}
