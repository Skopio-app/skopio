use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{update_synced_in, DBError},
    DBContext,
};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: Option<i64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: Option<String>,
    pub app_name: String,
    pub entity_name: Option<String>,
    pub entity_type: Option<String>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub source_name: String,
    pub end_timestamp: Option<DateTime<Utc>>,
}

impl Event {
    pub async fn insert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO events (timestamp, duration, category, app_name, entity_name, entity_type, project_name, project_path, branch_name, language_name, source_name, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.timestamp,
            self.duration,
            self.category,
            self.app_name,
            self.entity_name,
            self.entity_type,
            self.project_name,
            self.project_path,
            self.branch_name,
            self.language_name,
            self.source_name,
            self.end_timestamp,
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }

    pub async fn unsynced(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query!(
            "
            SELECT
             id,
             timestamp,
             duration,
             category,
             app_name,
             entity_name,
             entity_type,
             project_name,
             project_path,
             branch_name,
             language_name,
             source_name,
             end_timestamp
            FROM events
            WHERE synced = 0
            LIMIT 100
            "
        )
        .fetch_all(db_context.pool())
        .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                let timestamp = row.timestamp.parse::<DateTime<Utc>>()?;
                let end_timestamp = row.end_timestamp.parse::<DateTime<Utc>>()?;

                Ok(Event {
                    id: Some(row.id),
                    timestamp: Some(timestamp),
                    duration: row.duration,
                    category: row.category,
                    app_name: row.app_name,
                    entity_name: row.entity_name,
                    entity_type: row.entity_type,
                    project_name: row.project_name,
                    project_path: row.project_path,
                    branch_name: row.branch_name,
                    language_name: row.language_name,
                    source_name: row.source_name,
                    end_timestamp: Some(end_timestamp),
                })
            })
            .collect::<Result<Vec<_>, DBError>>()?;

        Ok(events)
    }

    pub async fn mark_as_synced(
        db_context: &DBContext,
        events: &[Self],
    ) -> Result<(), sqlx::Error> {
        let ids: Vec<i64> = events.iter().filter_map(|e| e.id).collect();
        update_synced_in(db_context, "events", &ids).await
    }

    pub async fn delete_synced(db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM events
             WHERE id IN (
                SELECT id FROM events
                WHERE synced = 1
                  AND timestamp < datetime('now', '-15days')
                LIMIT 100
            );
            "
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }
}
