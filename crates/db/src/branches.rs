use serde::{Deserialize, Serialize};
use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Branch {
    pub id: Option<i64>,
    pub project_id: i64,
    pub name: String,
}

impl Branch {
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO branches (project_id, name) VALUES (?, ?)",
            self.project_id,
            self.name
        )
            .execute(db_context.pool())
            .await?;

        Ok(())
    }

    pub async fn all_project(db_context: &DBContext, project_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT id, project_id, name FROM branches WHERE project_id = ?",
            project_id
        )
            .fetch_all(db_context.pool())
            .await
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM branches WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}