use crate::DBContext;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct YearlySummary {
    pub id: Option<i64>,
    pub year: i64,
    pub total_active_time: i64,
    pub total_afk_time: i64,
    pub most_active_app: Option<String>,
    pub most_active_project: Option<String>,
    pub most_active_language: Option<String>,
    pub metadata: Option<String>,
    pub last_updated: String,
}

impl YearlySummary {
    /// Insert a new yearly summary or update it if it already exists.
    pub async fn upsert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO yearly_summaries (year, total_active_time, total_afk_time, most_active_app, most_active_project, most_active_language, metadata, last_updated)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(year) DO UPDATE SET
                total_active_time = excluded.total_active_time,
                total_afk_time = excluded.total_afk_time,
                most_active_app = excluded.most_active_app,
                most_active_project = excluded.most_active_project,
                most_active_language = excluded.most_active_language,
                metadata = excluded.metadata,
                last_updated = excluded.last_updated",
                self.year,
                self.total_active_time,
                self.total_afk_time,
                self.most_active_app,
                self.most_active_project,
                self.most_active_language,
                self.metadata,
                self.last_updated
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }

    /// Fetch the yearly summary for a specific year
    pub async fn fetch_by_year(
        db_context: &DBContext,
        year: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT id, year, total_active_time, total_afk_time, most_active_app, most_active_project, most_active_language, metadata, last_updated
            FROM yearly_summaries WHERE year = ?",
            year
        )
        .fetch_optional(db_context.pool())
        .await
    }
}
