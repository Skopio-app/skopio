use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{utils::update_synced_in, DBContext};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: Option<i64>,
    pub timestamp: Option<NaiveDateTime>,
    pub duration: Option<i64>,
    pub activity_type: Option<String>,
    pub app_name: String,
    pub entity_name: Option<String>,
    pub entity_type: Option<String>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub end_timestamp: Option<NaiveDateTime>,
}

impl Event {
    pub async fn insert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO events (timestamp, duration, activity_type, app_name, entity_name, entity_type, project_name, project_path, branch_name, language_name, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.timestamp,
            self.duration,
            self.activity_type,
            self.app_name,
            self.entity_name,
            self.entity_type,
            self.project_name,
            self.project_path,
            self.branch_name,
            self.language_name,
            self.end_timestamp,
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }

    pub async fn unsynced(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Event,
            "
            SELECT
             id,
             timestamp,
             duration,
             activity_type,
             app_name,
             entity_name,
             entity_type,
             project_name,
             project_path,
             branch_name,
             language_name,
             end_timestamp
            FROM events
            WHERE synced = 0
            LIMIT 100
            "
        )
        .fetch_all(db_context.pool())
        .await
    }

    pub async fn mark_as_synced(
        db_context: &DBContext,
        events: &[Self],
    ) -> Result<(), sqlx::Error> {
        let ids: Vec<i64> = events.iter().filter_map(|e| e.id).collect();
        update_synced_in(db_context, "events", &ids).await
    }
}
