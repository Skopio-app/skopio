#![cfg(feature = "desktop")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Heartbeat {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub entity_name: String,
    pub entity_type: String,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    pub app_name: String,
    pub is_write: bool,
    pub lines: Option<i64>,
    pub cursorpos: Option<i64>,
}

impl Heartbeat {
    pub async fn insert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO heartbeats (timestamp, project_name, project_path, entity_name, entity_name, branch_name, language_name, app_name, is_write, lines, cursorpos)
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
}
