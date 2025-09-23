use crate::{error::DBError, models::App, DBContext};
use chrono::Utc;
use uuid::Uuid;

impl App {
    /// Finds an existing app by name or inserts a new one, returning its ID.
    pub async fn find_or_insert(db_context: &DBContext, name: &str) -> Result<Uuid, DBError> {
        let record = sqlx::query!("SELECT id FROM apps WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        let timestamp = Utc::now().timestamp();

        if let Some(row) = record {
            sqlx::query!("UPDATE projects SET last_updated = ?", timestamp)
                .execute(db_context.pool())
                .await?;
            let id = Uuid::from_slice(&row.id)?;
            return Ok(id);
        }

        let id = Uuid::now_v7();
        let result = sqlx::query!(
            "INSERT INTO apps (id, name, last_updated) VALUES (?, ?, ?) RETURNING id",
            id,
            name,
            timestamp
        )
        .fetch_one(db_context.pool())
        .await?;
        let result_id = Uuid::from_slice(&result.id)?;

        Ok(result_id)
    }

    /// Retrieves all apps
    pub async fn get_all(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let apps = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id AS "id: Uuid",
                name
              FROM apps
            "#
        )
        .fetch_all(db_context.pool())
        .await?;
        Ok(apps)
    }

    /// Deletes an app
    pub async fn delete(self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query!("DELETE FROM apps WHERE id = ?", self.id)
            .execute(db_context.pool())
            .await?;
        Ok(())
    }
}
