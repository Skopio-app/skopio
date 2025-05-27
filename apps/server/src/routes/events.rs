use crate::utils::error_response;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use common::models::inputs::EventInput;
use db::server::apps::App;
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::events::Event;
use db::server::languages::Language;
use db::server::projects::Project;
use db::DBContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

async fn handle_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<EventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    info!("Handling {} events", payload.len());

    for event in payload {
        let app_id = App::find_or_insert(&db, &event.app_name)
            .await
            .map_err(error_response)?;
        let project_id = Project::find_or_insert(&db, &event.project_name, &event.project_path)
            .await
            .map_err(error_response)?;
        let branch_id = Branch::find_or_insert(&db, project_id, &event.branch_name)
            .await
            .map_err(error_response)?;
        let entity_id =
            Entity::find_or_insert(&db, project_id, &event.entity_name, &event.entity_type)
                .await
                .map_err(error_response)?;
        let language_id = Language::find_or_insert(&db, &event.language_name)
            .await
            .map_err(error_response)?;

        let event = Event {
            id: None,
            timestamp: event.timestamp.unwrap_or_default(),
            duration: event.duration,
            activity_type: event.activity_type,
            app_id,
            entity_id: Some(entity_id),
            project_id: Some(project_id),
            branch_id: Some(branch_id),
            language_id: Some(language_id),
            end_timestamp: event.end_timestamp,
        };

        event.create(&db).await.map_err(error_response)?;
    }

    info!("Event details stored successfully");
    Ok(Json("Events recorded".to_string()))
}

pub fn event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/events", post(handle_events))
        .with_state(db)
}
