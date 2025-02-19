use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub root_path: Option<String>,
}

impl Project {
    /// Creates a new project
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR IGNORE INTO projects (name, root_path) VALUES (?, ?)",
            self.name,
            self.root_path,
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }

    /// Fetches a project by name
    pub async fn find_by_name(db_context: &DBContext, name: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Project,
            "SELECT id, name, root_path FROM projects WHERE NAME = ?",
            name
        )
        .fetch_optional(db_context.pool())
        .await
    }

    /// Deletes a project
    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM projects WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}
