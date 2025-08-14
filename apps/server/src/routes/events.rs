use crate::utils::error_response;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use common::models::inputs::{BucketedSummaryInput, EventInput};
use common::models::outputs::EventGroupResult;
use common::time::TimeRange;
use db::models::{App, Category, Source};
use db::server::branches::Branch;
use db::server::entities::Entity;
use db::server::events::Event;
use db::server::languages::Language;
use db::server::projects::ServerProject;
use db::server::summary::SummaryQueryBuilder;
use db::DBContext;
use serde_qs::axum::QsQuery;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

async fn insert_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<EventInput>>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    debug!("Handling {} events", payload.len());

    for event in payload {
        info!(
            "The branch name: {:?}. The language name: {:?}",
            event.branch_name, event.language_name
        );
        let app_id = App::find_or_insert(&db, &event.app_name)
            .await
            .map_err(error_response)?;
        let project_id =
            ServerProject::find_or_insert(&db, &event.project_name, &event.project_path)
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
        let category_id = Category::find_or_insert(&db, &event.category)
            .await
            .map_err(error_response)?;
        let source_id = Source::find_or_insert(&db, &event.source_name)
            .await
            .map_err(error_response)?;

        info!(
            "The branch id: {:?}. The language id: {:?}",
            branch_id, language_id
        );

        let id = uuid::Uuid::now_v7();

        let event = Event {
            id: id,
            timestamp: event.timestamp.unwrap_or_default(),
            duration: event.duration,
            category_id,
            app_id,
            entity_id: Some(entity_id),
            project_id: Some(project_id),
            branch_id,
            language_id,
            source_id,
            end_timestamp: event.end_timestamp,
        };

        event.create(&db).await.map_err(error_response)?;
    }
    Ok(Json("Events saved".to_string()))
}

async fn fetch_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<BucketedSummaryInput>,
) -> Result<Json<EventGroupResult>, (StatusCode, Json<String>)> {
    info!("The payload: {:?}", payload);

    let time_range = TimeRange::from(payload.preset);

    let mut builder = SummaryQueryBuilder::new()
        .start(time_range.start())
        .end(time_range.end())
        .apps(payload.app_names.unwrap_or_default())
        .projects(payload.project_names.unwrap_or_default())
        .entities(payload.entity_names.unwrap_or_default())
        .branches(payload.branch_names.unwrap_or_default())
        .categories(payload.category_names.unwrap_or_default())
        .languages(payload.language_names.unwrap_or_default());

    if let Some(group) = payload.group_by {
        builder = builder.group_by(group);
    }

    let db = db.lock().await;
    match builder.fetch_event_range(&db).await {
        Ok(result) => Ok(Json(result)),
        Err(err) => Err(error_response(err)),
    }
}

pub fn event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/events", post(insert_events))
        .route("/events", get(fetch_events))
        .with_state(db)
}
