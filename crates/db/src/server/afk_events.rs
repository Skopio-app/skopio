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

/// Fetch recent AFK events
pub async fn fetch_recent(
    db_context: &DBContext,
    since: DateTime<Utc>,
) -> Result<Vec<AFKEvent>, DBError> {
    let rows = sqlx::query!(
        "
        SELECT id, afk_start, afk_end, duration
        FROM afk_events
        WHERE afk_start >= ?
        ORDER BY afk_events.afk_start ASC
        ",
        since
    )
    .fetch_all(db_context.pool())
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let afk_start = row.afk_start.parse::<DateTime<Utc>>()?;
            let afk_end = match row.afk_end {
                Some(ref s) => Some(s.parse::<DateTime<Utc>>()?),
                None => None,
            };
            let id = Uuid::from_slice(&row.id)?;

            Ok(AFKEvent {
                id,
                afk_start,
                afk_end,
                duration: row.duration,
            })
        })
        .collect::<Result<Vec<_>, DBError>>()?;

    Ok(events)
}

/// Fetch AFK events within a date range
pub async fn fetch_range(
    db_context: &DBContext,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<AFKEvent>, DBError> {
    let rows = sqlx::query!(
        "
        SELECT id, afk_start, afk_end, duration
        FROM afk_events
        WHERE afk_start BETWEEN ? AND ?
        ORDER BY afk_start ASC
            ",
        start_time,
        end_time
    )
    .fetch_all(db_context.pool())
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let afk_start = row.afk_start.parse::<DateTime<Utc>>()?;
            let afk_end = match row.afk_end {
                Some(ref s) => Some(s.parse::<DateTime<Utc>>()?),
                None => None,
            };
            let id = Uuid::from_slice(&row.id)?;

            Ok(AFKEvent {
                id,
                afk_start,
                afk_end,
                duration: row.duration,
            })
        })
        .collect::<Result<Vec<_>, DBError>>()?;

    Ok(events)
}
