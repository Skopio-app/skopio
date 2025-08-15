use crate::{utils::DBError, DBContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Heartbeat {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub language_id: Option<Uuid>,
    pub app_id: Option<Uuid>,
    pub source_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub is_write: Option<bool>,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

impl Heartbeat {
    pub async fn create(&self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query(
            "INSERT INTO heartbeats (id, project_id, entity_id, branch_id, language_id, app_id, source_id, timestamp, is_write, lines, cursorpos)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(self.id)
            .bind(self.project_id)
            .bind(self.entity_id)
            .bind(self.branch_id)
            .bind(self.language_id)
            .bind(self.app_id)
            .bind(self.source_id)
            .bind(self.timestamp)
            .bind(self.is_write)
            .bind(self.lines)
            .bind(self.cursorpos)
            .execute(db_context.pool())
            .await?;

        Ok(())
    }

    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id  AS "id: Uuid",
                project_id  AS "project_id: Option<Uuid>",
                entity_id   AS "entity_id: Option<Uuid>",
                branch_id   AS "branch_id: Option<Uuid>",
                language_id AS "language_id: Option<Uuid>",
                app_id      AS "app_id: Option<Uuid>",
                source_id   AS "source_id: Uuid",
                timestamp,
                is_write,
                lines,
                cursorpos
            FROM heartbeats
            "#
        )
        .fetch_all(db_context.pool())
        .await?;

        let heartbeats = rows
            .into_iter()
            .map(|row| Heartbeat {
                id: row.id,
                project_id: row.project_id.unwrap_or_default(),
                entity_id: row.entity_id.unwrap_or_default(),
                branch_id: row.branch_id.unwrap_or_default(),
                language_id: row.language_id.unwrap_or_default(),
                app_id: row.app_id,
                source_id: row.source_id,
                timestamp: row.timestamp.parse::<DateTime<Utc>>().unwrap_or_default(),
                is_write: row.is_write,
                lines: row.lines,
                cursorpos: row.cursorpos,
            })
            .collect();

        Ok(heartbeats)
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM heartbeats WHERE id = ?", self.id)
            .execute(db_context.pool())
            .await?;
        Ok(())
    }
}
