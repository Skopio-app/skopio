use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;


#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub root_path: Option<String>,
    pub metadata: Option<String>,
}

impl Project {
    // Insert a new project if it doesn't already exist
    pub async fn insert(pool: &SqlitePool, project: Project) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR IGNORE INTO projects (name, root_path, metadata) VALUES (?, ?, ?)",
            project.name,
            project.root_path,
            project.metadata
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    // Fetch a project by name
    pub async fn get_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as!(
            Project,
            "SELECT id, name, root_path, metadata FROM projects WHERE NAME = ?",
            name
        )
        .fetch_optional(pool)
        .await
    }
}
