use crate::{error::DBError, server::projects::cursor::ProjectCursor, DBContext};
use chrono::Utc;
use common::models::Project;
use uuid::Uuid;
pub mod cursor;

#[derive(Debug, sqlx::FromRow)]
pub struct ServerProject {
    pub id: Uuid,
    pub name: String,
    pub root_path: Option<String>,
    pub last_updated: Option<i64>,
}

impl From<ServerProject> for Project {
    fn from(value: ServerProject) -> Self {
        Self {
            id: value.id,
            name: value.name,
            root_path: value.root_path,
            last_updated: value.last_updated,
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
                root_path,
                last_updated
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
        after: Option<ProjectCursor>,
        limit: u32,
    ) -> Result<Vec<Project>, DBError> {
        let limit = limit.min(100) as i64;

        let rows: Vec<Self> = if let Some(cursor) = after {
            sqlx::query_as!(
                Self,
                r#"
                SELECT
                    id  AS "id: Uuid",
                    name,
                    root_path,
                    last_updated
                FROM projects
                WHERE
                  (last_updated < ?)
                  OR (last_updated = ? AND id < ?)
                ORDER BY last_updated DESC
                LIMIT ?
                "#,
                cursor.last_updated,
                cursor.last_updated,
                cursor.id,
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
                    root_path,
                    last_updated
                FROM projects
                ORDER BY last_updated DESC
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
    pub async fn get_page_cursors(
        db_context: &DBContext,
        limit: u32,
    ) -> Result<Vec<String>, DBError> {
        let limit = limit.min(100) as i64;

        let rows = sqlx::query!(
            r#"
            SELECT id AS "id: Uuid",
                   last_updated AS "last_updated: i64"
            FROM (
              SELECT
                id,
                last_updated,
                ROW_NUMBER() OVER (ORDER BY last_updated DESC) AS row_num
                FROM projects
            )
            WHERE (row_num - 1) % ? = 0
            ORDER BY last_updated DESC
            "#,
            limit
        )
        .fetch_all(db_context.pool())
        .await?;

        let cursors = rows
            .into_iter()
            .map(|r| {
                ProjectCursor {
                    last_updated: r.last_updated.unwrap_or(0),
                    id: r.id,
                }
                .encode()
            })
            .collect();

        Ok(cursors)
    }

    pub async fn total_pages(db_context: &DBContext, limit: u32) -> Result<u32, DBError> {
        let limit = limit.min(100) as i64;
        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM projects")
            .fetch_one(db_context.pool())
            .await?;
        Ok(((total + limit - 1) / limit) as u32)
    }

    pub async fn search_project(
        db_context: &DBContext,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Project>, DBError> {
        let limit = limit.min(100) as i64;

        let escaped_query = query.replace('"', "\"\"");
        let formatted_query = format!("\"{}\"*", escaped_query);

        let sql_query = sqlx::query_as!(
            Self,
            r#"
            SELECT
                p.id    AS "id: Uuid",
                p.name,
                p.root_path,
                p.last_updated
            FROM projects_fts fts
            JOIN projects_fts_map m ON m.docid = fts.rowid
            JOIN projects p ON p.id = m.project_id
            WHERE fts.name MATCH ?
            ORDER BY fts.rank ASC
            LIMIT ?
            "#,
            formatted_query,
            limit
        );

        #[cfg(debug_assertions)]
        {
            use crate::utils::explain_query;
            use log::warn;
            use sqlx::Execute;

            let sql = sql_query.sql();
            if let Err(e) = explain_query(db_context.pool(), sql).await {
                warn!("Failed to explain project search query: {}", e);
            }
        }

        let rows: Vec<ServerProject> = sql_query.fetch_all(db_context.pool()).await?;

        let projects = rows.into_iter().map(Into::into).collect();

        Ok(projects)
    }
}
