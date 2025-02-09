use sqlx::SqlitePool;
use chrono::{DateTime, Utc };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub duration: i64,
    pub activity_type: String,
    pub app_name: String,
    pub file_name: Option<String>,
    pub project_id: Option<i64>,
    pub branch_name: Option<String>,
    pub language: Option<String>,
    pub metadata: Option<String>,
    pub status: String,
    pub end_timestamp: Option<String>,
}

impl Event {
    // Insert a new event
    pub async fn insert(pool: &SqlitePool, event: Event) -> Result<(), sqlx::Error> {
        let timestamp = event.timestamp.to_rfc3339();

        sqlx::query!(
            "INSERT INTO EVENTS (timestamp, duration, activity_type, app_name, file_name, project_id, branch_name, language, metadata, status, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            timestamp,
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
        end_time: DateTime<Utc>
    ) -> Result<Vec<Event>, sqlx::Error> {
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();
        
        let rows = sqlx::query_as_unchecked!(
            Event,
            "SELECT
                id as 'id?',
                timestamp as 'timestamp: _',
                duration,
                activity_type,
                app_name,
                file_name as 'file_name?',
                project_id as 'project_id?',
                branch_name as 'branch_name?',
                language as 'language?',
                metadata as 'metadata?',
                status,
                end_timestamp as 'end_timestamp?'
            FROM events
            WHERE timestamp BETWEEN ? AND ?",
            start_time_str,
            end_time_str
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
