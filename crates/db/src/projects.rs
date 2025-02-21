use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub root_path: Option<String>,
}

impl Project {
    pub async fn find_or_insert(db_context: &DBContext, name: &str, root_path: &str) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM projects WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        if let Some(row) = record {
            return Ok(row.id);
        }

        let result = sqlx::query!(
            "INSERT INTO projects (name, root_path) VALUES (?, ?) RETURNING id",
            name,
            root_path,
        )
        .fetch_one(db_context.pool())
        .await?;

        Ok(result.id)
    }

    /// Fetches a project by name
    pub async fn find_by_name(
        db_context: &DBContext,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
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
