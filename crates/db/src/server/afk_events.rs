use crate::{error::DBError, server::summary::SummaryQueryBuilder, DBContext};
use chrono::{DateTime, Utc};
use common::models::outputs::FullEvent;
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct AFKEvent {
    pub id: Uuid,
    pub afk_start: i64,
    pub afk_end: Option<i64>,
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

                CASE
                  WHEN typeof(afk_start) = 'integer' THEN afk_start
                  WHEN typeof(afk_start) = 'text' THEN
                    CASE
                      WHEN afk_start GLOB '[0-9]*' THEN CAST(afk_start AS INTEGER)
                      ELSE CAST(strftime('%s', afk_start) AS INTEGER)
                    END
                  ELSE NULL
                END AS afk_start_i,

                CASE
                  WHEN afk_end IS NULL THEN NULL
                  WHEN typeof(afk_end) = 'integer' THEN afk_end
                  WHEN typeof(afk_end) = 'text' THEN
                    CASE
                      WHEN afk_end GLOB '[0-9]*' THEN CAST(afk_end AS INTEGER)
                      ELSE CAST(strftime('%s', afk_end) AS INTEGER)
                    END
                  ELSE NULL
                END AS afk_end_i,

                duration
            FROM afk_events
            WHERE 1=1
            ",
        );

        if self.filters.start.is_some() || self.filters.end.is_some() {
            qb.push(" AND (1=1");

            if let Some(start) = self.filters.start {
                qb.push(" AND COALESCE(afk_end_i, CAST(strftime('%s','now') AS INTEGER)) > ")
                    .push_bind(start);
            }

            if let Some(end) = self.filters.end {
                qb.push(" AND afk_start_i < ").push_bind(end);
            }

            qb.push(")");
        }

        qb.push(" ORDER BY afk_start_i");

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

        let mut afk_events = Vec::with_capacity(rows.len());

        for row in rows {
            let id = row.try_get("id").map(Uuid::from_slice).unwrap()?;

            let timestamp: DateTime<Utc> =
                DateTime::<Utc>::from_timestamp(row.try_get::<i64, _>("afk_start_i")?, 0)
                    .unwrap_or_default();
            let end_timestamp: Option<DateTime<Utc>> =
                DateTime::<Utc>::from_timestamp(row.try_get::<i64, _>("afk_end_i")?, 0);
            let duration: Option<i64> = row.try_get("duration")?;

            afk_events.push(FullEvent {
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

        Ok(afk_events)
    }
}
