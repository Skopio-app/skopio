use common::{models::Group, time::TimeBucket};

use crate::server::utils::summary_filter::SummaryFilters;

/// Appends a date range filter to the query using the specified start and end field names.
/// Dates are formatted as RFC3339 (ISO 8601) to ensure proper string comparison.
pub fn append_date_range(
    query: &mut String,
    start: Option<i64>,
    end: Option<i64>,
    start_field: &str,
    end_field: &str,
) {
    if let Some(s) = start {
        query.push_str(&format!(" AND {end_field} > '{}'", s));
    }

    if let Some(e) = end {
        query.push_str(&format!(" AND {start_field} < '{}'", e));
    }
}

/// Appends an `IN (...)` filter to the query if the list is not empty.
pub fn append_filter_list(query: &mut String, field: &str, values: &[String]) {
    if !values.is_empty() {
        let list = values
            .iter()
            .map(|v| format!("'{}'", v))
            .collect::<Vec<_>>()
            .join(", ");
        query.push_str(&format!(" AND {field} IN ({list})"));
    }
}

/// Appends a full set of optional filters (apps, projects, categories, etc.)
/// using the appropriate field names.
pub fn append_all_filters(query: &mut String, filters: SummaryFilters) {
    if let Some(apps) = &filters.apps {
        append_filter_list(query, "apps.name", apps);
    }
    if let Some(projects) = &filters.projects {
        append_filter_list(query, "projects.name", projects);
    }
    if let Some(categories) = &filters.categories {
        append_filter_list(query, "categories.name", categories);
    }
    if let Some(branches) = &filters.branches {
        append_filter_list(query, "branches.name", branches);
    }
    if let Some(entities) = &filters.entities {
        append_filter_list(query, "entities.name", entities);
    }
    if let Some(languages) = &filters.languages {
        append_filter_list(query, "languages.name", languages);
    }
}

/// Appends JOIN clauses for events to resolve all foreign keys.
/// `inner_join` indicates which related table (if any) should be INNER JOINed
/// instead of LEFT JOINed (i.e., the table that supplies the group_key).
pub fn append_standard_joins(query: &mut String, inner_join: Option<&str>) {
    let j = |tbl: &str| {
        if inner_join == Some(tbl) {
            " JOIN "
        } else {
            " LEFT JOIN "
        }
    };
    query.push_str(&format!(
        "{j_apps}apps ON events.app_id = apps.id\
          {j_projects}projects ON events.project_id = projects.id\
          {j_entities}entities ON events.entity_id = entities.id\
          {j_branches}branches ON events.branch_id = branches.id\
          {j_categories}categories ON events.category_id = categories.id\
          {j_languages}languages ON events.language_id = languages.id\
          {j_sources}sources ON events.source_id = sources.id",
        j_apps = j("apps"),
        j_projects = j("projects"),
        j_entities = j("entities"),
        j_branches = j("branches"),
        j_categories = j("categories"),
        j_languages = j("languages"),
        j_sources = j("sources"),
    ));
}

/// Optionally appends a GROUP BY clause.
pub fn append_group_by(query: &mut String, group_by_field: Option<&str>) {
    if let Some(field) = group_by_field {
        query.push_str(&format!(" GROUP BY {}", field));
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
