use crate::{error::DBError, server::summary::SummaryQueryBuilder, DBContext};
use chrono::{DateTime, Utc};
use common::models::outputs::FullEvent;
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row, Sqlite};
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

impl SummaryQueryBuilder {
    /// Fetch AFK events overlapping the configured time range
    pub async fn fetch_afk_event_range(&self, db: &DBContext) -> Result<Vec<FullEvent>, DBError> {
        let mut qb = QueryBuilder::<Sqlite>::new(
            "
            SELECT
                id,
                afk_start,
                afk_end,
                duration
            FROM afk_events
            WHERE 1=1
            ",
        );

        if self.filters.start.is_some() || self.filters.end.is_some() {
            qb.push(" AND (1=1");

            if let Some(start) = self.filters.start {
                qb.push(
                    " AND CAST(strftime('%s', COALESCE(afk_end, datetime('now'))) AS INTEGER) > ",
                )
                .push_bind(start);
            }

            if let Some(end) = self.filters.end {
                qb.push(" AND CAST(strftime('%s', afk_start) AS INTEGER) < ")
                    .push_bind(end);
            }

            qb.push(")");
        }

        qb.push(" ORDER BY CAST(strftime('%s', afk_start) AS INTEGER)");

        let query = qb.build();

        #[cfg(debug_assertions)]
        {
            use crate::utils::explain_query;
            use log::{info, warn};
            use sqlx::Execute;

            let sql = query.sql();
            info!("Executing AFK range query: {}", sql);
            if let Err(e) = explain_query(db.pool(), sql).await {
                warn!("Failed to explain AFK query: {}", e);
            }
        }

        let rows = query.fetch_all(db.pool()).await?;

        let mut events = Vec::with_capacity(rows.len());

        for row in rows {
            let id = row.try_get::<Vec<u8>, _>("id").and_then(|bytes| {
                Uuid::from_slice(&bytes).map_err(|e| sqlx::Error::Decode(Box::new(e)))
            })?;

            let afk_start_s: String = row.try_get("afk_start")?;
            let afk_end_s: Option<String> = row.try_get("afk_end")?;
            let duration: Option<i64> = row.try_get("duration")?;

            let timestamp = DateTime::parse_from_rfc3339(&afk_start_s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_default();

            let end_timestamp = afk_end_s
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            events.push(FullEvent {
                id,
                timestamp,
                end_timestamp,
                duration,
                category: "AFK".to_string(),
                app: None,
                entity: None,
                entity_type: None,
                project: None,
                branch: None,
                language: None,
                source: "skopio-desktop".to_string(),
            });
        }

        Ok(events)
    }
}
