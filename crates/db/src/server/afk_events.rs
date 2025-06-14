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

    /// Fetch AFK events within a date range
    pub async fn fetch_in_range(
        db_context: &DBContext,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Result<Vec<AFKEvent>, sqlx::Error> {
        sqlx::query_as!(
            AFKEvent,
            r#"
            SELECT id, afk_start, afk_end, duration
            FROM afk_events
            WHERE afk_start BETWEEN ? AND ?
            ORDER BY afk_start ASC
            "#,
            start,
            end
        )
        .fetch_all(db_context.pool())
        .await
    }
}

/// Fetch recent AFK events
pub async fn fetch_recent(
    db_context: &DBContext,
    since: NaiveDateTime,
) -> Result<Vec<AFKEvent>, sqlx::Error> {
    sqlx::query_as!(
        AFKEvent,
        r#"
        SELECT id, afk_start, afk_end, duration
        FROM afk_events
        WHERE afk_start >= ?
        ORDER BY afk_events.afk_start ASC
        "#,
        since
    )
    .fetch_all(db_context.pool())
    .await
}

pub async fn fetch_range(
    db_context: &DBContext,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
) -> Result<Vec<AFKEvent>, sqlx::Error> {
    sqlx::query_as!(
        AFKEvent,
        r#"
        SELECT id, afk_start, afk_end, duration
        FROM afk_events
        WHERE afk_start >= ? AND afk_start <= ?
        ORDER BY afk_events.afk_start ASC
        "#,
        start_time,
        end_time,
    )
    .fetch_all(db_context.pool())
    .await
}
