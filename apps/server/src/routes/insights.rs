use std::{collections::BTreeMap, sync::Arc};

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use common::models::inputs::Group;
use db::{
    server::insights::{Aggregation, InsightProvider, InsightQuery, InsightType, Insights},
    DBContext,
};
use tokio::sync::Mutex;

use crate::utils::error_response;

pub async fn fetch_active_years(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<i32>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;
    let query = InsightQuery {
        insight_type: InsightType::ActiveYears,
        year: None,
        month: None,
        week: None,
        day: None,
    };

    let result = Insights::execute(&db, query)
        .await
        .map_err(error_response)?;

    let years = result.into_active_years().map_err(error_response)?;

    Ok(Json(years))
}

pub async fn fetch_aggregated_average(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<BTreeMap<String, Vec<(String, f64)>>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let query = InsightQuery {
        insight_type: InsightType::AggregatedAverage {
            bucket: Aggregation::Month,
            group_by: None,
        },
        year: Some(2025),
        month: None,
        week: None,
        day: None,
    };

    let result = Insights::execute(&db, query)
        .await
        .map_err(error_response)?;

    let data = result.into_aggregated_average().map_err(error_response)?;

    Ok(Json(data))
}

pub async fn fetch_top_n(
    State(db): State<Arc<Mutex<DBContext>>>,
) -> Result<Json<Vec<(String, i64)>>, (StatusCode, Json<String>)> {
    let db = db.lock().await;

    let query = InsightQuery {
        insight_type: InsightType::TopN {
            group_by: Group::Project,
            limit: 4,
        },
        year: Some(2025),
        month: None,
        week: None,
        day: None,
    };

    let result = Insights::execute(&db, query)
        .await
        .map_err(error_response)?;

    let data = result.into_top_n().map_err(error_response)?;

    Ok(Json(data))
}

pub fn insights_routes(db: Arc<Mutex<DBContext>>) -> Router {
    Router::new()
        .route("/insights/years", get(fetch_active_years))
        .route("/insights/average", get(fetch_aggregated_average))
        .route("/insights/top", get(fetch_top_n))
        .with_state(db)
}
