use crate::error::AppResult;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use common::models::inputs::{BucketSummaryInput, EventInput};
use common::models::outputs::EventGroupResult;
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
use uuid::Uuid;

async fn insert_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    Json(payload): Json<Vec<EventInput>>,
) -> AppResult<()> {
    debug!("Handling {} events", payload.len());

    let db = db.lock().await;

    let mut staged: Vec<Event> = Vec::with_capacity(payload.len());

    for event in payload {
        let app_id = App::find_or_insert(&db, &event.app_name).await?;
        let project_id =
            ServerProject::find_or_insert(&db, &event.project_name, &event.project_path).await?;
        let branch_id = Branch::find_or_insert(&db, project_id, &event.branch_name).await?;
        let entity_id =
            Entity::find_or_insert(&db, project_id, &event.entity_name, &event.entity_type).await?;
        let language_id = Language::find_or_insert(&db, &event.language_name).await?;
        let category_id = Category::find_or_insert(&db, &event.category).await?;
        let source_id = Source::find_or_insert(&db, &event.source_name).await?;

        staged.push(Event {
            id: Uuid::now_v7(),
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
        });
    }

    let inserted = Event::bulk_create(&db, &staged).await?;
    info!("Inserted {} events", inserted);

    Ok(())
}

async fn fetch_events(
    State(db): State<Arc<Mutex<DBContext>>>,
    QsQuery(payload): QsQuery<BucketSummaryInput>,
) -> AppResult<Json<EventGroupResult>> {
    let db = db.lock().await;
    let builder = SummaryQueryBuilder::from(payload);
    let result = builder.fetch_event_range(&db).await?;

    Ok(Json(result))
}

pub fn event_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/events", post(insert_events))
        .route("/events", get(fetch_events))
        .with_state(db)
}
