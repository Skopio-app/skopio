#![cfg(feature = "server")]
use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Branch {
    pub id: Option<i64>,
    pub project_id: i64,
    pub name: String,
}

impl Branch {
    /// Inserts a new branch if it doesn't exist, or returns the existing ID
    pub async fn find_or_insert(
        db_context: &DBContext,
        project_id: i64,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT id FROM branches WHERE project_id = ? AND name = ?",
            project_id,
            name
        )
        .fetch_optional(db_context.pool())
        .await?;

        if let Some(row) = record {
            return Ok(row.id);
        }

        let result = sqlx::query!(
            "INSERT INTO branches (project_id, name) VALUES (?, ?) RETURNING id",
            project_id,
            name
        )
        .fetch_one(db_context.pool())
        .await?;

        result.id.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn all_project(
        db_context: &DBContext,
        project_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
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
