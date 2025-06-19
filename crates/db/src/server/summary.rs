use chrono::NaiveDateTime;
use serde::Serialize;

use crate::DBContext;

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

    pub async fn execute_range_summary(
        &self,
        db: &DBContext,
    ) -> Result<Vec<GroupedTimeSummary>, sqlx::Error> {
        let group_key = match self.group_by.as_deref() {
            Some("app") => "apps.name",
            Some("project") => "projects.name",
            Some("language") => "languages.name",
            Some("branch") => "branches.name",
            Some("activity_type") => "events.activity_type",
            Some("entity") => "entities.name",
            _ => "projects.name",
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

        base_query.push_str(" GROUP BY ");
        base_query.push_str(group_key);

        let mut union_query = base_query;

        if self.include_afk {
            if let Some(start) = self.start {
                union_query.push_str(" UNION ALL SELECT 'AFK' as group_key, SUM(duration) as total_seconds FROM afk_events WHERE afk_start >= '");
                union_query.push_str(&start.to_string());
                union_query.push('\'');

                if let Some(end) = self.end {
                    union_query.push_str(" AND afk_end <= '");
                    union_query.push_str(&end.to_string());
                    union_query.push('\'');
                }
            }
        }

        let records = sqlx::query_as::<_, GroupedTimeSummary>(&union_query)
            .fetch_all(db.pool())
            .await?;

        Ok(records)
    }

    pub async fn execute_total_time(&self, db: &DBContext) -> Result<i64, sqlx::Error> {
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

        Ok(result.unwrap_or(0))
    }
}

impl Default for SummaryQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
