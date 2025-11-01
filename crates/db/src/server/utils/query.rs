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
    fn push_bucket_label_expr(&mut self, bucket: Option<TimeBucket>);
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

    /// Formats a SQLite-compatible time bucket expression based on the bucket type.
    fn push_bucket_label_expr(&mut self, bucket: Option<TimeBucket>) {
        match bucket {
            Some(TimeBucket::Hour) => self.push(
                "strftime('%Y-%m-%d %H:00:00', datetime(buckets.start_ts,'unixepoch','localtime'))",
            ),
            Some(TimeBucket::Day) => self
                .push("strftime('%Y-%m-%d', datetime(buckets.start_ts,'unixepoch','localtime'))"),
            Some(TimeBucket::Week) => {
                self.push("strftime('%Y-W%W', datetime(buckets.start_ts,'unixepoch','localtime'))")
            }
            Some(TimeBucket::Month) => {
                self.push("strftime('%Y-%m', datetime(buckets.start_ts,'unixepoch','localtime'))")
            }
            Some(TimeBucket::Year) => {
                self.push("strftime('%Y', datetime(buckets.start_ts,'unixepoch','localtime'))")
            }
            None => self.push("'Unbucketed'"),
        };
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

/// Appends an SQL expression that computes the time overlap (in seconds)
/// between an event’s active range (`events.timestamp` .. `events.end_timestamp`)
/// and a given interval defined by `push_start` and `push_end`.
/// The expression yields `max(0, min(end_event, end_range) - max(start_event, start_range))`,
/// ensuring only positive (actual overlapping) durations are counted.
pub fn push_overlap_with<FStart, FEnd>(
    qb: &mut QueryBuilder<Sqlite>,
    push_start: FStart,
    push_end: FEnd,
) where
    FStart: FnOnce(&mut QueryBuilder<Sqlite>),
    FEnd: FnOnce(&mut QueryBuilder<Sqlite>),
{
    qb.push("max(0, min(events.end_timestamp, ");
    push_end(qb);
    qb.push(") - max(events.timestamp, ");
    push_start(qb);
    qb.push("))");
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

/// Appends an SQL expression that computes the next bucket’s end timestamp
/// based on a given start (pushed by `push_start`) and a `BucketStep`.
/// Supports both fixed‐interval (seconds) and calendar‐aligned (`+1 month`, `+1 year`, etc.) steps.
pub fn push_next_end_with<F>(qb: &mut QueryBuilder<Sqlite>, push_start: F, step: &BucketStep)
where
    F: FnOnce(&mut QueryBuilder<Sqlite>),
{
    match step {
        BucketStep::Seconds(n) => {
            push_start(qb);
            qb.push(" + ").push(*n);
        }
        BucketStep::Calendar(modif) => {
            qb.push("strftime('%s', datetime(");
            push_start(qb);
            qb.push(", 'unixepoch', 'localtime', '")
                .push(modif)
                .push("'))");
        }
    }
}
