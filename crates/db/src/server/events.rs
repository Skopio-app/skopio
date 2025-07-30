use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::models::outputs::{EventGroup, EventGroupResult, FullEvent};
use log::info;
use serde::{Deserialize, Serialize};

use sqlx::Row;

use crate::{
    server::{
        summary::SummaryQueryBuilder,
        utils::query::{
            append_all_filters, append_date_range, append_group_by, append_standard_joins,
            group_key_column,
        },
    },
    utils::DBError,
    DBContext,
};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Event {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub duration: Option<i64>,
    pub category_id: i64,
    pub app_id: i64,
    pub entity_id: Option<i64>,
    pub project_id: Option<i64>,
    pub branch_id: Option<i64>,
    pub language_id: Option<i64>,
    pub end_timestamp: Option<DateTime<Utc>>,
}

impl Event {
    // Create a new event
    pub async fn create(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO events (timestamp, duration, category_id, app_id, entity_id, project_id, branch_id, language_id, end_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            self.timestamp,
            self.duration,
            self.category_id,
            self.app_id,
            self.entity_id,
            self.project_id,
            self.branch_id,
            self.language_id,
            self.end_timestamp
        )
        .execute(db_context.pool())
        .await?;
        Ok(())
    }
}

// TODO: Add time bucket implementation
/// Fetch recent events
pub async fn fetch_recent(db: &DBContext, since: DateTime<Utc>) -> Result<Vec<FullEvent>, DBError> {
    let rows = sqlx::query!(
        r#"
            SELECT
                events.id,
                events.timestamp,
                events.end_timestamp,
                events.duration,
                apps.name AS app,
                categories.name AS category,
                entities.name AS entity,
                projects.name AS project,
                branches.name AS branch,
                languages.name AS language
            FROM events
            LEFT JOIN apps ON events.app_id = apps.id
            LEFT JOIN entities ON events.entity_id = entities.id
            LEFT JOIN projects ON events.project_id = projects.id
            LEFT JOIN branches ON events.branch_id = branches.id
            LEFT JOIN languages ON events.language_id = languages.id
            LEFT JOIN categories ON events.category_id = categories.id
            WHERE events.timestamp > ?
            ORDER BY events.timestamp ASC
            "#,
        since
    )
    .fetch_all(db.pool())
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let timestamp = row.timestamp.parse::<DateTime<Utc>>()?;
            let end_timestamp = match row.end_timestamp {
                Some(ref s) => Some(s.parse::<DateTime<Utc>>()?),
                None => None,
            };

            Ok(FullEvent {
                id: row.id,
                timestamp,
                end_timestamp,
                duration: row.duration,
                category: row.category.unwrap_or_default(),
                app: row.app,
                entity: row.entity,
                project: row.project,
                branch: row.branch,
                language: row.language,
            })
        })
        .collect::<Result<Vec<_>, DBError>>()?;

    Ok(events)
}

impl SummaryQueryBuilder {
    /// Fetches events given a range
    pub async fn fetch_event_range(&self, db: &DBContext) -> Result<EventGroupResult, DBError> {
        let is_grouped = self.group_by.is_some();
        info!("self: {:?}", self);
        let group_key = group_key_column(self.group_by);

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
                projects.name AS project,
                branches.name AS branch,
                languages.name AS language
                {select_group}
            FROM events
            "
        );

        append_standard_joins(&mut query);
        query.push_str(" WHERE 1=1");

        append_date_range(
            &mut query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );
        append_all_filters(&mut query, self.filters.clone());

        if self.group_by.is_some() {
            append_group_by(&mut query, Some(group_key));
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

            let event = FullEvent {
                id: row.try_get("id")?,
                timestamp,
                end_timestamp,
                duration: row.try_get("duration")?,
                category: row
                    .try_get::<Option<String>, _>("category")?
                    .unwrap_or_default(),
                app: row.try_get("app")?,
                entity: row.try_get("entity")?,
                project: row.try_get("project")?,
                branch: row.try_get("branch")?,
                language: row.try_get("language")?,
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
