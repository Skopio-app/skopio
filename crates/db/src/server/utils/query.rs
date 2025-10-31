use common::{models::Group, time::TimeBucket};
use sqlx::{QueryBuilder, Sqlite};

use crate::server::utils::summary_filter::SummaryFilters;

pub enum BucketStep {
    Seconds(i64),           // for Hour/Day (fixed length)
    Calendar(&'static str), // SQLite date modifier "+7 days" | "+1 month" | "+1 year"
}

pub trait QueryBuilderExt<'qb> {
    fn append_date_range(
        &mut self,
        start: Option<i64>,
        end: Option<i64>,
        start_field: &str,
        end_field: &str,
    );

    fn append_filter_list(&mut self, field: &str, values: &'qb [String]);
    fn append_all_filters(&mut self, filters: &'qb SummaryFilters);
    fn append_standard_joins(&mut self, inner_join: Option<&str>);
}

impl<'qb> QueryBuilderExt<'qb> for QueryBuilder<'qb, Sqlite> {
    /// Appends a date range filter to the query using the specified start and end field names.
    /// Dates are formatted as RFC3339 (ISO 8601) to ensure proper string comparison.
    fn append_date_range(
        &mut self,
        start: Option<i64>,
        end: Option<i64>,
        start_field: &str,
        end_field: &str,
    ) {
        if start.is_some() || end.is_some() {
            self.push(" AND (1=1");
            if let Some(s) = start {
                self.push(" AND ").push(end_field).push(" > ").push_bind(s);
            }
            if let Some(e) = end {
                self.push(" AND ")
                    .push(start_field)
                    .push(" < ")
                    .push_bind(e);
            }
            self.push(")");
        }
    }

    /// Appends an `IN (...)` filter to the query if the list is not empty.
    fn append_filter_list(&mut self, field: &str, values: &'qb [String]) {
        if values.is_empty() {
            return;
        }

        self.push(" AND ").push(field).push(" IN (");
        {
            let mut sep = self.separated(", ");
            for v in values {
                sep.push_bind(v);
            }
        }
        self.push(")");
    }

    /// Appends a full set of optional filters (apps, projects, categories, etc.)
    /// using the appropriate field names.
    fn append_all_filters(&mut self, filters: &'qb SummaryFilters) {
        if let Some(apps) = &filters.apps {
            self.append_filter_list("apps.name", apps);
        }
        if let Some(projects) = &filters.projects {
            self.append_filter_list("projects.name", projects);
        }
        if let Some(categories) = &filters.categories {
            self.append_filter_list("categories.name", categories);
        }
        if let Some(branches) = &filters.branches {
            self.append_filter_list("branches.name", branches);
        }
        if let Some(entities) = &filters.entities {
            self.append_filter_list("entities.name", entities);
        }
        if let Some(languages) = &filters.languages {
            self.append_filter_list("languages.name", languages);
        }
    }

    /// Appends JOIN clauses for events to resolve all foreign keys.
    /// `inner_join` indicates which related table (if any) should be INNER JOINed
    /// instead of LEFT JOINed (i.e., the table that supplies the group_key).
    fn append_standard_joins(&mut self, inner_join: Option<&str>) {
        let j = |tbl: &str| {
            if inner_join == Some(tbl) {
                " JOIN "
            } else {
                " LEFT JOIN "
            }
        };

        self.push(j("apps"))
            .push("apps ON events.app_id = apps.id")
            .push(j("projects"))
            .push("projects ON events.project_id = projects.id")
            .push(j("entities"))
            .push("entities ON events.entity_id = entities.id")
            .push(j("branches"))
            .push("branches ON events.branch_id = branches.id")
            .push(j("categories"))
            .push("categories ON events.category_id = categories.id")
            .push(j("languages"))
            .push("languages ON events.language_id = languages.id")
            .push(j("sources"))
            .push("sources ON events.source_id = sources.id");
    }
}

/// Returns (group_key_sql, inner_join_table_name)
pub fn group_key_info(group: Option<Group>) -> (&'static str, Option<&'static str>) {
    match group {
        Some(Group::App) => ("apps.name", Some("apps")),
        Some(Group::Project) => ("projects.name", Some("projects")),
        Some(Group::Entity) => ("entities.name", Some("entities")),
        Some(Group::Branch) => ("branches.name", Some("branches")),
        Some(Group::Category) => ("categories.name", Some("categories")),
        Some(Group::Language) => ("languages.name", Some("languages")),
        Some(Group::Source) => ("sources.name", Some("sources")),
        None => ("'Total'", None),
    }
}

/// Formats a SQLite-compatible time bucket expression based on the bucket type.
pub fn get_time_bucket_expr(bucket: Option<TimeBucket>) -> &'static str {
    match bucket {
        Some(TimeBucket::Hour) => {
            "strftime('%Y-%m-%d %H:%M:%S', datetime(events.timestamp, 'unixepoch', 'localtime'))"
        }
        Some(TimeBucket::Day) => {
            "strftime('%Y-%m-%d', datetime(events.timestamp, 'unixepoch', 'localtime'))"
        }
        Some(TimeBucket::Week) => {
            "strftime('%Y-W%W', datetime(events.timestamp, 'unixepoch', 'localtime'))"
        }
        Some(TimeBucket::Month) => {
            "strftime('%Y-%m', datetime(events.timestamp, 'unixepoch', 'localtime'))"
        }
        Some(TimeBucket::Year) => {
            "strftime('%Y', datetime(events.timestamp, 'unixepoch', 'localtime'))"
        }
        None => "'Unbucketed'",
    }
}

pub fn push_overlap_bind(qb: &mut QueryBuilder<Sqlite>, start: i64, end: i64) {
    qb.push("max(0, min(events.end_timestamp, ")
        .push_bind(end)
        .push(") - max(events.timestamp, ")
        .push_bind(start)
        .push("))");
}

pub fn push_overlap_expr<'qb>(
    qb: &mut QueryBuilder<'qb, Sqlite>,
    start_expr: &'qb str,
    end_expr: &'qb str,
) {
    qb.push("max(0, min(events.end_timestamp, ")
        .push(end_expr)
        .push(") - max(events.timestamp, ")
        .push(start_expr)
        .push("))");
}

pub fn bucket_step(bucket: Option<TimeBucket>) -> BucketStep {
    match bucket {
        Some(TimeBucket::Hour) => BucketStep::Seconds(3600),
        Some(TimeBucket::Day) => BucketStep::Seconds(86_400),
        Some(TimeBucket::Week) => BucketStep::Calendar("+7 days"),
        Some(TimeBucket::Month) => BucketStep::Calendar("+1 month"),
        Some(TimeBucket::Year) => BucketStep::Calendar("+1 year"),
        None => BucketStep::Seconds(86_400),
    }
}

pub fn push_bucket_label_expr(qb: &mut QueryBuilder<Sqlite>, bucket: Option<TimeBucket>) {
    match bucket {
        Some(TimeBucket::Hour) => qb.push(
            "strftime('%Y-%m-%d %H:00:00', datetime(buckets.start_ts,'unixepoch','localtime'))",
        ),
        Some(TimeBucket::Day) => {
            qb.push("strftime('%Y-%m-%d', datetime(buckets.start_ts,'unixepoch','localtime'))")
        }
        Some(TimeBucket::Week) => {
            qb.push("strftime('%Y-W%W', datetime(buckets.start_ts,'unixepoch','localtime'))")
        }
        Some(TimeBucket::Month) => {
            qb.push("strftime('%Y-%m', datetime(buckets.start_ts,'unixepoch','localtime'))")
        }
        Some(TimeBucket::Year) => {
            qb.push("strftime('%Y', datetime(buckets.start_ts,'unixepoch','localtime'))")
        }
        None => qb.push("'Unbucketed'"),
    };
}

/// Pushes an expression that computes the next bucket end from a *column/expression*.
/// Example output (Month): strftime('%s', datetime(buckets.start_ts,'unixepoch','localtime','+1 month'))
pub fn push_next_end_from_expr(qb: &mut QueryBuilder<Sqlite>, start_expr: &str, step: &BucketStep) {
    match step {
        BucketStep::Seconds(n) => {
            qb.push(start_expr).push(" + ").push(*n);
        }
        BucketStep::Calendar(modif) => {
            qb.push("strftime('%s', datetime(")
                .push(start_expr)
                .push(", 'unixepoch', 'localtime', '")
                .push(modif)
                .push("'))'");
        }
    }
}

/// Pushes an expression that computes the next bucket end from a *bound value*.
/// Example output (Month): strftime('%s', datetime(?,'unixepoch','localtime','+1 month'))
pub fn push_next_end_from_bind(qb: &mut QueryBuilder<Sqlite>, start_bind: i64, step: &BucketStep) {
    match step {
        BucketStep::Seconds(n) => {
            qb.push_bind(start_bind).push(" + ").push(*n);
        }
        BucketStep::Calendar(modif) => {
            qb.push("strftime('%s', datetime(")
                .push_bind(start_bind)
                .push(", 'unixepoch', 'localtime', '")
                .push(modif)
                .push("'))'");
        }
    }
}
