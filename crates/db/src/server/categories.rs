use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Category {
    pub id: Option<i64>,
    pub category: String,
}

impl Category {
    /// Inserts a new category if it doesn't exist, or returns the existing ID
    pub async fn find_or_insert(db: &DBContext, name: &str) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT id from categories WHERE name = ?", name)
            .fetch_optional(db.pool())
            .await?;

        if let Some(row) = record {
            return row.id.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let result = sqlx::query!(
            "INSERT INTO categories (name) VALUES (?) RETURNING id",
            name
        )
        .fetch_one(db.pool())
        .await?;

        Ok(result.id)
    }
}
