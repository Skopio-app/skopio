use common::models::outputs::FullEvent;
use db::server::afk_events::AFKEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AFKEventOutput {
    pub id: i64,
    pub afk_start: String,
    pub afk_end: Option<String>,
    pub duration: Option<i64>,
}

impl From<AFKEvent> for AFKEventOutput {
    fn from(value: AFKEvent) -> Self {
        AFKEventOutput {
            id: value.id.unwrap_or_default(),
            afk_start: value.afk_start.to_rfc3339(),
            afk_end: value.afk_end.map(|c| c.to_rfc3339()),
            duration: value.duration,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventOutput {
    pub id: i64,
    pub timestamp: String,
    pub end_timestamp: Option<String>,
    pub duration: Option<i64>,
    pub activity_type: String,
    pub app: Option<String>,
    pub entity: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
}

impl From<FullEvent> for EventOutput {
    fn from(value: FullEvent) -> Self {
        EventOutput {
            id: value.id,
            timestamp: value.timestamp.to_rfc3339(),
            end_timestamp: value.end_timestamp.map(|c| c.to_rfc3339()),
            duration: value.duration,
            activity_type: value.category,
            app: value.app,
            entity: value.entity,
            project: value.project,
            branch: value.branch,
            language: value.language,
        }
    }
}
