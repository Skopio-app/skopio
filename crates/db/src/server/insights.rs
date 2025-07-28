use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::NaiveDate;
use common::{
    models::{outputs::InsightResult, Group, InsightBucket, InsightType},
    time::InsightRange,
};
use log::info;
use sqlx::Row;

use crate::DBContext;

#[derive(sqlx::FromRow)]
struct YearResult {
    year: Option<String>,
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

#[derive(Debug)]
pub struct InsightQuery {
    pub insight_type: InsightType,
    pub insight_range: Option<InsightRange>,
    pub group_by: Option<Group>,
    pub limit: Option<usize>,
    pub bucket: Option<InsightBucket>,
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

            InsightType::TopN => {
                let Some(InsightRange { start, end, .. }) = query.insight_range else {
                    return Err(sqlx::Error::Protocol("insight_range is required".into()));
                };

                let Some(group_by) = query.group_by else {
                    return Err(sqlx::Error::Protocol(
                        "group_by is required for TopN".into(),
                    ));
                };

                let Some(limit) = query.limit else {
                    return Err(sqlx::Error::Protocol("limit is required for TopN".into()));
                };

                let (_field, join) = match group_by {
                    Group::Project => ("project_id", "JOIN projects p ON p.id = e.project_id"),
                    Group::App => ("app_id", "JOIN apps a ON a.id = e.app_id"),
                    Group::Category => ("category_id", "JOIN categories c ON c.id = e.category_id"),
                    Group::Branch => ("branch_id", "JOIN branches b ON b.id = e.branch_id"),
                    Group::Entity => ("entity_id", "JOIN entities en ON en.id = e.entity_id"),
                    Group::Language => ("language_id", "JOIN languages l ON l.id = e.language_id"),
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
                        WHERE e.timestamp >= ? AND e.timestamp < ?
                        GROUP BY {label}
                        ORDER BY total_duration DESC
                        LIMIT ?
                        "
                );

                let rows = sqlx::query(&query_string)
                    .bind(start)
                    .bind(end)
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
                let Some(InsightRange { start, end, bucket }) = query.insight_range else {
                    return Err(sqlx::Error::Protocol("insight_range is required".into()));
                };

                match bucket {
                    Some(InsightBucket::Year | InsightBucket::Month | InsightBucket::Week) => {}
                    _ => {
                        return Err(sqlx::Error::Protocol(
                            "
                        Only year, month, or week formats are allowed for MostActiveDay"
                                .into(),
                        ));
                    }
                }

                let rows = sqlx::query!(
                    "
                    SELECT DATE(datetime(timestamp, 'localtime')) as date,
                            SUM(duration) as total
                    FROM events
                    WHERE timestamp >= ? AND timestamp < ?
                    GROUP BY date
                    ORDER BY total DESC
                    LIMIT 1
                    ",
                    start,
                    end
                )
                .fetch_optional(db_context.pool())
                .await?;

                if let Some(row) = rows {
                    Ok(InsightResult::MostActiveDay {
                        date: row.date.unwrap_or_default(),
                        total_duration: row.total.unwrap_or(0),
                    })
                } else {
                    Ok(InsightResult::MostActiveDay {
                        date: "".into(),
                        total_duration: 0,
                    })
                }
            }

            InsightType::AggregatedAverage => {
                info!("The query: {:?}", query);

                let Some(InsightRange { start, end, bucket }) = query.insight_range else {
                    return Err(sqlx::Error::Protocol("insight_range is required".into()));
                };

                let Some(bucket) = bucket else {
                    return Err(sqlx::Error::Protocol("bucket is required".into()));
                };

                let bucket_format = match bucket {
                    InsightBucket::Day => "%Y-%m-%d",
                    InsightBucket::Month => "%Y-%m",
                    _ => {
                        return Err(sqlx::Error::Protocol(
                            "Only day or month is supported".into(),
                        ));
                    }
                };

                let (join_clause, label_select, group_by_clause) = match query.group_by {
                    Some(Group::App) => (
                        "JOIN apps a ON a.id = e.app_id",
                        ", a.name as label",
                        ", label",
                    ),
                    Some(Group::Project) => (
                        "JOIN projects p ON p.id = e.project_id",
                        ", p.name as label",
                        ", label",
                    ),
                    Some(Group::Category) => (
                        "JOIN categories c ON c.id = e.category_id",
                        ", c.name as label",
                        ", label",
                    ),
                    Some(Group::Branch) => (
                        "JOIN branches b ON b.id = e.branch_id",
                        ", b.name as label",
                        ", label",
                    ),
                    Some(Group::Entity) => (
                        "JOIN entities en ON en.id = e.entity_id",
                        ", en.name as label",
                        ", label",
                    ),
                    Some(Group::Language) => (
                        "JOIN languages l ON l.id = e.language_id",
                        ", l.name as label",
                        ", label",
                    ),
                    None => ("", ", '_' as label", ""),
                };

                let sql = format!(
                    "
                    SELECT
                        strftime('{bucket_format}', datetime(e.timestamp, 'localtime')) as bucket,
                        ROUND(AVG(e.duration), 2) as avg_duration
                        {label_select}
                    FROM events e
                    {join_clause}
                    WHERE timestamp >= ? AND timestamp < ?
                    GROUP BY bucket{group_by_clause}
                    ORDER BY bucket
                    "
                );

                let rows = sqlx::query(&sql)
                    .bind(start)
                    .bind(end)
                    .fetch_all(db_context.pool())
                    .await?;

                let mut map: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();

                for row in rows {
                    let bucket_str: Option<String> = row.try_get("bucket").ok();
                    let avg: Option<f64> = row.try_get("avg_duration").ok();
                    let label: Option<String> = row.try_get("label").ok();

                    if let (Some(bucket_str), Some(avg_duration), Some(group)) =
                        (bucket_str, avg, label)
                    {
                        let label_key = match bucket {
                            InsightBucket::Day => {
                                NaiveDate::parse_from_str(&bucket_str, "%Y-%m-%d")
                                    .ok()
                                    .map(|d| d.format("%a").to_string())
                            }
                            InsightBucket::Month => {
                                NaiveDate::parse_from_str(&format!("{}-01", bucket_str), "%Y-%m-%d")
                                    .ok()
                                    .map(|d| d.format("%b").to_string())
                            }
                            _ => None,
                        };

                        if let Some(label) = label_key {
                            map.entry(label).or_default().push((group, avg_duration));
                        }
                    }
                }

                Ok(InsightResult::AggregatedAverage(map))
            }
        }
    }
}
