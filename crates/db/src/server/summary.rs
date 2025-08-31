use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, Utc};
use common::{
    models::{
        inputs::{BucketSummaryInput, SummaryQueryInput},
        Group,
    },
    time::{TimeBucket, TimeRange},
};
use log::debug;

use crate::{
    models::{BucketTimeSummary, GroupedTimeSummary},
    server::utils::{
        query::{
            append_all_filters, append_date_range, append_group_by, append_standard_joins,
            get_time_bucket_expr, group_key_info,
        },
        summary_filter::SummaryFilters,
    },
    utils::DBError,
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
            builder = builder.start(start);
        }

        if let Some(end) = input.end {
            builder = builder.end(end);
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
        builder = builder.start(time_range.start());
        builder = builder.end(time_range.end());
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

    /// Executes a range summary query with the current filters.
    ///
    /// Returns aggregated durations grouped by the chosen `Group`, if any.
    pub async fn execute_range_summary(
        &self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, DBError> {
        let (group_key, inner_tbl) = group_key_info(self.filters.group_by);

        let mut base_query =
            format!("SELECT {group_key} as group_key, SUM(duration) as total_seconds FROM events");
        append_standard_joins(&mut base_query, inner_tbl);
        base_query.push_str(" WHERE 1=1");

        append_date_range(
            &mut base_query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );
        append_all_filters(&mut base_query, self.filters.clone());

        if self.filters.group_by.is_some() {
            append_group_by(&mut base_query, Some(group_key));
        }

        let final_query = base_query.clone();

        let records = sqlx::query_as::<_, GroupedTimeSummary>(&final_query)
            .fetch_all(db.pool())
            .await?;

        Ok(records)
    }

    /// Executes a query that returns only the total time (in seconds)
    /// for the current filters
    pub async fn execute_total_time(&self, db: &DBContext) -> Result<i64, DBError> {
        let mut query = String::from("SELECT SUM(duration) as total_seconds FROM events");
        append_standard_joins(&mut query, None);
        query.push_str(" WHERE 1=1");

        append_date_range(
            &mut query,
            self.filters.start,
            self.filters.end,
            "events.timestamp",
            "events.end_timestamp",
        );

        debug!(
            "The filter start: {:?}. The filter end: {:?}",
            self.filters.start, self.filters.end
        );
        append_all_filters(&mut query, self.filters.clone());

        let result = sqlx::query_scalar::<_, Option<i64>>(&query)
            .fetch_one(db.pool())
            .await?;

        Ok(result.unwrap_or(0))
    }

    /// Executes a bucketed range summary, aggregating durations into
    /// time buckets (e.g. by day, week, or month) and grouping optionally.
    pub async fn execute_range_summary_with_bucket(
        &self,
        db: &DBContext,
    ) -> Result<Vec<BucketTimeSummary>, DBError> {
        debug!("The query builder: {:?}", self);
        let (group_key, inner_tbl) = group_key_info(self.filters.group_by);

        let time_bucket_expr = get_time_bucket_expr(self.filters.time_bucket);

        let needs_entity_type = matches!(self.filters.group_by, Some(Group::Entity));
        let meta_select = if needs_entity_type {
            ", entities.type AS group_meta "
        } else {
            " , NULL AS group_meta "
        };

        let mut base_query = format!(
            "SELECT {time_bucket_expr} AS bucket, \
                    {group_key} AS group_key, \
                    SUM(duration) as total_seconds \
                    {meta_select} \
            FROM events",
        );
        append_standard_joins(&mut base_query, inner_tbl);
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

        debug!("The query: {}", base_query);

        let rows = sqlx::query_as::<_, RawBucketRow>(&base_query)
            .fetch_all(db.pool())
            .await?;

        let mut records = Vec::new();

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
