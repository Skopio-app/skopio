use crate::{error::DBError, DBContext};
use chrono::Utc;
use common::models::Project;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ServerProject {
    pub id: Uuid,
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
    ) -> Result<Uuid, DBError> {
        let record = sqlx::query!("SELECT id FROM projects WHERE name = ?", name)
            .fetch_optional(db_context.pool())
            .await?;

        let timestamp = Utc::now().timestamp();

        if let Some(row) = record {
            sqlx::query!(
                "UPDATE projects SET last_updated = ? WHERE id = ?",
                timestamp,
                row.id
            )
            .execute(db_context.pool())
            .await?;
            let id = Uuid::from_slice(&row.id)?;
            return Ok(id);
        }

        let id = uuid::Uuid::now_v7();
        let result = sqlx::query!(
            "INSERT INTO projects (id, name, root_path, last_updated) VALUES (?, ?, ?, ?) RETURNING id",
            id,
            name,
            root_path,
            timestamp
        )
        .fetch_one(db_context.pool())
        .await?;

        let result_id = Uuid::from_slice(&result.id)?;
        Ok(result_id)
    }

    /// Fetches a project by id
    pub async fn find_by_id(db_context: &DBContext, id: Uuid) -> Result<Option<Project>, DBError> {
        let result = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id  AS "id: Uuid",
                name,
                root_path
            FROM projects
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(db_context.pool())
        .await?;

        Ok(result.map(Into::into))
    }

    pub async fn fetch_paginated(
        db_context: &DBContext,
        after_id: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<Project>, DBError> {
        let limit = limit.min(100) as i64;

        let rows: Vec<Self> = if let Some(cursor) = after_id {
            sqlx::query_as!(
                Self,
                r#"
                SELECT
                    id  AS "id: Uuid",
                    name,
                    root_path
                FROM projects
                WHERE id > ?
                ORDER BY last_updated
                LIMIT ?
                "#,
                cursor,
                limit
            )
            .fetch_all(db_context.pool())
            .await?
        } else {
            sqlx::query_as!(
                Self,
                r#"
                SELECT
                    id  AS "id: Uuid",
                    name,
                    root_path
                FROM projects
                ORDER BY last_updated
                LIMIT ?
                "#,
                limit
            )
            .fetch_all(db_context.pool())
            .await?
        };

        let projects = rows.into_iter().map(Into::into).collect();

        Ok(projects)
    }

    // TODO: Investigate cursor returning empty projects
    pub async fn get_all_cursors(
        db_context: &DBContext,
        limit: u32,
    ) -> Result<Vec<Option<Uuid>>, DBError> {
        let limit_param = limit.min(100) as i64;

        let cursors: Vec<Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id AS "id: Uuid"
            FROM (
                SELECT id, ROW_NUMBER() OVER (ORDER BY id) as row_num
                FROM projects
             )
             WHERE (row_num - 1) % ? = 0
            "#,
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
    ) -> Result<Vec<Project>, DBError> {
        let limit = limit.min(100) as i64;

        let escaped_query = query.replace('"', "\"\"");
        let formatted_query = format!("\"{}\"*", escaped_query);

        let rows: Vec<ServerProject> = sqlx::query_as!(
            Self,
            r#"
            SELECT
                p.id    AS "id: Uuid",
                p.name,
                p.root_path
            FROM projects_fts fts
            JOIN projects_fts_map m ON m.docid = fts.rowid
            JOIN projects p ON p.id = m.project_id
            WHERE fts.name MATCH ?
            ORDER BY fts.rank ASC
            LIMIT ?
            "#,
            formatted_query,
            limit
        )
        .fetch_all(db_context.pool())
        .await?;

        let projects = rows.into_iter().map(Into::into).collect();

        Ok(projects)
    }
}
