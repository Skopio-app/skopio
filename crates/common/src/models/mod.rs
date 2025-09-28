use serde::{Deserialize, Serialize};

pub mod inputs;
pub mod outputs;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InsightBucket {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Group {
    App,
    Project,
    Language,
    Branch,
    Category,
    Entity,
    Source,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InsightType {
    ActiveYears,
    TopN,
    MostActiveDay,
    AggregatedAverage,
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: uuid::Uuid,
    pub name: String,
    pub root_path: Option<String>,
    pub last_updated: Option<i64>,
}
