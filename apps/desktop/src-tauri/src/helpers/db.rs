use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};
use db::{
    apps::App, branches::Branch, entities::Entity, languages::Language, projects::Project,
    DBContext,
};

use tauri::{AppHandle, Manager, Runtime};

use crate::{event_tracker::Event, heartbeat_tracker::Heartbeat};

const DB_NAME: &str = "skopio.db";

pub fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(DB_NAME)
}

pub fn to_naive_datetime(datetime: Option<DateTime<Utc>>) -> Option<NaiveDateTime> {
    datetime.map(|dt| dt.naive_utc())
}

#[derive(Default, Debug)]
pub struct ResolvedHeartbeatIDs {
    pub app_id: Option<i64>,
    pub project_id: Option<i64>,
    pub entity_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub language_id: Option<i64>,
}

#[derive(Default, Debug)]
pub struct ResolvedEventIDs {
    pub app_id: Option<i64>,
    pub project_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub entity_id: Option<i64>,
    pub language_id: Option<i64>,
}

pub async fn resolve_heartbeat_ids(
    db: &DBContext,
    heartbeat: &Heartbeat,
) -> Result<ResolvedHeartbeatIDs, anyhow::Error> {
    let app_id = App::find_or_insert(db, heartbeat.app_name.as_str())
        .await
        .ok();
    let project_id = match (&heartbeat.project_name, &heartbeat.project_path) {
        (Some(name), Some(path)) => Project::find_or_insert(db, name, path).await.ok(),
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

    let branch_id = match (project_id, &heartbeat.branch_name) {
        (Some(pid), Some(branch)) => Branch::find_or_insert(db, pid, branch).await.ok(),
        _ => None,
    };

    let language_id = match &heartbeat.language_name {
        Some(lang) => Language::find_or_insert(db, lang).await.ok(),
        None => None,
    };

    Ok(ResolvedHeartbeatIDs {
        app_id,
        project_id,
        entity_id,
        branch_id,
        language_id,
    })
}

pub async fn resolve_event_ids(
    db: &DBContext,
    event: Event,
) -> Result<ResolvedEventIDs, anyhow::Error> {
    let app_id = App::find_or_insert(db, &event.app_name).await.ok();

    let project_id = match (&event.project_name, &event.project_path) {
        (Some(name), Some(path)) => Project::find_or_insert(db, name, path).await.ok(),
        _ => None,
    };

    let entity_id = match (&project_id, &event.entity_name, &event.entity_type) {
        (Some(pid), Some(name), Some(entity_type)) => {
            Entity::find_or_insert(db, *pid, name, &entity_type.to_string())
                .await
                .ok()
        }
        _ => None,
    };

    let branch_id = match (&project_id, &event.branch_name) {
        (Some(pid), Some(branch)) => Branch::find_or_insert(db, *pid, branch).await.ok(),
        _ => None,
    };

    let language_id = match &event.language_name {
        Some(lang) => Language::find_or_insert(db, lang).await.ok(),
        _ => None,
    };

    Ok(ResolvedEventIDs {
        app_id,
        project_id,
        branch_id,
        entity_id,
        language_id,
    })
}
