use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub root_path: Option<String>,
}
