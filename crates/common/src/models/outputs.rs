use serde::{Deserialize, Serialize};
use types::Project;

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedProjects {
    pub data: Vec<Project>,
    pub total_pages: Option<u32>,
    pub cursors: Vec<Option<i64>>,
}
