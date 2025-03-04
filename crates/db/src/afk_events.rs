use crate::DBContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AfkEvent {
    pub id: Option<i64>,
    pub afk_start: DateTime<Utc>,
    pub afk_end: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

impl AfkEvent {
    /// Insert an AFK event
    pub async fn insert(&self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        let afk_start_str = self.afk_start.to_rfc3339();
        let afk_end_str = self.afk_end.map(|end| end.to_rfc3339());
        sqlx::query!(
            r#"
            INSERT INTO afk_events (afk_start, afk_end, duration)
             VALUES (?, ?, ?)
             "#,
            afk_start_str,
            afk_end_str,
            self.duration
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }
}
