#![cfg(feature = "server")]
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
    /// Finds an entity by name under a specific project, or inserts it if it doesn't exist.
    pub async fn find_or_insert(
        db_context: &DBContext,
        project_id: i64,
        name: &str,
        entity_type: &str,
    ) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT id FROM entities WHERE project_id = ? AND name = ?",
            project_id,
            name
        )
        .fetch_optional(db_context.pool())
        .await?;

        if let Some(row) = record {
            return row.id.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let result = sqlx::query!(
            "INSERT INTO entities (project_id, name, type) VALUES (?, ?, ?) RETURNING id",
            project_id,
            name,
            entity_type
        )
        .fetch_one(db_context.pool())
        .await?;

        result.id.ok_or_else(|| sqlx::Error::RowNotFound)
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
