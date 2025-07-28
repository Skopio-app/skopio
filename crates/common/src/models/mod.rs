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
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum InsightType {
    ActiveYears,
    TopN {
        group_by: Group,
        limit: usize,
    },
    MostActiveDay,
    AggregatedAverage {
        bucket: InsightBucket,
        group_by: Option<Group>,
    },
}
