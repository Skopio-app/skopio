use crate::DBContext;
use types::Project;

#[derive(Debug, sqlx::FromRow)]
pub struct ServerProject {
    pub id: Option<i64>,
    pub name: String,
    pub root_path: Option<String>,
}

impl From<ServerProject> for Project {
    fn from(value: ServerProject) -> Self {
        Self {
            id: value.id,
            name: value.name,
            root_path: value.root_path,
        }
    }
}

impl ServerProject {
    pub async fn find_or_insert(
        db_context: &DBContext,
        name: &str,
        root_path: &str,
    ) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM projects WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        if let Some(row) = record {
            return row.id.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let result = sqlx::query!(
            "INSERT INTO projects (name, root_path) VALUES (?, ?) RETURNING id",
            name,
            root_path,
        )
        .fetch_one(db_context.pool())
        .await?;

        Ok(result.id)
    }

    /// Fetches a project by name
    pub async fn find_by_name(
        db_context: &DBContext,
        name: &str,
    ) -> Result<Option<Project>, sqlx::Error> {
        let result = sqlx::query_as!(
            ServerProject,
            "SELECT id, name, root_path FROM projects WHERE name = ?",
            name
        )
        .fetch_optional(db_context.pool())
        .await?;

        Ok(result.map(Into::into))
    }

    // pub async fn delete(self, db_context: &DBContext) -> Result<(), sqlx::Error> {
    //     if let Some(id) = self.id {
    //         sqlx::query!("DELETE FROM projects WHERE id = ?", id)
    //             .execute(db_context.pool())
    //             .await?;
    //     }

    //     Ok(())
    // }

    pub async fn fetch_paginated(
        db_context: &DBContext,
        after_id: Option<i64>,
        limit: u32,
    ) -> Result<(Vec<Project>, u32), sqlx::Error> {
        let limit = limit.min(100) as i64;

        let total_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM projects")
            .fetch_one(db_context.pool())
            .await?;

        let rows: Vec<ServerProject> = if let Some(cursor) = after_id {
            sqlx::query_as!(
                ServerProject,
                "
                SELECT id, name, root_path
                FROM projects
                WHERE id > ?
                ORDER BY id
                LIMIT ?
                ",
                cursor,
                limit
            )
            .fetch_all(db_context.pool())
            .await?
        } else {
            sqlx::query_as!(
                ServerProject,
                "
                SELECT id, name, root_path
                FROM projects
                ORDER BY id
                LIMIT ?
                ",
                limit
            )
            .fetch_all(db_context.pool())
            .await?
        };

        let total_pages = ((total_count + limit - 1) / limit) as u32;
        let projects = rows.into_iter().map(Into::into).collect();

        Ok((projects, total_pages))
    }

    pub async fn get_all_cursors(
        db_context: &DBContext,
        limit: u32,
    ) -> Result<Vec<Option<i64>>, sqlx::Error> {
        let limit = limit.min(100);
        let ids = sqlx::query_scalar!("SELECT id FROM projects ORDER BY id")
            .fetch_all(db_context.pool())
            .await?;

        let cursors: Vec<Option<i64>> = ids
            .chunks(limit as usize)
            .map(|chunk| chunk.first().copied())
            .collect();

        Ok(cursors)
    }
}
