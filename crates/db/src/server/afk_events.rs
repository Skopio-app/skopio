use crate::{utils::DBError, DBContext};
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
    /// Insert an AFK event
    pub async fn create(&self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO afk_events (id, afk_start, afk_end, duration)
             VALUES (?, ?, ?, ?)
             ",
            self.id,
            self.afk_start,
            self.afk_end,
            self.duration
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
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
