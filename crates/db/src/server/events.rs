use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{utils::DBError, DBContext};

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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FullEvent {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub category: String,
    pub app: Option<String>,
    pub entity: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
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

/// Fetches events given a range
pub async fn fetch_range(
    db: &DBContext,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<FullEvent>, DBError> {
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
            WHERE events.timestamp >= ? AND events.timestamp <= ?
            ORDER BY events.timestamp ASC
            "#,
        start_time,
        end_time,
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
