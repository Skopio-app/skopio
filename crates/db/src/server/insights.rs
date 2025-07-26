use async_trait::async_trait;
use chrono::{NaiveDate, Weekday};
use common::models::inputs::Group;
use sqlx::Row;

use crate::DBContext;

#[derive(sqlx::FromRow)]
struct YearResult {
    year: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Aggregation {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone)]
pub enum InsightType {
    ActiveYears,
    WeekdayAverages,
    TopN { group_by: Group, limit: usize },
    MostActiveDay,
    AggregatedAverage,
}

pub struct InsightQuery {
    pub insight_type: InsightType,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub week: Option<u32>,
    pub day: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub enum InsightResult {
    ActiveYears(Vec<i32>),
    WeekdayAverage(Vec<(Weekday, i64)>),
    TopN(Vec<(String, i64)>),
    MostActiveDay { date: String, total_duration: i64 },
    AggregatedAverage(Vec<(String, f64)>),
}

#[async_trait]
pub trait InsightProvider {
    async fn execute(
        db_context: &DBContext,
        query: InsightQuery,
    ) -> Result<InsightResult, sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct Insights {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub week: Option<u32>,
    pub day: Option<u32>,
    pub category_ids: Option<Vec<i64>>,
    pub app_ids: Option<Vec<i64>>,
    pub project_ids: Option<Vec<i64>>,
    pub entity_ids: Option<Vec<i64>>,
    pub branch_ids: Option<Vec<i64>>,
    pub language_ids: Option<Vec<i64>>,
    pub top_n: Option<usize>,
    pub kind: InsightType,
}

#[async_trait]
impl InsightProvider for Insights {
    async fn execute(
        db_context: &DBContext,
        query: InsightQuery,
    ) -> Result<InsightResult, sqlx::Error> {
        match query.insight_type {
            InsightType::ActiveYears => {
                let rows: Vec<YearResult> = sqlx::query_as!(
                    YearResult,
                    "
                    SELECT DISTINCT strftime('%Y', datetime(timestamp, 'localtime')) as year
                    FROM events
                    ORDER BY year DESC
                    "
                )
                .fetch_all(db_context.pool())
                .await?;

                let years = rows
                    .into_iter()
                    .filter_map(|r| r.year.and_then(|y| y.parse::<i32>().ok()))
                    .collect();

                Ok(InsightResult::ActiveYears(years))
            }

            InsightType::WeekdayAverages => {
                let rows = sqlx::query!(
                    "
                    SELECT strftime('%w', datetime(timestamp, 'localtime')) as weekday,
                            AVG(duration) as avg_duration
                    FROM events
                    GROUP BY weekday
                    "
                )
                .fetch_all(db_context.pool())
                .await?;

                let weekday_avg = rows
                    .into_iter()
                    .filter_map(|r| {
                        let weekday = r.weekday?.parse::<u32>().ok()?;
                        let duration = r.avg_duration.unwrap_or(0);
                        Some((Weekday::try_from(weekday as u8).ok()?, duration))
                    })
                    .collect();

                Ok(InsightResult::WeekdayAverage(weekday_avg))
            }

            InsightType::TopN { group_by, limit } => {
                let (_field, join) = match group_by {
                    Group::Project => ("project_id", "JOIN projects p ON p.id = e.project_id"),
                    Group::App => ("app_id", "JOIN apps a ON a.id = e.app_id"),
                    Group::Category => ("category_id", "JOIN categories c ON c.id = e.category_id"),
                    Group::Branch => ("branch_id", "JOIN branches b ON b.id = e.branch_id"),
                    Group::Entity => ("entity_id", "JOIN entities en ON en.id = e.entity_id"),
                    Group::Language => ("language_id", "JOIN langauges l ON l.id = e.language_id"),
                };

                let label = match group_by {
                    Group::Project => "p.name",
                    Group::App => "a.name",
                    Group::Category => "c.name",
                    Group::Branch => "b.name",
                    Group::Entity => "en.name",
                    Group::Language => "l.name",
                };

                let query_string = format!(
                    "
                        SELECT {label} as name, SUM(e.duration) as total_duration
                        FROM events e
                        {join}
                        GROUP BY {label}
                        ORDER BY total_duration DESC
                        LIMIT ?
                        "
                );

                let rows = sqlx::query(&query_string)
                    .bind(limit as i64)
                    .fetch_all(db_context.pool())
                    .await?;

                let results = rows
                    .into_iter()
                    .filter_map(|r| {
                        let name: Option<String> = r.try_get("name").ok();
                        let total_duration: Option<i64> = r.try_get("total_duration").ok();
                        if let (Some(n), Some(td)) = (name, total_duration) {
                            Some((n, td))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(InsightResult::TopN(results))
            }

            InsightType::MostActiveDay => {
                let rows = sqlx::query!(
                    "
                    SELECT DATE(datetime(timestamp, 'localtime')) as date,
                            SUM(duration) as total
                    FROM events
                    GROUP BY date
                    ORDER BY total DESC
                    LIMIT 1
                    "
                )
                .fetch_one(db_context.pool())
                .await?;

                Ok(InsightResult::MostActiveDay {
                    date: rows.date.unwrap_or_default(),
                    total_duration: rows.total.unwrap_or(0),
                })
            }

            InsightType::AggregatedAverage => {
                let mut where_clause = String::new();

                if let Some(year) = query.year {
                    where_clause.push_str(&format!(
                        "WHERE strftime('%Y', datetime(timestamp, 'localtime')) = '{}'",
                        year
                    ));
                }

                let query_string = format!(
                    "
                        SELECT strftime('%m', datetime(timestamp, 'localtime')) as bucket,
                                AVG(duration) as avg_duration
                        FROM events
                        {where_clause}
                        GROUP BY bucket
                        "
                );

                let rows = sqlx::query(&query_string)
                    .fetch_all(db_context.pool())
                    .await?;

                let result = rows
                    .into_iter()
                    .filter_map(|r| {
                        let bucket = r.try_get("bucket").ok();
                        let avg_duration = r.try_get("avg_duration").ok();

                        if let (Some(b), Some(avg)) = (bucket, avg_duration) {
                            Some((b, avg))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(InsightResult::AggregatedAverage(result))
            }
        }
    }
}
