use crate::{error::DBError, DBContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Language {
    pub id: Uuid,
    pub name: String,
}

impl Language {
    pub async fn find_or_insert(
        db_context: &DBContext,
        name: &Option<String>,
    ) -> Result<Option<Uuid>, DBError> {
        if let Some(language) = name {
            let record = sqlx::query!("SELECT id FROM languages WHERE name = ?", language)
                .fetch_optional(db_context.pool())
                .await?;

            if let Some(row) = record {
                let id = Uuid::from_slice(&row.id)?;
                return Ok(Some(id));
            }

            let id = uuid::Uuid::now_v7();
            let result = sqlx::query!(
                "INSERT INTO languages (id, name) VALUES (?, ?) RETURNING id",
                id,
                name
            )
            .fetch_one(db_context.pool())
            .await?;
            let result_id = Uuid::from_slice(&result.id)?;
            Ok(Some(result_id))
        } else {
            Ok(None)
        }
    }

    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id AS "id: Uuid",
                name
            FROM languages
            "#
        )
        .fetch_all(db_context.pool())
        .await?;

        Ok(rows)
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query!("DELETE FROM languages WHERE id = ?", self.id)
            .execute(db_context.pool())
            .await?;
        Ok(())
    }
}
