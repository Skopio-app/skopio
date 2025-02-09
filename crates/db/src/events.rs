use sqlx::SqlitePool;
use chrono::{DateTime, NaiveDateTime, Utc };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app_name: String,
    pub file_name: Option<String>,
    pub project_id: Option<i64>,
    pub branch_name: Option<String>,
    pub language: Option<String>,
    pub metadata: Option<String>,
    pub status:Option<String>,
    pub end_timestamp: Option<NaiveDateTime>,
}

impl Event {
    // Insert a new event
    pub async fn insert(pool: &SqlitePool, event: Event) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO EVENTS (timestamp, duration, activity_type, app_name, file_name, project_id, branch_name, language, metadata, status, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            event.timestamp,
            event.duration,
            event.activity_type,
            event.app_name,
            event.file_name,
            event.project_id,
            event.branch_name,
            event.language,
            event.metadata,
            event.status,
            event.end_timestamp
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    // Fetch events within a time range
    pub async fn fetch_events_in_range(
        pool: &SqlitePool,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Event>, sqlx::Error> {
        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT
                id,
                timestamp,
                duration,
                activity_type,
                app_name,
                file_name,
                project_id,
                branch_name,
                language,
                metadata,
                status,
                end_timestamp
            FROM events
            WHERE timestamp BETWEEN ? AND ?
            "#,
            start_time,
            end_time
        )
        .fetch_all(pool)
        .await?;

        Ok(events)
    }
}
