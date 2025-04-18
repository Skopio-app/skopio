use crate::DBContext;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEvent {
    pub id: Option<i64>,
    pub afk_start: NaiveDateTime,
    pub afk_end: Option<NaiveDateTime>,
    pub duration: Option<i64>,
}

impl AFKEvent {
    /// Insert an AFK event
    pub async fn create(&self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO afk_events (afk_start, afk_end, duration)
             VALUES (?, ?, ?)
             ",
            self.afk_start,
            self.afk_end,
            self.duration
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }
}
