use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::models::outputs::{EventGroup, EventGroupResult, FullEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sqlx::Row;

use crate::{
    error::DBError,
    server::{
        summary::SummaryQueryBuilder,
        utils::query::{
            append_all_filters, append_date_range, append_standard_joins, group_key_info,
        },
    },
    DBContext,
};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration: Option<i64>,
    pub category_id: Uuid,
    pub app_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub language_id: Option<Uuid>,
    pub source_id: Uuid,
    pub end_timestamp: Option<DateTime<Utc>>,
}

impl Event {
    // Inserts a new event into the database
    pub async fn create(self, db_context: &DBContext) -> Result<(), DBError> {
        sqlx::query!(
            "
            INSERT INTO events (id, timestamp, duration, category_id, app_id, entity_id, project_id, branch_id, language_id, source_id, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.id,
            self.timestamp,
            self.duration,
            self.category_id,
            self.app_id,
            self.entity_id,
            self.project_id,
            self.branch_id,
            self.language_id,
            self.source_id,
            self.end_timestamp
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }
}

impl SummaryQueryBuilder {
    /// Fetches events within the configured time range and optional filters,
    /// and returns them either as a flat list or grouped by a chosen dimension
    pub async fn fetch_event_range(&self, db: &DBContext) -> Result<EventGroupResult, DBError> {
        let is_grouped = self.filters.group_by.is_some();
        let (group_key, inner_tbl) = group_key_info(self.filters.group_by);

        let select_group = if is_grouped {
            format!(", {group_key} AS group_key")
        } else {
            String::new()
        };

        let mut query = format!(
            "
            SELECT
                events.id,
                events.timestamp,
                events.end_timestamp,
                events.duration,
                apps.name AS app,
                categories.name AS category,
                entities.name AS entity,
                entities.type AS entity_type,
                projects.name AS project,
                branches.name AS branch,
                languages.name AS language,
                sources.name AS source
                {select_group}
            FROM events
            "
        );

        append_standard_joins(&mut query, inner_tbl);
        query.push_str(" WHERE 1=1");

        append_date_range(
            &mut query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );
        append_all_filters(&mut query, self.filters.clone());

        if self.filters.group_by.is_some() {
            query.push_str(&format!(" ORDER BY {}, events.timestamp", group_key));
        } else {
            query.push_str(" ORDER BY events.timestamp");
        }

        let rows = sqlx::query(&query).fetch_all(db.pool()).await?;

        let mut flat_events = Vec::new();
        let mut grouped_events: HashMap<String, Vec<FullEvent>> = HashMap::new();

        for row in rows {
            let timestamp = row
                .try_get::<String, _>("timestamp")?
                .parse::<DateTime<Utc>>()?;
            let end_timestamp = row
                .try_get::<Option<String>, _>("end_timestamp")?
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()?;

            let id = row.try_get("id").map(|id| Uuid::from_slice(id)).unwrap()?;

            let event = FullEvent {
                id,
                timestamp,
                end_timestamp,
                duration: row.try_get("duration")?,
                category: row
                    .try_get::<Option<String>, _>("category")?
                    .unwrap_or_default(),
                app: row.try_get("app")?,
                entity: row.try_get("entity")?,
                entity_type: row.try_get("entity_type")?,
                project: row.try_get("project")?,
                branch: row.try_get("branch")?,
                language: row.try_get("language")?,
                source: row.try_get("source")?,
            };

            if is_grouped {
                let group_key = row.try_get::<String, _>("group_key")?;
                grouped_events.entry(group_key).or_default().push(event);
            } else {
                flat_events.push(event);
            }
        }

        if is_grouped {
            let grouped_vec = grouped_events
                .into_iter()
                .map(|(group, events)| EventGroup { group, events })
                .collect();

            Ok(EventGroupResult::Grouped(grouped_vec))
        } else {
            Ok(EventGroupResult::Flat(flat_events))
        }
    }
}
