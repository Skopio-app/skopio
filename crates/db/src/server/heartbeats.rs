use crate::DBContext;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Heartbeat {
    pub id: Option<i64>,
    pub project_id: Option<i64>,
    pub entity_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub language_id: Option<i64>,
    pub app_id: Option<i64>,
    pub timestamp: NaiveDateTime,
    pub is_write: Option<bool>,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

impl Heartbeat {
    pub async fn create(&self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO heartbeats (project_id, entity_id, branch_id, language_id, app_id, timestamp, is_write, lines, cursorpos)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(self.project_id)
            .bind(self.entity_id)
            .bind(self.branch_id)
            .bind(self.language_id)
            .bind(self.app_id)
            .bind(self.timestamp)
            .bind(self.is_write)
            .bind(self.lines)
            .bind(self.cursorpos)
            .execute(db_context.pool())
            .await?;

        Ok(())
    }

    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, project_id, entity_id, branch_id, language_id, app_id, timestamp, is_write, lines, cursorpos FROM heartbeats"
            )
            .fetch_all(db_context.pool())
            .await?;

        let heartbeats = rows
            .into_iter()
            .map(|row| Heartbeat {
                id: Some(row.id),
                project_id: row.project_id,
                entity_id: row.entity_id,
                branch_id: row.branch_id,
                language_id: row.language_id,
                app_id: Some(row.app_id),
                timestamp: row.timestamp,
                is_write: row.is_write,
                lines: row.lines,
                cursorpos: row.cursorpos,
            })
            .collect();

        Ok(heartbeats)
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM heartbeats WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}
