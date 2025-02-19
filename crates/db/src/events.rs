use sqlx::SqlitePool;
use chrono::{DateTime, NaiveDateTime, Utc };
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

    // // Fetch events within a time range
    // pub async fn fetch_events_in_range(
    //     pool: &SqlitePool,
    //     start_time: DateTime<Utc>,
    //     end_time: DateTime<Utc>,
    // ) -> Result<Vec<Event>, sqlx::Error> {
    //     let events = sqlx::query_as!(
    //         Event,
    //         r#"
    //         SELECT
    //             id,
    //             timestamp,
    //             duration,
    //             activity_type,
    //             app_name,
    //             file_name,
    //             project_id,
    //             branch_name,
    //             language,
    //             metadata,
    //             status,
    //             end_timestamp
    //         FROM events
    //         WHERE timestamp BETWEEN ? AND ?
    //         "#,
    //         start_time,
    //         end_time
    //     )
    //     .fetch_all(pool)
    //     .await?;
    //
    //     Ok(events)
    // }

    pub async fn fetch_by_project(db_context: &DBContext, project_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM events WHERE project_id = ?",
            project_id
        )
            .fetch_all(db_context.pool())
            .await
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM events WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}
