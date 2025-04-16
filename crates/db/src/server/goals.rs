#![cfg(feature = "server")]

use crate::DBContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Goal {
    pub id: Option<i64>,
    pub name: String,
    pub target_duration: i64,
    pub frequency: String,
    pub exclude_days: Option<String>,
    pub progress: i64,
}

impl Goal {
    /// Creates a new goal
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO goals (name, target_duration, frequency, exclude_days, progress)
            VALUES (?, ?, ?, ?, ?)",
            self.name,
            self.target_duration,
            self.frequency,
            self.exclude_days,
            self.progress,
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }

    /// Update a goal progress by a specified amount
    pub async fn update_progress(
        self,
        db_context: &DBContext,
        increment: i64,
    ) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            sqlx::query!(
                "UPDATE goals SET progress = progress + ? WHERE id = ?",
                increment,
                id
            )
            .execute(db_context.pool())
            .await?;
        }

        Ok(())
    }

    /// Fetch all goals.
    pub async fn all(db_context: &DBContext) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Goal>("SELECT * FROM goals")
            .fetch_all(db_context.pool())
            .await
    }
}
