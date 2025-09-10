use crate::{error::DBError, DBContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Entity {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub entity_type: Option<String>,
}

impl Entity {
    /// Finds an entity by name under a specific project, or inserts it if it doesn't exist.
    pub async fn find_or_insert(
        db_context: &DBContext,
        project_id: Uuid,
        name: &str,
        entity_type: &str,
    ) -> Result<Uuid, DBError> {
        let record = sqlx::query!(
            "SELECT id FROM entities WHERE project_id = ? AND name = ?",
            project_id,
            name
        )
        .fetch_optional(db_context.pool())
        .await?;

        if let Some(row) = record {
            let id = Uuid::from_slice(&row.id)?;
            return Ok(id);
        }

        let id = uuid::Uuid::now_v7();
        let result = sqlx::query!(
            "INSERT INTO entities (id, project_id, name, type) VALUES (?, ?, ?, ?) RETURNING id",
            id,
            project_id,
            name,
            entity_type
        )
        .fetch_one(db_context.pool())
        .await?;

        let result_id = Uuid::from_slice(&result.id)?;
        Ok(result_id)
    }

    pub async fn all_by_project(
        db_context: &DBContext,
        project_id: Uuid,
    ) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id AS "id: Uuid",
                project_id  AS "project_id: Uuid",
                name,
                type as entity_type
                FROM entities
                WHERE project_id = ?
                "#,
            project_id
        )
        .fetch_all(db_context.pool())
        .await?;

        Ok(rows)
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query!("DELETE FROM entities WHERE id = ?", self.id)
            .execute(db_context.pool())
            .await?;
        Ok(())
    }
}
