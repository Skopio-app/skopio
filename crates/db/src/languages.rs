use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Language {
    pub id: Option<i64>,
    pub name: String,
}

impl Language {
    pub async fn create(self, db_context: &DBContext) -> Result<Self, sqlx::Error> {
        sqlx::query!("INSERT INTO languages (name) VALUES (?)", self.name)
            .execute(db_context.pool())
            .await?;

        Ok(self)
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
