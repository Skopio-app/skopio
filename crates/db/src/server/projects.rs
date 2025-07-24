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

    /// Fetches a project by id
    pub async fn find_by_id(
        db_context: &DBContext,
        id: i64,
    ) -> Result<Option<Project>, sqlx::Error> {
        let result = sqlx::query_as!(
            ServerProject,
            "SELECT id, name, root_path FROM projects WHERE id = ?",
            id
        )
        .fetch_optional(db_context.pool())
        .await?;

        Ok(result.map(Into::into))
    }

    pub async fn fetch_paginated(
        db_context: &DBContext,
        after_id: Option<i64>,
        limit: u32,
    ) -> Result<Vec<Project>, sqlx::Error> {
        let limit = limit.min(100) as i64;

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

        let projects = rows.into_iter().map(Into::into).collect();

        Ok(projects)
    }

    pub async fn get_all_cursors(
        db_context: &DBContext,
        limit: u32,
    ) -> Result<Vec<Option<i64>>, sqlx::Error> {
        let limit_param = limit.min(100) as i64;

        let cursors: Vec<i64> = sqlx::query_scalar!(
            "SELECT id FROM (
                SELECT id, ROW_NUMBER() OVER (ORDER BY id) as row_num
                FROM projects
             )
             WHERE (row_num - 1) % ? = 0
            ",
            limit_param
        )
        .fetch_all(db_context.pool())
        .await?;

        let cursors = cursors.into_iter().map(Some).collect();

        Ok(cursors)
    }

    pub async fn search_project(
        db_context: &DBContext,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Project>, sqlx::Error> {
        let limit = limit.min(100) as i64;

        let formatted_query = format!("{}*", query);

        let rows: Vec<ServerProject> = sqlx::query_as!(
            ServerProject,
            "
            SELECT p.id, p.name, p.root_path
            FROM projects_fts fts
            JOIN projects p ON fts.rowid = p.id
            WHERE fts.name MATCH ?
            ORDER BY bm25('fts') ASC
            LIMIT ?
            ",
            formatted_query,
            limit
        )
        .fetch_all(db_context.pool())
        .await?;

        let projects = rows.into_iter().map(Into::into).collect();

        Ok(projects)
    }
}
