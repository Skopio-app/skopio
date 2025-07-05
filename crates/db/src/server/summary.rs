use std::{collections::HashMap, fmt::Debug, time::Instant};

use chrono::{DateTime, Utc};
use common::{models::inputs::Group, time::TimeBucket};
use log::info;

use crate::{
    models::{BucketTimeSummary, GroupedTimeSummary},
    server::utils::{
        query::{
            append_all_filters, append_date_range, append_group_by, append_standard_joins,
            convert_utc_bucket_to_local, get_time_bucket_expr, group_key_column,
        },
        summary_filter::SummaryFilters,
    },
    DBContext,
};

pub trait SummaryQueryParams {
    fn filters(&self) -> &SummaryFilters;
}

pub struct SummaryQueryBuilder {
    filters: SummaryFilters,
    group_by: Option<Group>,
    include_afk: bool,
    time_bucket: Option<TimeBucket>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawBucketRow {
    bucket: String,
    group_key: String,
    total_seconds: i64,
}

impl SummaryQueryParams for SummaryQueryBuilder {
    fn filters(&self) -> &SummaryFilters {
        &self.filters
    }
}

impl SummaryQueryBuilder {
    pub fn new() -> Self {
        Self {
            filters: SummaryFilters::default(),
            group_by: None,
            include_afk: false,
            time_bucket: None,
        }
    }

    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.filters.start = Some(start);
        self
    }

    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.filters.end = Some(end);
        self
    }

    pub fn apps(mut self, apps: Vec<String>) -> Self {
        self.filters.apps = Some(apps);
        self
    }

    pub fn projects(mut self, projects: Vec<String>) -> Self {
        self.filters.projects = Some(projects);
        self
    }

    pub fn categories(mut self, types: Vec<String>) -> Self {
        self.filters.categories = Some(types);
        self
    }

    pub fn entities(mut self, entities: Vec<String>) -> Self {
        self.filters.entities = Some(entities);
        self
    }

    pub fn branches(mut self, branches: Vec<String>) -> Self {
        self.filters.branches = Some(branches);
        self
    }

    pub fn languages(mut self, langs: Vec<String>) -> Self {
        self.filters.languages = Some(langs);
        self
    }

    pub fn group_by(mut self, field: Group) -> Self {
        self.group_by = Some(field);
        self
    }

    pub fn include_afk(mut self, include: bool) -> Self {
        self.include_afk = include;
        self
    }

    pub fn time_bucket(mut self, bucket: TimeBucket) -> Self {
        self.time_bucket = Some(bucket);
        self
    }

    pub async fn execute_grouped_summary_by(
        self,
        db: &DBContext,
        group_key_field: Group,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.group_by(group_key_field)
            .execute_range_summary(db)
            .await
    }

    pub async fn execute_range_summary(
        &self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        let start_time = Instant::now();
        let group_key = group_key_column(self.group_by);

        let mut base_query =
            format!("SELECT {group_key} as group_key, SUM(duration) as total_seconds FROM events");
        append_standard_joins(&mut base_query);
        base_query.push_str(" WHERE 1=1");

        append_date_range(
            &mut base_query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );
        append_all_filters(&mut base_query, self.filters.clone());

        if self.group_by.is_some() {
            append_group_by(&mut base_query, Some(group_key));
        }

        let mut final_query = base_query.clone();

        if self.include_afk {
            let mut afk_query = String::from("SELECT 'AFK' as group_key, SUM(duration) as total_seconds FROM afk_events WHERE 1=1");

            if let Some(start) = self.filters.start {
                afk_query.push_str(" AND afk_start >= '");
                afk_query.push_str(&start.to_string());
                afk_query.push('\'');
            }

            if let Some(end) = self.filters.end {
                afk_query.push_str(" AND afk_end <= '");
                afk_query.push_str(&end.to_string());
                afk_query.push('\'');
            }

            final_query = format!("{} UNION ALL {}", base_query, afk_query);
        }

        let records = sqlx::query_as::<_, GroupedTimeSummary>(&final_query)
            .fetch_all(db.pool())
            .await?;
        let elapsed = start_time.elapsed();
        info!(
            "Executed range summary SQL in {:.2?} - {} rows (group_by: {:?})",
            elapsed,
            records.len(),
            self.group_by,
        );

        Ok(records)
    }

    pub async fn execute_total_time(&self, db: &DBContext) -> Result<i64, sqlx::Error> {
        let start_time = Instant::now();
        let mut query = String::from("SELECT SUM(duration) as total_seconds FROM events");
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

        let result = sqlx::query_scalar::<_, Option<i64>>(&query)
            .fetch_one(db.pool())
            .await?;

        let elapsed = start_time.elapsed();
        info!(
            "Executed total time query in {:.2?} - {:?} (group_by: {:?})",
            elapsed, result, self.group_by,
        );

        Ok(result.unwrap_or(0))
    }

    pub async fn execute_range_summary_with_bucket(
        &self,
        db: &DBContext,
    ) -> Result<Vec<BucketTimeSummary>, sqlx::Error> {
        let start_time = Instant::now();
        let group_key = group_key_column(self.group_by);

        let time_bucket_expr = get_time_bucket_expr(self.time_bucket);

        let mut base_query = format!(
            "SELECT {time_bucket_expr} AS bucket, {group_key} AS group_key, SUM(duration) as total_seconds
            FROM events",
        );
        append_standard_joins(&mut base_query);
        base_query.push_str(" WHERE 1=1");

        append_date_range(
            &mut base_query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );
        append_all_filters(&mut base_query, self.filters.clone());

        base_query.push_str(&format!(" GROUP BY {time_bucket_expr}, {group_key}"));

        let rows = sqlx::query_as::<_, RawBucketRow>(&base_query)
            .fetch_all(db.pool())
            .await?;

        let mut grouped_map: HashMap<String, HashMap<String, i64>> = HashMap::new();

        for row in rows {
            grouped_map
                .entry(row.bucket)
                .or_default()
                .insert(row.group_key, row.total_seconds);
        }

        let records = grouped_map
            .into_iter()
            .map(|(bucket, grouped_values)| BucketTimeSummary {
                bucket: convert_utc_bucket_to_local(&bucket),
                grouped_values,
            })
            .collect::<Vec<_>>();

        let elapsed = start_time.elapsed();
        info!(
            "Executed range summary with bucket query in {:.2?} - {} rows (group_key: {:?})",
            elapsed,
            records.len(),
            self.group_by,
        );

        Ok(records)
    }
}

impl Default for SummaryQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
