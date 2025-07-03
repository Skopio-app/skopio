use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{update_synced_in, DBError},
    DBContext,
};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct AFKEvent {
    pub id: Option<i64>,
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

    pub async fn unsynced(db_context: &DBContext) -> Result<Vec<Self>, DBError> {
        let rows = sqlx::query!(
            r#"
            SELECT
             id,
             afk_start,
             afk_end,
             duration
            FROM afk_events
            WHERE synced = 0
            LIMIT 100
            "#
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

                Ok(AFKEvent {
                    id: Some(row.id),
                    afk_start: Some(afk_start),
                    afk_end,
                    duration: row.duration,
                })
            })
            .collect::<Result<Vec<_>, DBError>>()?;

        Ok(events)
    }

    pub async fn mark_as_synced(
        db_context: &DBContext,
        events: &[Self],
    ) -> Result<(), sqlx::Error> {
        let ids: Vec<i64> = events.iter().filter_map(|afk| afk.id).collect();
        update_synced_in(db_context, "afk_events", &ids).await
    }
}
