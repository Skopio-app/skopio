use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Language {
    pub id: Option<i64>,
    pub name: String,
}

impl Language {
    pub async fn find_or_insert(db_context: &DBContext, name: &str) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM languages WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        if let Some(row) = record {
            return row.id.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let result = sqlx::query!("INSERT INTO languages (name) VALUES (?) RETURNING id", name)
            .fetch_one(db_context.pool())
            .await?;

        Ok(result.id)
    }

    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT id, name FROM languages")
            .fetch_all(db_context.pool())
            .await
    }

    pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!("DELETE FROM languages WHERE id = ?", id)
                .execute(db_context.pool())
                .await?;
        }
        Ok(())
    }
}
