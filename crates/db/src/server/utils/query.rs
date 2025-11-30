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
    fn push_overlap_duration(&mut self, range_start_field: &str, range_end_field: &str);
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

    /// Appends SQL logic to calculate the overlapping duration between events and a time range.
    ///
    /// This function generates a CASE statement that computes how much of an event's duration
    /// falls within a specified time range. It handles four scenarios where events may
    /// partially or fully overlap with the range boundaries.
    ///
    /// # Overview
    ///
    /// When tracking time-based events across specific time ranges (hours, days, weeks),
    /// events may:
    /// - Start and end within the range (full overlap)
    /// - Start before the range and end within it (partial overlap at start)
    /// - Start within the range and end after it (partial overlap at end)
    /// - Start before and end after the range (span the entire range)
    ///
    /// This function ensures only the overlapping portion is counted, providing accurate
    /// time tracking across boundaries.
    fn push_overlap_duration(&mut self, range_start_field: &str, range_end_field: &str) {
        self.push(
            "CASE \
        WHEN events.timestamp >= ",
        )
        .push(range_start_field)
        .push(" AND events.end_timestamp <= ")
        .push(range_end_field)
        .push(
            " THEN events.duration \
        WHEN events.timestamp < ",
        )
        .push(range_start_field)
        .push(" AND events.end_timestamp <= ")
        .push(range_end_field)
        .push(" THEN events.end_timestamp - ")
        .push(range_start_field)
        .push(
            " \
        WHEN events.timestamp >= ",
        )
        .push(range_start_field)
        .push(" AND events.end_timestamp > ")
        .push(range_end_field)
        .push(" THEN ")
        .push(range_end_field)
        .push(
            " - events.timestamp \
        WHEN events.timestamp < ",
        )
        .push(range_start_field)
        .push(" AND events.end_timestamp > ")
        .push(range_end_field)
        .push(" THEN ")
        .push(range_end_field)
        .push(" - ")
        .push(range_start_field)
        .push(
            " \
        ELSE 0 \
        END",
        );
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

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::server::utils::summary_filter::SummaryFilters;
    use common::{models::Group, time::TimeBucket};
    use sqlx::{Execute, QueryBuilder, Sqlite, SqlitePool};
    use uuid::Uuid;

    /// Helper to create a test database with events table
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Minimal schema for testing
        sqlx::query(
            "CREATE TABLE events (
                    id BLOB PRIMARY KEY,
                    timestamp INTEGER NOT NULL,
                    end_timestamp INTEGER NOT NULL,
                    duration INTEGER NOT NULL
                )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Helper to insert a test event
    async fn insert_event(
        pool: &SqlitePool,
        timestamp: i64,
        end_timestamp: i64,
        duration: i64,
    ) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO events (id, timestamp, end_timestamp, duration)
                  VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(timestamp)
        .bind(end_timestamp)
        .bind(duration)
        .execute(pool)
        .await
        .unwrap();

        id
    }

    /// Helper to build and execute overlap query
    async fn calculate_overlap(pool: &SqlitePool, range_start: i64, range_end: i64) -> i64 {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT COALESCE(SUM(");
        qb.push_overlap_duration(&range_start.to_string(), &range_end.to_string());
        qb.push("), 0) FROM events WHERE end_timestamp > ")
            .push_bind(range_start)
            .push(" AND timestamp < ")
            .push_bind(range_end);

        let result: i64 = qb.build_query_scalar().fetch_one(pool).await.unwrap();

        result
    }

    #[sqlx::test]
    async fn test_event_fully_within_range() {
        let pool = setup_test_db().await;

        // Event: 1000-2000 (duration: 1000)
        // Range: 500-2500
        // Expected: Full duration (1000)
        insert_event(&pool, 1000, 2000, 1000).await;

        let overlap = calculate_overlap(&pool, 500, 2500).await;
        assert_eq!(
            overlap, 1000,
            "Event fully within range should count full duration"
        );
    }

    #[sqlx::test]
    async fn test_event_starts_before_range() {
        let pool = setup_test_db().await;

        // Event: 500-1500 (duration: 1000)
        // Range: 1000-2000
        // Expected: 1500 - 1000 = 500 (only portion after range start)
        insert_event(&pool, 500, 1500, 1000).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(
            overlap, 500,
            "Event starting before range should count from range start"
        );
    }

    #[sqlx::test]
    async fn test_event_ends_after_range() {
        let pool = setup_test_db().await;

        // Event: 1500-2500 (duration: 1000)
        // Range: 1000-2000
        // Expected: 2000 - 1500 = 500 (only portion before range end)
        insert_event(&pool, 1500, 2500, 1000).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(
            overlap, 500,
            "Event ending after range should count until range end"
        );
    }

    #[sqlx::test]
    async fn test_event_spans_entire_range() {
        let pool = setup_test_db().await;

        // Event: 500-2500 (duration: 2000)
        // Range: 1000-2000
        // Expected: 2000 - 1000 = 1000 (entire range duration)
        insert_event(&pool, 500, 2500, 2000).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(
            overlap, 1000,
            "Event spanning entire range should count range duration"
        );
    }

    #[sqlx::test]
    async fn test_event_completely_before_range() {
        let pool = setup_test_db().await;

        // Event: 100-500 (duration: 400)
        // Range: 1000-2000
        // Expected: 0 (no overlap)
        insert_event(&pool, 100, 500, 400).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 0, "Event before range should not be counted");
    }

    #[sqlx::test]
    async fn test_event_completely_after_range() {
        let pool = setup_test_db().await;

        // Event: 2500-3000 (duration: 500)
        // Range: 1000-2000
        // Expected: 0 (no overlap)
        insert_event(&pool, 2500, 3000, 500).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 0, "Event after range should not be counted");
    }

    #[sqlx::test]
    async fn test_multiple_events_various_overlaps() {
        let pool = setup_test_db().await;

        // Range: 1000-2000

        // Event 1: Fully within (1200-1800, duration: 600)
        insert_event(&pool, 1200, 1800, 600).await;

        // Event 2: Starts before (800-1500, duration: 700)
        // Expected overlap: 1500 - 1000 = 500
        insert_event(&pool, 800, 1500, 700).await;

        // Event 3: Ends after (1700-2300, duration: 600)
        // Expected overlap: 2000 - 1700 = 300
        insert_event(&pool, 1700, 2300, 600).await;

        // Event 4: Spans entire range (500-2500, duration: 2000)
        insert_event(&pool, 500, 2500, 2000).await;

        // Total expected = 600 + 500 + 300 + 1000 = 2400
        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 2400, "Multiple events should sum correctly");
    }

    #[sqlx::test]
    async fn test_event_at_exact_boundaries() {
        let pool = setup_test_db().await;

        // Event exactly at range start
        insert_event(&pool, 1000, 1500, 500).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 500, "Event at range start should be included");
    }

    #[sqlx::test]
    async fn test_event_ending_at_exact_boundary() {
        let pool = setup_test_db().await;

        // Event ending exactly at range end
        insert_event(&pool, 1500, 2000, 500).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(
            overlap, 500,
            "Event ending at range end should be fully included"
        );
    }

    #[sqlx::test]
    async fn test_zero_duration_event() {
        let pool = setup_test_db().await;

        // Event with zero duration (point in time)
        insert_event(&pool, 1500, 1500, 0).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 0, "Zero duration event should contribute 0");
    }

    #[sqlx::test]
    async fn test_single_second_event() {
        let pool = setup_test_db().await;

        // Event: 1500-1501 (duration: 1)
        insert_event(&pool, 1500, 1501, 1).await;

        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 1, "Single second event should count as 1");
    }

    #[sqlx::test]
    async fn test_empty_database() {
        let pool = setup_test_db().await;

        // No events inserted
        let overlap = calculate_overlap(&pool, 1000, 2000).await;
        assert_eq!(overlap, 0, "Empty database should return 0");
    }

    #[sqlx::test]
    async fn test_negative_duration_handling() {
        let pool = setup_test_db().await;

        // Malformed event: end before start
        insert_event(&pool, 2000, 1000, -1000).await;

        let overlap = calculate_overlap(&pool, 1000, 3000).await;

        // The WHERE clause (end_timestamp > range_start AND timestamp < range_end)
        // will exclude this event, so overlap should be 0
        assert_eq!(
            overlap, 0,
            "Malformed event should not contribute to overlap"
        );
    }

    #[sqlx::test]
    async fn test_very_large_duration() {
        let pool = setup_test_db().await;

        // Event spanning a week (604800 seconds)
        let week_start = 1764450000;
        let week_end = 1765054800;

        insert_event(&pool, week_start, week_end, 604800).await;

        // Query for one day within that week
        let day_start = 1764450000;
        let day_end = 1764536400; // +86400 seconds (1 day)

        let overlap = calculate_overlap(&pool, day_start, day_end).await;
        assert_eq!(
            overlap, 86400,
            "Should count exactly one day from week-long event"
        );
    }

    #[test]
    fn test_append_date_range_both_bounds() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        qb.append_date_range(
            Some(1000),
            Some(2000),
            "events.timestamp",
            "events.end_timestamp",
        );

        let sql = qb.build().sql();
        assert!(sql.contains("AND (1=1"));
        assert!(sql.contains("events.end_timestamp >"));
        assert!(sql.contains("events.timestamp <"));
    }

    #[test]
    fn test_append_date_range_start_only() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        qb.append_date_range(Some(1000), None, "events.timestamp", "events.end_timestamp");

        let sql = qb.build().sql();
        assert!(sql.contains("events.end_timestamp >"));
        assert!(!sql.contains("events.timestamp <"));
    }

    #[test]
    fn test_append_date_range_end_only() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        qb.append_date_range(None, Some(2000), "events.timestamp", "events.end_timestamp");

        let sql = qb.build().sql();
        assert!(!sql.contains("events.end_timestamp >"));
        assert!(sql.contains("events.timestamp <"));
    }

    #[test]
    fn test_append_date_range_none() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        let original_sql = qb.build().sql().to_string();

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        qb.append_date_range(None, None, "events.timestamp", "events.end_timestamp");

        assert_eq!(original_sql, qb.build().sql());
    }

    #[test]
    fn test_append_filter_list_with_values() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        let apps = vec!["VSCode".to_string(), "IntelliJ".to_string()];
        qb.append_filter_list("apps.name", &apps);

        let sql = qb.build().sql();
        assert!(sql.contains("AND apps.name IN ("));
    }

    #[test]
    fn test_append_filter_list_empty() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        let original_sql = qb.build().sql().to_string();

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        let apps: Vec<String> = vec![];
        qb.append_filter_list("apps.name", &apps);

        assert_eq!(original_sql, qb.build().sql());
    }

    #[test]
    fn test_append_all_filters() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events WHERE 1=1");
        let filters = SummaryFilters {
            apps: Some(vec!["VSCode".to_string()]),
            projects: Some(vec!["MyProject".to_string()]),
            categories: None,
            entities: None,
            branches: None,
            languages: Some(vec!["Rust".to_string()]),
            ..Default::default()
        };

        qb.append_all_filters(&filters);
        let sql = qb.build().sql();

        assert!(sql.contains("apps.name IN ("));
        assert!(sql.contains("projects.name IN ("));
        assert!(sql.contains("languages.name IN ("));
        assert!(!sql.contains("categories.name"));
    }

    #[test]
    fn test_append_standard_joins_no_inner() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events");
        qb.append_standard_joins(None);

        let sql = qb.build().sql();
        assert!(sql.contains("LEFT JOIN apps"));
        assert!(sql.contains("LEFT JOIN projects"));
        assert!(sql.contains("LEFT JOIN entities"));
        assert!(sql.contains("LEFT JOIN branches"));
        assert!(sql.contains("LEFT JOIN categories"));
        assert!(sql.contains("LEFT JOIN languages"));
        assert!(sql.contains("LEFT JOIN sources"));
    }

    #[test]
    fn test_append_standard_joins_with_inner() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM events");
        qb.append_standard_joins(Some("apps"));

        let sql = qb.build().sql();
        assert!(sql.contains(" JOIN apps ON"));
        assert!(!sql.contains("LEFT JOIN apps"));
        assert!(sql.contains("LEFT JOIN projects"));
    }

    #[test]
    fn test_group_key_info_all_variants() {
        assert_eq!(
            group_key_info(Some(Group::App)),
            ("apps.name", Some("apps"))
        );
        assert_eq!(
            group_key_info(Some(Group::Project)),
            ("projects.name", Some("projects"))
        );
        assert_eq!(
            group_key_info(Some(Group::Entity)),
            ("entities.name", Some("entities"))
        );
        assert_eq!(
            group_key_info(Some(Group::Branch)),
            ("branches.name", Some("branches"))
        );
        assert_eq!(
            group_key_info(Some(Group::Category)),
            ("categories.name", Some("categories"))
        );
        assert_eq!(
            group_key_info(Some(Group::Language)),
            ("languages.name", Some("languages"))
        );
        assert_eq!(
            group_key_info(Some(Group::Source)),
            ("sources.name", Some("sources"))
        );
        assert_eq!(group_key_info(None), ("'Total'", None));
    }

    #[test]
    fn test_bucket_step_variants() {
        match bucket_step(Some(TimeBucket::Hour)) {
            BucketStep::Seconds(3600) => {}
            _ => panic!("Expected Hour to be 3600 seconds"),
        }

        match bucket_step(Some(TimeBucket::Day)) {
            BucketStep::Seconds(86400) => {}
            _ => panic!("Expected Day to be 86400 seconds"),
        }

        match bucket_step(Some(TimeBucket::Week)) {
            BucketStep::Calendar("+7 days") => {}
            _ => panic!("Expected Week to be calendar-based"),
        }

        match bucket_step(Some(TimeBucket::Month)) {
            BucketStep::Calendar("+1 month") => {}
            _ => panic!("Expected Month to be calendar-based"),
        }

        match bucket_step(Some(TimeBucket::Year)) {
            BucketStep::Calendar("+1 year") => {}
            _ => panic!("Expected Year to be calendar-based"),
        }

        match bucket_step(None) {
            BucketStep::Seconds(86400) => {}
            _ => panic!("Expected None to default to Day"),
        }
    }

    #[test]
    fn test_push_bucket_label_expr() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(Some(TimeBucket::Hour));
        assert!(qb.build().sql().contains("%Y-%m-%d %H:00:00"));

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(Some(TimeBucket::Day));
        assert!(qb.build().sql().contains("%Y-%m-%d"));

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(Some(TimeBucket::Week));
        assert!(qb.build().sql().contains("%Y-W%W"));

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(Some(TimeBucket::Month));
        assert!(qb.build().sql().contains("%Y-%m"));

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(Some(TimeBucket::Year));
        assert!(qb.build().sql().contains("%Y"));

        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        qb.push_bucket_label_expr(None);
        assert!(qb.build().sql().contains("'Unbucketed'"));
    }

    #[test]
    fn test_push_next_end_with_seconds() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        let step = BucketStep::Seconds(3600);
        push_next_end_with(
            &mut qb,
            |q| {
                q.push("start_ts");
            },
            &step,
        );

        let sql = qb.build().sql();
        assert!(sql.contains("start_ts + 3600"));
    }

    #[test]
    fn test_push_next_end_with_calendar() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
        let step = BucketStep::Calendar("+1 month");
        push_next_end_with(
            &mut qb,
            |q| {
                q.push("start_ts");
            },
            &step,
        );

        let sql = qb.build().sql();
        assert!(sql.contains("strftime('%s', datetime("));
        assert!(sql.contains("+1 month"));
    }
}
