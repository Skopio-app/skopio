use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{utils::update_synced_in, DBContext};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Heartbeat {
    pub id: Option<i64>,
    pub timestamp: Option<NaiveDateTime>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub entity_name: String,
    pub entity_type: String,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub app_name: String,
    pub is_write: Option<bool>,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

impl Heartbeat {
    pub async fn insert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO heartbeats (timestamp, project_name, project_path, entity_name, entity_type, branch_name, language_name, app_name, is_write, lines, cursorpos)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.timestamp,
            self.project_name,
            self.project_path,
            self.entity_name,
            self.entity_type,
            self.branch_name,
            self.language_name,
            self.app_name,
            self.is_write,
            self.lines,
            self.cursorpos
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }

    pub async fn unsynced(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Heartbeat,
            "
            SELECT
             id,
             timestamp,
             project_name,
             project_path,
             entity_name,
             entity_type,
             branch_name,
             language_name,
             app_name,
             is_write,
             lines,
             cursorpos
            FROM heartbeats
            WHERE synced = 0
            LIMIT 100
            "
        )
        .fetch_all(db_context.pool())
        .await
    }

    pub async fn mark_as_synced(
        db_context: &DBContext,
        heartbeats: &[Self],
    ) -> Result<(), sqlx::Error> {
        let ids: Vec<i64> = heartbeats.iter().filter_map(|h| h.id).collect();
        update_synced_in(db_context, "heartbeats", &ids).await
    }
}
