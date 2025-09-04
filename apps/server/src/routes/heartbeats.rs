use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use common::models::inputs::HeartbeatInput;
use db::models::{App, Source};
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::heartbeats::Heartbeat;
use db::server::languages::Language;
use db::server::projects::ServerProject;
use db::DBContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::error::AppResult;

// TODO: Investigate the need for optimizations in the for loop
async fn handle_heartbeats(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<HeartbeatInput>>,
) -> AppResult<Json<String>> {
    let db = db.lock().await;

    debug!("Handling {} heartbeats", payload.len());

    for hb in payload {
        let app_id = App::find_or_insert(&db, &hb.app_name).await?;
        let project_id =
            ServerProject::find_or_insert(&db, &hb.project_name, &hb.project_path).await?;
        let branch_id = Branch::find_or_insert(&db, project_id, &hb.branch_name).await?;
        let entity_id =
            Entity::find_or_insert(&db, project_id, &hb.entity_name, &hb.entity_type).await?;
        let language_id = Language::find_or_insert(&db, &hb.language_name).await?;

        let source_id = Source::find_or_insert(&db, &hb.source_name).await?;

        let id = uuid::Uuid::now_v7();
        let heartbeat = Heartbeat {
            id: id,
            project_id: Some(project_id),
            entity_id: Some(entity_id),
            branch_id,
            language_id,
            app_id: Some(app_id),
            source_id,
            timestamp: hb.timestamp.unwrap_or_default(),
            is_write: Some(hb.is_write),
            lines: hb.lines,
            cursorpos: hb.cursorpos,
        };

        heartbeat.create(&db).await?;
    }

    Ok(Json("Heartbeats saved".to_string()))
}

pub fn heartbeat_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/heartbeats", post(handle_heartbeats))
        .with_state(db)
}
