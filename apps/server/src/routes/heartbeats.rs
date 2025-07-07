use crate::utils::error_response;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use common::models::inputs::HeartbeatInput;
use db::models::App;
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::heartbeats::Heartbeat;
use db::server::languages::Language;
use db::server::projects::Project;
use db::DBContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// TODO: Investigate the need for optimizations in the for loop
async fn handle_heartbeats(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<HeartbeatInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    debug!("Handling {} heartbeats", payload.len());

    for hb in payload {
        let app_id = App::find_or_insert(&db, &hb.app_name)
            .await
            .map_err(error_response)?;
        let project_id = Project::find_or_insert(&db, &hb.project_name, &hb.project_path)
            .await
            .map_err(error_response)?;
        let branch_id = Branch::find_or_insert(&db, project_id, &hb.branch_name)
            .await
            .map_err(error_response)?;
        let entity_id = Entity::find_or_insert(&db, project_id, &hb.entity_name, &hb.entity_type)
            .await
            .map_err(error_response)?;
        let language_id = if let Some(lang) = &hb.language_name {
            Some(
                Language::find_or_insert(&db, lang)
                    .await
                    .map_err(error_response)?,
            )
        } else {
            None
        };

        let heartbeat = Heartbeat {
            id: None,
            project_id: Some(project_id),
            entity_id: Some(entity_id),
            branch_id: Some(branch_id),
            language_id,
            app_id: Some(app_id),
            timestamp: hb.timestamp.unwrap_or_default(),
            is_write: Some(hb.is_write),
            lines: hb.lines,
            cursorpos: hb.cursorpos,
        };

        heartbeat.create(&db).await.map_err(error_response)?;
    }

    Ok(Json("Heartbeats saved".to_string()))
}

pub fn heartbeat_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/heartbeats", post(handle_heartbeats))
        .with_state(db)
}
