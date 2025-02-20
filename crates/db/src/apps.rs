use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct App {
    pub id: Option<i64>,
    pub name: String,
}

impl App {
    /// Creates a new app
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT INTO apps (name) VALUES (?)", self.name)
            .execute(db_context.pool())
            .await?;

        Ok(())
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
