use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Entity {
    pub id: Option<i64>,
    pub project_id: i64,
    pub name: String,
    pub entity_type: Option<String>,
}

impl Entity {
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO entities (project_id, name, type) VALUES (?, ?, ?)",
            self.project_id,
            self.name,
            self.entity_type
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }

    pub async fn all_by_project(
        db_context: &DBContext,
        project_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT id, project_id, name, type as entity_type FROM entities WHERE project_id = ?",
            project_id
        )
        .fetch_all(db_context.pool())
        .await
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM entities WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }

        Ok(())
    }
}
