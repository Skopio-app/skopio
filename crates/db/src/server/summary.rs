use std::{collections::HashMap, fmt::Debug};

use common::{
    models::{
        inputs::{BucketSummaryInput, SummaryQueryInput},
        Group,
    },
    time::{TimeBucket, TimeRange},
};
use sqlx::{QueryBuilder, Sqlite};

use crate::{
    error::DBError,
    models::BucketTimeSummary,
    server::utils::{
        query::{
            append_all_filters, append_date_range, append_standard_joins, bucket_step_seconds,
            group_key_info, push_bucket_label_expr, push_overlap_bind, push_overlap_expr,
        },
        summary_filter::SummaryFilters,
    },
    DBContext,
};

pub trait SummaryQueryParams {
    fn filters(&self) -> &SummaryFilters;
}

/// Builder for constructing and executing time summary queries against the database.
///
/// The builder pattern is used to incrementally add filters (apps, projects, categories, etc.)
/// and execution methods run parameterized SQL queries to return summarized time data.
#[derive(Debug)]
pub struct SummaryQueryBuilder {
    pub filters: SummaryFilters,
}

impl From<SummaryQueryInput> for SummaryQueryBuilder {
    fn from(input: SummaryQueryInput) -> Self {
        let mut builder = SummaryQueryBuilder::new();

        if let Some(start) = input.start {
            builder = builder.start(start.timestamp());
        }

        if let Some(end) = input.end {
            builder = builder.end(end.timestamp());
        }

        if let Some(apps) = input.apps {
            builder = builder.apps(apps);
        }

        if let Some(projects) = input.projects {
            builder = builder.projects(projects);
        }

        if let Some(types) = input.categories {
            builder = builder.categories(types);
        }

        if let Some(entities) = input.entities {
            builder = builder.entities(entities);
        }

        if let Some(branches) = input.branches {
            builder = builder.branches(branches);
        }

        if let Some(langs) = input.languages {
            builder = builder.languages(langs);
        }
        builder
    }
}

impl From<BucketSummaryInput> for SummaryQueryBuilder {
    fn from(input: BucketSummaryInput) -> Self {
        let mut builder = SummaryQueryBuilder::new();

        let time_range = TimeRange::from(input.preset);
        builder = builder.start(time_range.start().timestamp());
        builder = builder.end(time_range.end().timestamp());
        builder.filters.time_bucket = time_range.bucket();

        if let Some(apps) = input.apps {
            builder = builder.apps(apps);
        }

        if let Some(projects) = input.projects {
            builder = builder.projects(projects);
        }

        if let Some(categories) = input.categories {
            builder = builder.categories(categories)
        }

        if let Some(entities) = input.entities {
            builder = builder.entities(entities);
        }

        if let Some(branches) = input.branches {
            builder = builder.branches(branches);
        }

        if let Some(langs) = input.languages {
            builder = builder.languages(langs);
        }

        if let Some(group) = input.group_by {
            builder = builder.group_by(group);
        }

        builder
    }
}

/// Raw row returned when querying bucketed summaries.
/// Used internally before mapping into `BucketTimeSummary`
#[derive(Debug, sqlx::FromRow)]
struct RawBucketRow {
    /// The time bucket label (e.g., "2025-08-01")
    bucket: String,
    /// The grouping key value (e.g., project name, app name)
    group_key: String,
    /// Total time in seconds aggregated for this bucket + group_key
    total_seconds: i64,
    // Extra metadata when grouping by entity (entity type)
    group_meta: Option<String>,
}

impl SummaryQueryParams for SummaryQueryBuilder {
    fn filters(&self) -> &SummaryFilters {
        &self.filters
    }
}

impl Default for SummaryQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SummaryQueryBuilder {
    pub fn new() -> Self {
        Self {
            filters: SummaryFilters::default(),
        }
    }

    pub fn start(mut self, start: i64) -> Self {
        self.filters.start = Some(start);
        self
    }

    pub fn end(mut self, end: i64) -> Self {
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

    pub fn categories(mut self, categories: Vec<String>) -> Self {
        self.filters.categories = Some(categories);
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
        self.filters.group_by = Some(field);
        self
    }

    pub fn time_bucket(mut self, bucket: TimeBucket) -> Self {
        self.filters.time_bucket = Some(bucket);
        self
    }

    /// Executes a query that returns only the total time (in seconds)
    /// for the current filters
    pub async fn execute_total_time(&self, db: &DBContext) -> Result<i64, DBError> {
        let start = self.filters.start.unwrap_or(i64::MIN);
        let end = self.filters.end.unwrap_or(i64::MAX);

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
        push_overlap_bind(&mut qb, start, end);
        qb.push(") AS total_seconds FROM events ");

        append_standard_joins(&mut qb, None);
        qb.push(" WHERE 1=1");

        append_date_range(
            &mut qb,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );

        append_all_filters(&mut qb, &self.filters);

        let query = qb.build_query_scalar::<Option<i64>>();
        let result = query.fetch_one(db.pool()).await?;

        Ok(result.unwrap_or(0))
    }

    /// Executes a bucketed range summary, aggregating durations into
    /// time buckets (e.g. by day, week, or month) and grouping optionally.
    pub async fn execute_range_summary_with_bucket(
        &self,
        db: &DBContext,
    ) -> Result<Vec<BucketTimeSummary>, DBError> {
        let (group_key, inner_tbl) = group_key_info(self.filters.group_by);
        let needs_entity_type = matches!(self.filters.group_by, Some(Group::Entity));

        let range_start = self.filters.start.unwrap_or(i64::MIN);
        let range_end = self.filters.end.unwrap_or(i64::MAX);
        let step = bucket_step_seconds(self.filters.time_bucket);

        let mut qb = QueryBuilder::<Sqlite>::new(
            "WITH RECURSIVE buckets(start_ts, end_ts) AS ( \
             SELECT ",
        );
        qb.push_bind(range_start)
            .push(", MIN(")
            .push_bind(range_end)
            .push(", ")
            .push_bind(range_start)
            .push(" + ")
            .push(step)
            .push(
                ") \
         UNION ALL \
             SELECT end_ts, MIN(",
            )
            .push_bind(range_end)
            .push(", end_ts + ")
            .push(step)
            .push(
                ") \
             FROM buckets \
             WHERE end_ts < ",
            )
            .push_bind(range_end)
            .push(") ");

        qb.push("SELECT ");
        push_bucket_label_expr(&mut qb, self.filters.time_bucket);
        qb.push(" AS bucket, ")
            .push(group_key)
            .push(" AS group_key, ")
            .push("SUM(");
        push_overlap_expr(&mut qb, "buckets.start_ts", "buckets.end_ts");
        qb.push(") AS total_seconds");

        if needs_entity_type {
            qb.push(", entities.type AS group_meta ");
        } else {
            qb.push(", NULL AS group_meta ");
        }

        qb.push(
            " FROM buckets \
              JOIN events \
                ON events.end_timestamp > buckets.start_ts \
               AND events.timestamp    < buckets.end_ts ",
        );

        append_standard_joins(&mut qb, inner_tbl);

        qb.push(" WHERE 1=1");

        append_date_range(
            &mut qb,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );

        append_all_filters(&mut qb, &self.filters);

        qb.push(" GROUP BY ");
        push_bucket_label_expr(&mut qb, self.filters.time_bucket);
        qb.push(", ").push(group_key);

        let rows = qb
            .build_query_as::<RawBucketRow>()
            .fetch_all(db.pool())
            .await?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut grouped_values = HashMap::new();
            grouped_values.insert(row.group_key, row.total_seconds);
            records.push(BucketTimeSummary {
                bucket: row.bucket,
                grouped_values,
                group_meta: row.group_meta,
            });
        }

        Ok(records)
    }
}
