use crate::{error::DBError, DBContext};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Branch {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
}

impl Branch {
    /// Inserts a new branch if it doesn't exist, or returns the existing ID
    pub async fn find_or_insert(
        db_context: &DBContext,
        project_id: Uuid,
        name: &Option<String>,
    ) -> Result<Option<Uuid>, DBError> {
        if let Some(branch) = name {
            let record = sqlx::query!(
                "SELECT id FROM branches WHERE project_id = ? AND name = ?",
                project_id,
                branch
            )
            .fetch_optional(db_context.pool())
            .await?;

            let timestamp = Utc::now().timestamp();

            if let Some(row) = record {
                sqlx::query!("UPDATE branches SET last_updated = ?", timestamp)
                    .execute(db_context.pool())
                    .await?;
                let id = Uuid::from_slice(&row.id)?;
                return Ok(Some(id));
            }

            let id = uuid::Uuid::now_v7();
            let result = sqlx::query!(
                "INSERT INTO branches (id, project_id, name, last_updated) VALUES (?, ?, ?, ?) RETURNING id",
                id,
                project_id,
                name,
                timestamp
            )
            .fetch_one(db_context.pool())
            .await?;

            let result_id = Uuid::from_slice(&result.id)?;
            Ok(Some(result_id))
        } else {
            Ok(None)
        }
    }

    pub async fn all_project(
        db_context: &DBContext,
        project_id: Uuid,
    ) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id AS "id: Uuid",
                project_id  AS "project_id: Uuid",
                name
            FROM branches
            WHERE project_id = ?
            "#,
            project_id
        )
        .fetch_all(db_context.pool())
        .await?;

        Ok(rows)
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query!("DELETE FROM branches WHERE id = ?", self.id)
            .execute(db_context.pool())
            .await?;
        Ok(())
    }
}
