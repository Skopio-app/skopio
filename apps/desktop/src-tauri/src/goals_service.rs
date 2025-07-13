use std::{sync::Arc, time::Duration as StdDuration};

use chrono::{Datelike, Duration, TimeZone, Utc};
use common::{
    models::inputs::SummaryQueryInput,
    time::{TimeRange, TimeRangePreset},
};
use db::{
    desktop::goals::{
        delete_goal, fetch_all_goals, insert_goal, modify_goal, Goal, GoalInput, GoalUpdateInput,
        TimeSpan,
    },
    DBContext,
};
use log::{debug, error, info};
use tauri::AppHandle;
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    network::summaries::fetch_total_time,
    notification::{create_notification_window, NotificationPayload},
};

#[derive(Clone)]
pub struct GoalService {
    db: Arc<DBContext>,
    stop_tx: broadcast::Sender<()>,
}

impl GoalService {
    pub fn new(db: Arc<DBContext>) -> Self {
        let (stop_tx, _) = broadcast::channel(1);
        Self { db, stop_tx }
    }

    pub fn start(self: Arc<Self>, app: &AppHandle) -> JoinHandle<()> {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_secs(30));
            let mut stop_rx = self.stop_tx.subscribe();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = self.check_goals(&app_handle).await {
                            error!("Goal check failed: {e:?}");
                        }
                    },
                    _ = stop_rx.recv() => {
                        info!("GoalService stopped");
                        break;
                    }
                }
            }
        })
    }

    pub async fn check_goals(&self, app: &AppHandle) -> anyhow::Result<()> {
        let goals = fetch_all_goals(&self.db).await?;
        for goal in goals {
            debug!("Evaluating goal: {:?}", goal);
            let total_tracked = self.evaluate_goal(&goal).await?;
            debug!(
                "Goal {} | Target: {}s | Tracked: {}s",
                goal.id, goal.target_seconds, total_tracked,
            );

            if total_tracked >= goal.target_seconds {
                debug!("Goal {} met", goal.id);
            } else {
                debug!("Goal {} in progress", goal.id);
                let payload = NotificationPayload {
                    title: "Goal in progress".to_string(),
                    duration_ms: 7000,
                    message: Some(goal.name),
                    sound_file: Some(String::from("goal_success.mp3")),
                };
                let _ = create_notification_window(app, payload)
                    .map_err(|e| format!("Error showing notification: {}", e));
            }
        }

        Ok(())
    }

    async fn evaluate_goal(&self, goal: &Goal) -> anyhow::Result<i64> {
        let range = resolve_time_range(&goal.time_span)
            .ok_or_else(|| anyhow::anyhow!("Invalid time_span: {:?}", goal.time_span))?;

        let query = SummaryQueryInput {
            start: Some(range.start()),
            end: Some(range.end()),
            apps: if goal.use_apps {
                Some(goal.apps.clone())
            } else {
                None
            },
            categories: if goal.use_categories {
                Some(goal.categories.clone())
            } else {
                None
            },
            entities: None,
            branches: None,
            languages: None,
            projects: None,
            include_afk: false,
        };

        let total = fetch_total_time(query).await.map_err(anyhow::Error::msg)?;
        Ok(total)
    }

    pub fn shutdown(&self) {
        let _ = self.stop_tx.send(());
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_goals(db: tauri::State<'_, Arc<DBContext>>) -> Result<Vec<Goal>, String> {
    fetch_all_goals(&db)
        .await
        .map_err(|e| format!("Failed to fetch goals: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn add_goal(
    db: tauri::State<'_, Arc<DBContext>>,
    input: GoalInput,
) -> Result<(), String> {
    insert_goal(&db, input)
        .await
        .map_err(|e| format!("DB insert failed: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_goal(
    db: tauri::State<'_, Arc<DBContext>>,
    goal_id: i64,
    input: GoalUpdateInput,
) -> Result<(), String> {
    modify_goal(&db, goal_id, input)
        .await
        .map_err(|e| format!("Goal DB updated failed: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_goal(db: tauri::State<'_, Arc<DBContext>>, goal_id: i64) -> Result<(), String> {
    delete_goal(&db, goal_id)
        .await
        .map_err(|e| format!("Goal DB delete failed: {}", e))?;

    Ok(())
}

fn resolve_time_range(time_span: &TimeSpan) -> Option<TimeRange> {
    match time_span {
        TimeSpan::Day => Some(TimeRange::from(TimeRangePreset::Today)),
        TimeSpan::Week => Some(TimeRange::from(TimeRangePreset::ThisWeek)),
        TimeSpan::Month => Some(TimeRange::from(TimeRangePreset::ThisMonth)),
        TimeSpan::Year => {
            let now = Utc::now();
            let start = Utc
                .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
                .single()
                .unwrap();
            let end = start + Duration::days(366);
            Some(TimeRange {
                start,
                end,
                bucket: None,
            })
        }
    }
}
