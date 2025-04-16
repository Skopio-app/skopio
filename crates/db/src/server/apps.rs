use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct App {
    pub id: Option<i64>,
    pub name: String,
}

impl App {
    /// Finds an existing app by name or inserts a new one, returning its ID.
    pub async fn find_or_insert(db_context: &DBContext, name: &str) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM apps WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        if let Some(row) = record {
            return row.id.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let result = sqlx::query!("INSERT INTO apps (name) VALUES (?) RETURNING id", name)
            .fetch_one(db_context.pool())
            .await?;

        Ok(result.id)
    }

    /// Retrieves all apps
    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT id, name FROM apps")
            .fetch_all(db_context.pool())
            .await
    }

    /// Deletes an app
    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM apps WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}
