use std::{collections::HashMap, time::Instant};

use chrono::NaiveDateTime;
use common::time::TimeBucket;
use log::info;
use serde::Serialize;

use crate::{
    models::{BucketTimeSummary, RawBucketRow},
    DBContext,
};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupedTimeSummary {
    pub group_key: String,
    pub total_seconds: i64,
}

pub struct SummaryQueryBuilder {
    start: Option<NaiveDateTime>,
    end: Option<NaiveDateTime>,
    app_names: Option<Vec<String>>,
    project_names: Option<Vec<String>>,
    activity_types: Option<Vec<String>>,
    entity_names: Option<Vec<String>>,
    branch_names: Option<Vec<String>>,
    language_names: Option<Vec<String>>,
    group_by: Option<String>,
    include_afk: bool,
    time_bucket: Option<TimeBucket>,
}

impl SummaryQueryBuilder {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            app_names: None,
            project_names: None,
            activity_types: None,
            entity_names: None,
            branch_names: None,
            language_names: None,
            group_by: None,
            include_afk: false,
            time_bucket: None,
        }
    }

    pub fn start(mut self, start: NaiveDateTime) -> Self {
        self.start = Some(start);
        self
    }

    pub fn end(mut self, end: NaiveDateTime) -> Self {
        self.end = Some(end);
        self
    }

    pub fn app_names(mut self, apps: Vec<String>) -> Self {
        self.app_names = Some(apps);
        self
    }

    pub fn project_names(mut self, projects: Vec<String>) -> Self {
        self.project_names = Some(projects);
        self
    }

    pub fn activity_types(mut self, types: Vec<String>) -> Self {
        self.activity_types = Some(types);
        self
    }

    pub fn entity_names(mut self, entities: Vec<String>) -> Self {
        self.entity_names = Some(entities);
        self
    }

    pub fn branch_names(mut self, branches: Vec<String>) -> Self {
        self.branch_names = Some(branches);
        self
    }

    pub fn language_names(mut self, langs: Vec<String>) -> Self {
        self.language_names = Some(langs);
        self
    }

    pub fn group_by(mut self, field: &str) -> Self {
        self.group_by = Some(field.to_string());
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
        group_key_field: &str,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.group_by(group_key_field)
            .execute_range_summary(db)
            .await
    }

    pub async fn execute_apps_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "app").await
    }

    pub async fn execute_projects_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "project").await
    }

    pub async fn execute_branches_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "branch").await
    }

    pub async fn execute_entities_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "entity").await
    }

    pub async fn execute_languages_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "language").await
    }

    pub async fn execute_activity_type_summary(
        self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        self.execute_grouped_summary_by(db, "activity_type").await
    }

    pub async fn execute_range_summary(
        &self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        let start_time = Instant::now();
        let group_key = match self.group_by.as_deref() {
            Some("app") => "apps.name",
            Some("project") => "projects.name",
            Some("language") => "languages.name",
            Some("branch") => "branches.name",
            Some("activity_type") => "events.activity_type",
            Some("entity") => "entities.name",
            _ => "'Total'",
        };

        let mut base_query = format!(
            "SELECT {group_key} as group_key, SUM(duration) as total_seconds FROM events \
            LEFT JOIN apps ON events.app_id = apps.id \
            LEFT JOIN projects ON events.project_id = projects.id \
            LEFT JOIN entities ON events.entity_id = entities.id \
            LEFT JOIN branches ON events.branch_id = branches.id \
            LEFT JOIN languages ON events.language_id = languages.id WHERE 1=1",
            group_key = group_key
        );

        if let Some(start) = self.start {
            base_query.push_str(" AND events.timestamp >= '");
            base_query.push_str(&start.to_string());
            base_query.push('\'');
        }

        if let Some(end) = self.end {
            base_query.push_str(" AND events.end_timestamp <= '");
            base_query.push_str(&end.to_string());
            base_query.push('\'');
        }

        if let Some(apps) = &self.app_names {
            let app_list = apps
                .iter()
                .map(|a| format!("'{}'", a))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND apps.name IN ({})", app_list));
        }

        if let Some(projects) = &self.project_names {
            let proj_list = projects
                .iter()
                .map(|p| format!("'{}'", p))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND projects.name IN ({})", proj_list));
        }

        if let Some(activity_types) = &self.activity_types {
            let list = activity_types
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND events.activity_type IN ({})", list));
        }

        if let Some(entities) = &self.entity_names {
            let list = entities
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND entities.name IN ({})", list));
        }

        if let Some(branches) = &self.branch_names {
            let list = branches
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND branches.name IN ({})", list));
        }

        if let Some(langs) = &self.language_names {
            let list = langs
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND languages.name IN ({})", list));
        }

        if self.group_by.is_some() {
            base_query.push_str(" GROUP BY ");
            base_query.push_str(group_key);
        }

        let mut final_query = base_query.clone();

        if self.include_afk {
            let mut afk_query = String::from("SELECT 'AFK' as group_key, SUM(duration) as total_seconds FROM afk_events WHERE 1=1");

            if let Some(start) = self.start {
                afk_query.push_str(" AND afk_start >= '");
                afk_query.push_str(&start.to_string());
                afk_query.push('\'');
            }

            if let Some(end) = self.end {
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
        let mut query = String::from(
            "SELECT SUM(duration) as total_seconds FROM events \
            LEFT JOIN apps ON events.app_id = apps.id \
            LEFT JOIN projects ON events.project_id = projects.id \
            LEFT JOIN entities ON events.entity_id = entities.id \
            LEFT JOIN branches ON events.branch_id = branches.id \
            LEFT JOIN languages ON events.language_id = languages.id WHERE 1=1",
        );

        if let Some(start) = self.start {
            query.push_str(" AND events.timestamp >= '");
            query.push_str(&start.to_string());
            query.push('\'');
        }

        if let Some(end) = self.end {
            query.push_str(" AND events.end_timestamp <= '");
            query.push_str(&end.to_string());
            query.push('\'');
        }

        if let Some(apps) = &self.app_names {
            let app_list = apps
                .iter()
                .map(|a| format!("'{}'", a))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND apps.name IN ({})", app_list));
        }

        if let Some(projects) = &self.project_names {
            let proj_list = projects
                .iter()
                .map(|p| format!("'{}'", p))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND projects.name IN ({})", proj_list));
        }

        if let Some(activity_types) = &self.activity_types {
            let list = activity_types
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND events.activity_type IN ({})", list));
        }

        if let Some(entities) = &self.entity_names {
            let list = entities
                .iter()
                .map(|e| format!("'{}'", e))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND entities.name IN ({})", list));
        }

        if let Some(branches) = &self.branch_names {
            let list = branches
                .iter()
                .map(|e| format!("'{}'", e))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND branches.name IN ({})", list));
        }

        if let Some(langs) = &self.language_names {
            let list = langs
                .iter()
                .map(|l| format!("'{}'", l))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND languages.name IN ({})", list));
        }

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
        let group_key = match self.group_by.as_deref() {
            Some("app") => "apps.name",
            Some("project") => "projects.name",
            Some("language") => "languages.name",
            Some("branch") => "branches.name",
            Some("activity_type") => "events.activity_type",
            Some("entity") => "entities.name",
            _ => "'Total'",
        };

        let time_bucket_expr = match self.time_bucket {
            Some(TimeBucket::Hour) => "strftime('%Y-%m-%d %H:00:00', events.timestamp)",
            Some(TimeBucket::Day) => "strftime('%Y-%m-%d', events.timestamp)",
            Some(TimeBucket::Week) => "strftime('%Y-W%W', events.timestamp)",
            Some(TimeBucket::Month) => "strftime('%Y-%m', events.timestamp)",
            _ => "'Unbucketed'",
        };

        let mut base_query = format!(
            "SELECT {time_bucket_expr} AS bucket, {group_key} AS group_key, SUM(duration) as total_seconds
            FROM events
            LEFT JOIN apps ON events.app_id = apps.id \
            LEFT JOIN projects ON events.project_id = projects.id \
            LEFT JOIN entities ON events.entity_id = entities.id \
            LEFT JOIN branches ON events.branch_id = branches.id \
            LEFT JOIN languages ON events.language_id = languages.id WHERE 1=1",
            group_key = group_key,
            time_bucket_expr = time_bucket_expr
        );

        if let Some(start) = self.start {
            base_query.push_str(" AND events.timestamp >= '");
            base_query.push_str(&start.to_string());
            base_query.push('\'');
        }

        if let Some(end) = self.end {
            base_query.push_str(" AND events.end_timestamp <= '");
            base_query.push_str(&end.to_string());
            base_query.push('\'');
        }

        if let Some(apps) = &self.app_names {
            let list = apps
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND apps.name IN ({})", list));
        }

        if let Some(projects) = &self.project_names {
            let list = projects
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND projects.name IN ({})", list));
        }

        if let Some(activity_types) = &self.activity_types {
            let list = activity_types
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND events.activity_type IN ({})", list));
        }

        if let Some(entities) = &self.entity_names {
            let list = entities
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND entities.name IN ({})", list));
        }

        if let Some(branches) = &self.branch_names {
            let list = branches
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND branches.name IN ({})", list));
        }

        if let Some(langs) = &self.language_names {
            let list = langs
                .iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ");
            base_query.push_str(&format!(" AND languages.name IN ({})", list));
        }

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
                bucket,
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
