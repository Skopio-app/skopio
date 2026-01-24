use std::collections::BTreeMap;

use async_trait::async_trait;
use common::{
    models::{outputs::InsightResult, Group, InsightBucket, InsightType},
    time::insight::InsightRange,
};
use sqlx::{QueryBuilder, Sqlite};

use crate::{
    error::DBError,
    server::utils::query::{
        bucket_step, group_key_info, push_next_end_with, BucketStep, QueryBuilderExt,
    },
    DBContext,
};

#[derive(sqlx::FromRow)]
struct YearResult {
    year: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TopNRow {
    label: String,
    total_duration: i64,
}

#[derive(sqlx::FromRow)]
struct MostActiveDayRow {
    date: String,
    total: i64,
}

#[derive(sqlx::FromRow)]
struct AvgRow {
    key: i64,
    label: String,
    avg_duration: f64,
}

#[async_trait]
pub trait InsightProvider {
    async fn execute(db_context: &DBContext, query: InsightQuery)
        -> Result<InsightResult, DBError>;
}

#[derive(Debug, Clone)]
pub struct Insights;

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
    ) -> Result<InsightResult, DBError> {
        match query.insight_type {
            InsightType::ActiveYears => {
                let rows: Vec<YearResult> = sqlx::query_as!(
                    YearResult,
                    "
                    SELECT DISTINCT strftime('%Y', timestamp, 'unixepoch', 'localtime') as year
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
                    return Err(DBError::MissingField("insight_range"));
                };
                let Some(group_by) = query.group_by else {
                    return Err(DBError::MissingField("group_by"));
                };
                let Some(limit) = query.limit else {
                    return Err(DBError::MissingField("limit"));
                };

                let start_epoch = start.timestamp();
                let end_epoch = end.timestamp();

                let (group_key, inner_tbl) = group_key_info(Some(group_by));

                let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
                qb.push(group_key).push(" AS label, COALESCE(SUM(");
                qb.push_overlap_duration(&start_epoch.to_string(), &end_epoch.to_string());
                qb.push("), 0) AS total_duration FROM events ");

                qb.append_standard_joins(inner_tbl);
                qb.push(" WHERE 1=1");

                qb.append_date_range(
                    Some(start_epoch),
                    Some(end_epoch),
                    "events.timestamp",
                    "events.end_timestamp",
                );

                qb.push(" GROUP BY ").push(group_key);
                qb.push(" ORDER BY total_duration DESC LIMIT ");
                qb.push_bind(limit as i64);

                let rows: Vec<TopNRow> = qb.build_query_as().fetch_all(db_context.pool()).await?;

                let results = rows
                    .into_iter()
                    .map(|r| (r.label, r.total_duration))
                    .collect();

                Ok(InsightResult::TopN(results))
            }

            InsightType::MostActiveDay => {
                let Some(InsightRange { start, end }) = query.insight_range else {
                    return Err(DBError::MissingField("insight_range"));
                };

                match query.bucket {
                    Some(InsightBucket::Year | InsightBucket::Month | InsightBucket::Week) => {}
                    _ => {
                        return Err(DBError::Unsupported(
                            "Only year, month, or week buckets are supported",
                        ));
                    }
                }

                let range_start = start.timestamp();
                let range_end = end.timestamp();

                let step = BucketStep::Seconds(86_400);

                let mut qb = QueryBuilder::<Sqlite>::new(
                    "WITH RECURSIVE buckets(start_ts, end_ts) AS ( SELECT ",
                );

                qb.push_bind(range_start)
                    .push(", MIN(")
                    .push_bind(range_end)
                    .push(", ");
                push_next_end_with(
                    &mut qb,
                    |q| {
                        q.push_bind(range_start);
                    },
                    &step,
                );
                qb.push(") UNION ALL SELECT end_ts, MIN(")
                    .push_bind(range_end)
                    .push(", ");
                push_next_end_with(
                    &mut qb,
                    |q| {
                        q.push("end_ts");
                    },
                    &step,
                );
                qb.push(") FROM buckets WHERE end_ts < ")
                    .push_bind(range_end)
                    .push(") ");

                qb.push(
                    "SELECT \
                        strftime('%Y-%m-%d', datetime(buckets.start_ts,'unixepoch','localtime')) AS date, \
                        COALESCE(SUM(",
                );
                qb.push_overlap_duration("buckets.start_ts", "buckets.end_ts");
                qb.push(
                    "), 0) AS total \
                     FROM buckets \
                     JOIN events ON events.end_timestamp > buckets.start_ts AND events.timestamp < buckets.end_ts \
                     WHERE 1=1",
                );

                qb.append_date_range(
                    Some(range_start),
                    Some(range_end),
                    "events.timestamp",
                    "events.end_timestamp",
                );

                qb.push(" GROUP BY date ORDER BY total DESC LIMIT 1");

                let row: Option<MostActiveDayRow> = qb
                    .build_query_as()
                    .fetch_optional(db_context.pool())
                    .await?;

                Ok(InsightResult::MostActiveDay {
                    date: row.as_ref().map(|r| r.date.clone()).unwrap_or_default(),
                    total_duration: row.as_ref().map(|r| r.total).unwrap_or(0),
                })
            }

            InsightType::AggregatedAverage => {
                let Some(InsightRange { start, end }) = query.insight_range else {
                    return Err(DBError::MissingField("insight_range"));
                };

                let Some(bucket) = query.bucket else {
                    return Err(DBError::MissingField("bucket"));
                };

                let range_start = start.timestamp();
                let range_end = end.timestamp();

                let (key_expr, step): (&'static str, BucketStep) = match bucket {
                    InsightBucket::Day => (
                        "CAST(strftime('%w', datetime(buckets.start_ts,'unixepoch','localtime')) AS INTEGER)",
                        BucketStep::Seconds(86_400),
                    ),
                    InsightBucket::Month => (
                        "CAST(strftime('%m', datetime(buckets.start_ts,'unixepoch','localtime')) AS INTEGER)",
                        bucket_step(Some(common::time::TimeBucket::Month)),
                    ),
                    _ => return Err(DBError::Unsupported("Only day or month is supported")),
                };

                let (group_key, inner_tbl) = group_key_info(query.group_by);

                let mut qb = QueryBuilder::<Sqlite>::new(
                    "WITH RECURSIVE buckets(start_ts, end_ts) AS ( SELECT ",
                );

                qb.push_bind(range_start)
                    .push(", MIN(")
                    .push_bind(range_end)
                    .push(", ");
                push_next_end_with(
                    &mut qb,
                    |q| {
                        q.push_bind(range_start);
                    },
                    &step,
                );
                qb.push(") UNION ALL SELECT end_ts, MIN(")
                    .push_bind(range_end)
                    .push(", ");
                push_next_end_with(
                    &mut qb,
                    |q| {
                        q.push("end_ts");
                    },
                    &step,
                );
                qb.push(") FROM buckets WHERE end_ts < ")
                    .push_bind(range_end)
                    .push(") ");

                qb.push(
                    "SELECT key, label, ROUND(AVG(total_seconds), 2) AS avg_duration \
                     FROM ( \
                        SELECT ",
                );
                qb.push(key_expr).push(" AS key, ");
                qb.push(group_key).push(" AS label, COALESCE(SUM(");
                qb.push_overlap_duration("buckets.start_ts", "buckets.end_ts");
                qb.push(
                    "), 0) AS total_seconds \
                        FROM buckets \
                        JOIN events ON events.end_timestamp > buckets.start_ts AND events.timestamp < buckets.end_ts ",
                );

                qb.append_standard_joins(inner_tbl);
                qb.push(" WHERE 1=1");
                qb.append_date_range(
                    Some(range_start),
                    Some(range_end),
                    "events.timestamp",
                    "events.end_timestamp",
                );

                qb.push(
                    " GROUP BY buckets.start_ts, label, key \
                     ) \
                     GROUP BY key, label \
                     ORDER BY key",
                );

                let rows: Vec<AvgRow> = qb.build_query_as().fetch_all(db_context.pool()).await?;

                const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                const MONTHS: [&str; 12] = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];

                let mut map: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();

                for r in rows {
                    let bucket_label = match bucket {
                        InsightBucket::Day => {
                            let idx = (r.key.clamp(0, 6)) as usize;
                            WEEKDAYS[idx].to_string()
                        }
                        InsightBucket::Month => {
                            let idx = (r.key.clamp(1, 12) - 1) as usize;
                            MONTHS[idx].to_string()
                        }
                        _ => unreachable!(),
                    };

                    map.entry(bucket_label)
                        .or_default()
                        .push((r.label, r.avg_duration));
                }

                Ok(InsightResult::AggregatedAverage(map))
            }
        }
    }
}
