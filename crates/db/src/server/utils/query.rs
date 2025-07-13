use chrono::{DateTime, Local, NaiveDateTime, Utc};
use common::{models::inputs::Group, time::TimeBucket};

use crate::server::utils::summary_filter::SummaryFilters;

/// Appends a date range filter to the query using the specified start and end field names.
/// Dates are formatted as RFC3339 (ISO 8601) to ensure proper string comparison.
pub fn append_date_range(
    query: &mut String,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    start_field: &str,
    end_field: &str,
) {
    if let Some(s) = start {
        query.push_str(&format!(" AND {end_field} > '{}'", s.to_rfc3339()));
    }

    if let Some(e) = end {
        query.push_str(&format!(" AND {start_field} < '{}'", e.to_rfc3339()));
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

/// Appends JOIN clauses for events to resolve all foreign keys
pub fn append_standard_joins(query: &mut String) {
    // Space at the beginning is intentional
    query.push_str(
        " LEFT JOIN apps ON events.app_id = apps.id \
                 LEFT JOIN projects ON events.project_id = projects.id \
                 LEFT JOIN entities ON events.entity_id = entities.id \
                 LEFT JOIN branches ON events.branch_id = branches.id \
                 LEFT JOIN categories ON events.category_id = categories.id \
                 LEFT JOIN languages ON events.language_id = languages.id",
    );
}

/// Optionally appends a GROUP BY clause.
pub fn append_group_by(query: &mut String, group_by_field: Option<&str>) {
    if let Some(field) = group_by_field {
        query.push_str(&format!(" GROUP BY {}", field));
    }
}

pub fn group_key_column(group: Option<Group>) -> &'static str {
    match group {
        Some(Group::App) => "apps.name",
        Some(Group::Project) => "projects.name",
        Some(Group::Language) => "languages.name",
        Some(Group::Branch) => "branches.name",
        Some(Group::Category) => "categories.name",
        Some(Group::Entity) => "entities.name",
        None => "'Total'",
    }
}

/// Formats a SQLite-compatible time bucket expression based on the bucket type.
pub fn get_time_bucket_expr(bucket: Option<TimeBucket>) -> &'static str {
    match bucket {
        Some(TimeBucket::Hour) => "strftime('%Y-%m-%d %H:00:00', events.timestamp)",
        Some(TimeBucket::Day) => "strftime('%Y-%m-%d', events.timestamp)",
        Some(TimeBucket::Week) => "strftime('%Y-W%W', events.timestamp)",
        Some(TimeBucket::Month) => "strftime('%Y-%m', events.timestamp)",
        Some(TimeBucket::Year) => "strftime('%Y', events.timestamp)",
        None => "'Unbucketed'",
    }
}

pub fn convert_utc_bucket_to_local(bucket: &str) -> String {
    let formats = [
        ("%Y-%m-%d %H:%M:%S", "hour"),
        ("%Y-%m-%d", "day"),
        ("%Y-%m", "month"),
        ("%Y", "year"),
    ];

    let trimmed = bucket.trim();

    for (fmt, granularity) in formats.iter() {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            let utc_dt: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
            let local_dt = utc_dt.with_timezone(&Local);
            return match *granularity {
                "hour" => local_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                "day" => local_dt.format("%Y-%m-%d").to_string(),
                "month" => local_dt.format("%Y-%m").to_string(),
                "year" => local_dt.format("%Y").to_string(),
                _ => local_dt.to_rfc3339(),
            };
        }
    }

    bucket.to_string()
}
