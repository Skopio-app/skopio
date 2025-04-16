#![cfg(feature = "desktop")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Event {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app_name: String,
    pub entity_name: Option<String>,
    pub entity_type: Option<String>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub branch_name: Option<String>,
    pub language_name: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end_timestamp: Option<DateTime<Utc>>,
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
}
