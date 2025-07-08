use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DBContext;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TimeSpan {
    Day,
    Week,
    Month,
    Year,
}

impl FromStr for TimeSpan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" => Ok(TimeSpan::Day),
            "week" => Ok(TimeSpan::Week),
            "month" => Ok(TimeSpan::Month),
            "year" => Ok(TimeSpan::Year),
            _ => Err(format!("Invalid time span: {}", s)),
        }
    }
}

impl ToString for TimeSpan {
    fn to_string(&self) -> String {
        match self {
            TimeSpan::Day => "day",
            TimeSpan::Week => "week",
            TimeSpan::Month => "month",
            TimeSpan::Year => "year",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: i64,
    pub name: String,
    pub target_seconds: i64,
    pub time_span: TimeSpan,
    pub use_apps: bool,
    pub use_categories: bool,
    pub ignore_no_activity_days: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub apps: Vec<String>,
    pub categories: Vec<String>,
    pub excluded_days: Vec<String>,
}

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_seconds: i64,
    pub time_span: TimeSpan,
    pub use_apps: bool,
    pub use_categories: bool,
    pub ignore_no_activity_days: bool,
    pub apps: Vec<String>,
    pub categories: Vec<String>,
    pub excluded_days: Vec<String>,
}

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GoalUpdateInput {
    #[specta(optional)]
    pub name: Option<String>,
    #[specta(optional)]
    pub target_seconds: Option<i64>,
    #[specta(optional)]
    pub time_span: Option<TimeSpan>,
    #[specta(optional)]
    pub use_apps: Option<bool>,
    #[specta(optional)]
    pub use_categories: Option<bool>,
    #[specta(optional)]
    pub ignore_no_activity_days: Option<bool>,
    #[specta(optional)]
    pub apps: Option<Vec<String>>,
    #[specta(optional)]
    pub categories: Option<Vec<String>>,
    #[specta(optional)]
    pub excluded_days: Option<Vec<String>>,
}

pub async fn fetch_all_goals(db: &DBContext) -> Result<Vec<Goal>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
          g.id,
          g.name,
          g.target_seconds,
          g.time_span,
          g.use_apps,
          g.use_categories,
          g.ignore_no_activity_days,
          g.created_at,
          g.updated_at,
          ga.app,
          gc.category,
          gd.day
        FROM goals g
        LEFT JOIN goal_apps ga ON g.id = ga.goal_id
        LEFT JOIN goal_categories gc ON g.id = gc.goal_id
        LEFT JOIN goal_excluded_days gd ON g.id = gd.goal_id
        ORDER BY g.id
        "#
    )
    .fetch_all(db.pool())
    .await?;

    let mut grouped: HashMap<i64, Goal> = HashMap::new();

    for row in rows {
        let entry = grouped.entry(row.id).or_insert_with(|| Goal {
            id: row.id,
            name: row.name,
            target_seconds: row.target_seconds,
            time_span: TimeSpan::from_str(&row.time_span).unwrap(),
            use_apps: row.use_apps,
            use_categories: row.use_categories,
            ignore_no_activity_days: row.ignore_no_activity_days,
            created_at: row.created_at.parse::<DateTime<Utc>>().unwrap_or_default(),
            updated_at: row.updated_at.parse::<DateTime<Utc>>().unwrap_or_default(),
            apps: vec![],
            categories: vec![],
            excluded_days: vec![],
        });

        if let Some(app) = row.app {
            if !entry.apps.contains(&app) {
                entry.apps.push(app);
            }
        }

        if let Some(cat) = row.category {
            if !entry.categories.contains(&cat) {
                entry.categories.push(cat);
            }
        }

        if let Some(day) = row.day {
            if !entry.excluded_days.contains(&day) {
                entry.excluded_days.push(day);
            }
        }
    }

    Ok(grouped.into_values().collect())
}

pub async fn insert_goal(db: &DBContext, input: GoalInput) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let time_span = input.time_span.to_string();

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO goals (created_at, updated_at, name, target_seconds, time_span, use_apps, use_categories, ignore_no_activity_days, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        input.created_at,
        input.updated_at,
        input.name,
        input.target_seconds,
        time_span,
        input.use_apps,
        input.use_categories,
        input.ignore_no_activity_days,
        now,
        now
    )
    .fetch_one(db.pool())
    .await?;

    for app in input.apps {
        sqlx::query!(
            "INSERT INTO goal_apps (goal_id, app) VALUES (?, ?)",
            id,
            app
        )
        .execute(db.pool())
        .await?;
    }

    for cat in input.categories {
        sqlx::query!(
            "INSERT INTO goal_categories (goal_id, category) VALUES (?, ?)",
            id,
            cat
        )
        .execute(db.pool())
        .await?;
    }

    for day in input.excluded_days {
        sqlx::query!(
            "INSERT INTO goal_excluded_days (goal_id, day) VALUES (?, ?)",
            id,
            day
        )
        .execute(db.pool())
        .await?;
    }

    Ok(())
}

pub async fn modify_goal(
    db: &DBContext,
    goal_id: i64,
    update: GoalUpdateInput,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    let mut tx = db.pool().begin().await?;

    sqlx::query("UPDATE goals set updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&goal_id)
        .execute(&mut *tx)
        .await?;

    if let Some(name) = update.name {
        sqlx::query("UPDATE goals set name = ? WHERE id = ?")
            .bind(name)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(seconds) = update.target_seconds {
        sqlx::query("UPDATE goals set target_seconds = ? WHERE id = ?")
            .bind(seconds)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(span) = update.time_span {
        sqlx::query("UPDATE goals set time_span = ? WHERE id = ?")
            .bind(span.to_string())
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(val) = update.use_apps {
        sqlx::query("UPDATE goals set use_apps = ? WHERE id = ?")
            .bind(val)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(val) = update.use_categories {
        sqlx::query("UPDATE goals set use_categories = ? WHERE id = ?")
            .bind(val)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(val) = update.ignore_no_activity_days {
        sqlx::query("UPDATE goals set ignore_no_activity_days = ? WHERE id = ?")
            .bind(val)
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(apps) = update.apps {
        sqlx::query("DELETE FROM goal_apps WHERE goal_id = ?")
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;
        for app in apps {
            sqlx::query("INSERT INTO goal_apps (goal_id, app) VALUES (?, ?)")
                .bind(goal_id)
                .bind(app)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(cats) = update.categories {
        sqlx::query("DELETE FROM goal_categories WHERE goal_id = ?")
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;

        for cat in cats {
            sqlx::query("INSERT INTO goal_categories (goal_id, category) VALUES (?, ?)")
                .bind(goal_id)
                .bind(cat)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(days) = update.excluded_days {
        sqlx::query("DELETE FROM goal_excluded_days WHERE goal_id = ?")
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;

        for day in days {
            sqlx::query("INSERT INTO goal_excluded_days (goal_id, day) VALUES (?, ?)")
                .bind(goal_id)
                .bind(day)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
