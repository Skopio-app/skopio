use serde::{Deserialize, Serialize};
use types::Project;

#[derive(Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedProjects {
    pub data: Vec<Project>,
    pub next_cursor: Option<i64>,
    pub total_pages: Option<u32>,
    pub cursors: Vec<Option<i64>>,
}
