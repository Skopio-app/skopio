use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: Option<i64>,
    pub timestamp: NaiveDateTime,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app_id: i64,
    pub entity_id: Option<i64>,
    pub project_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub language_id: Option<i64>,
    pub end_timestamp: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FullEvent {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub end_timestamp: Option<NaiveDateTime>,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app: Option<String>,
    pub entity: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
}

impl Event {
    // Create a new event
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO events (timestamp, duration, activity_type, app_id, entity_id, project_id, branch_id, language_id, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.timestamp,
            self.duration,
            self.activity_type,
            self.app_id,
            self.entity_id,
            self.project_id,
            self.branch_id,
            self.language_id,
            self.end_timestamp
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }
}

/// Fetch recent events
pub async fn fetch_recent(
    db: &DBContext,
    since: NaiveDateTime,
) -> Result<Vec<FullEvent>, sqlx::Error> {
    sqlx::query_as!(
        FullEvent,
        r#"
            SELECT
                events.id,
                events.timestamp,
                events.end_timestamp,
                events.duration,
                events.activity_type,
                apps.name AS app,
                entities.name AS entity,
                projects.name AS project,
                branches.name AS branch,
                languages.name AS language
            FROM events
            LEFT JOIN apps ON events.app_id = apps.id
            LEFT JOIN entities ON events.entity_id = entities.id
            LEFT JOIN projects ON events.project_id = projects.id
            LEFT JOIN branches ON events.branch_id = branches.id
            LEFT JOIN languages ON events.language_id = languages.id
            WHERE events.timestamp > ?
            ORDER BY events.timestamp ASC
            "#,
        since
    )
    .fetch_all(db.pool())
    .await
}

pub async fn fetch_range(
    db: &DBContext,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
) -> Result<Vec<FullEvent>, sqlx::Error> {
    sqlx::query_as!(
        FullEvent,
        r#"
            SELECT
                events.id,
                events.timestamp,
                events.end_timestamp,
                events.duration,
                events.activity_type,
                apps.name AS app,
                entities.name AS entity,
                projects.name AS project,
                branches.name AS branch,
                languages.name AS language
            FROM events
            LEFT JOIN apps ON events.app_id = apps.id
            LEFT JOIN entities ON events.entity_id = entities.id
            LEFT JOIN projects ON events.project_id = projects.id
            LEFT JOIN branches ON events.branch_id = branches.id
            LEFT JOIN languages ON events.language_id = languages.id
            WHERE events.timestamp >= ? AND events.timestamp <= ?
            ORDER BY events.timestamp ASC
            "#,
        start_time,
        end_time,
    )
    .fetch_all(db.pool())
    .await
}
