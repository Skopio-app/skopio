use common::{models::Group, time::TimeBucket};
use sqlx::{QueryBuilder, Sqlite};

use crate::server::utils::summary_filter::SummaryFilters;

/// Appends a date range filter to the query using the specified start and end field names.
/// Dates are formatted as RFC3339 (ISO 8601) to ensure proper string comparison.
pub fn append_date_range(
    qb: &mut QueryBuilder<Sqlite>,
    start: Option<i64>,
    end: Option<i64>,
    start_field: &str,
    end_field: &str,
) {
    if start.is_some() || end.is_some() {
        qb.push(" AND (1=1");
        if let Some(s) = start {
            qb.push(" AND ").push(end_field).push(" > ").push_bind(s);
        }
        if let Some(e) = end {
            qb.push(" AND ").push(start_field).push(" < ").push_bind(e);
        }
        qb.push(")");
    }
}

/// Appends an `IN (...)` filter to the query if the list is not empty.
pub fn append_filter_list<'qb>(
    qb: &mut QueryBuilder<'qb, Sqlite>,
    field: &str,
    values: &'qb [String],
) {
    if values.is_empty() {
        return;
    }

    qb.push(" AND ").push(field).push(" IN (");
    {
        let mut sep = qb.separated(", ");
        for v in values {
            sep.push_bind(v);
        }
    }
    qb.push(")");
}

/// Appends a full set of optional filters (apps, projects, categories, etc.)
/// using the appropriate field names.
pub fn append_all_filters<'qb>(qb: &mut QueryBuilder<'qb, Sqlite>, filters: &'qb SummaryFilters) {
    if let Some(apps) = &filters.apps {
        append_filter_list(qb, "apps.name", apps);
    }
    if let Some(projects) = &filters.projects {
        append_filter_list(qb, "projects.name", projects);
    }
    if let Some(categories) = &filters.categories {
        append_filter_list(qb, "categories.name", categories);
    }
    if let Some(branches) = &filters.branches {
        append_filter_list(qb, "branches.name", branches);
    }
    if let Some(entities) = &filters.entities {
        append_filter_list(qb, "entities.name", entities);
    }
    if let Some(languages) = &filters.languages {
        append_filter_list(qb, "languages.name", languages);
    }
}

/// Appends JOIN clauses for events to resolve all foreign keys.
/// `inner_join` indicates which related table (if any) should be INNER JOINed
/// instead of LEFT JOINed (i.e., the table that supplies the group_key).
pub fn append_standard_joins(qb: &mut QueryBuilder<Sqlite>, inner_join: Option<&str>) {
    let j = |tbl: &str| {
        if inner_join == Some(tbl) {
            " JOIN "
        } else {
            " LEFT JOIN "
        }
    };

    qb.push(j("apps"))
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
