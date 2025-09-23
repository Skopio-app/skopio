use crate::{error::DBError, models::Category, DBContext};
use chrono::Utc;
use uuid::Uuid;

impl Category {
    /// Inserts a new category if it doesn't exist, or returns the existing ID
    pub async fn find_or_insert(db: &DBContext, name: &str) -> Result<Uuid, DBError> {
        let record = sqlx::query!("SELECT id from categories WHERE name = ?", name)
            .fetch_optional(db.pool())
            .await?;

        let timestamp = Utc::now().timestamp();

        if let Some(row) = record {
            sqlx::query!("UPDATE categories SET last_updated = ?", timestamp)
                .execute(db.pool())
                .await?;
            let id = Uuid::from_slice(&row.id)?;
            return Ok(id);
        }

        let id = Uuid::now_v7();
        let result = sqlx::query!(
            "INSERT INTO categories (id, name, last_updated) VALUES (?, ?, ?) RETURNING id",
            id,
            name,
            timestamp
        )
        .fetch_one(db.pool())
        .await?;

        let result_id = Uuid::from_slice(&result.id)?;
        Ok(result_id)
    }

    /// Retrieves all categories
    pub async fn get_all(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id AS "id: Uuid",
                name
            FROM categories
            "#
        )
        .fetch_all(db_context.pool())
        .await?;

        Ok(rows)
    }
}
