use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct AFKEvent {
    pub afk_start: Option<DateTime<Utc>>,
    pub afk_end: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

impl AFKEvent {
    pub async fn insert(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO afk_events (afk_start, afk_end, duration)
            VALUES (?, ?, ?)
            ",
            self.afk_start,
            self.afk_end,
            self.duration,
        )
        .execute(db_context.pool())
        .await?;

        Ok(())
    }
}
