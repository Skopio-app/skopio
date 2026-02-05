use crate::{error::DBError, DBContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEvent {
    pub id: Uuid,
    pub afk_start: DateTime<Utc>,
    pub afk_end: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

impl AFKEvent {
    /// Bulk inserts AFK events into the database
    pub async fn bulk_create(db_context: &DBContext, events: &[Self]) -> Result<u64, DBError> {
        if events.is_empty() {
            return Ok(0);
        }

        let mut tx = db_context.pool().begin().await?;
        let mut total_inserted: u64 = 0;

        for ev in events {
            let res = sqlx::query!(
                "INSERT OR IGNORE INTO afk_events (id, afk_start, afk_end, duration)
                 VALUES (?, ?, ?, ?)
                 ON CONFLICT(id) DO NOTHING",
                ev.id,
                ev.afk_start,
                ev.afk_end,
                ev.duration
            )
            .execute(&mut *tx)
            .await?;

            total_inserted += res.rows_affected();
        }

        tx.commit().await?;
        Ok(total_inserted)
    }
}
