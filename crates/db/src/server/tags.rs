#![cfg(feature = "server")]
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

impl Tag {
    /// Insert a new tag
    pub async fn insert(pool: &SqlitePool, tag: Tag) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT OR IGNORE INTO TAGS (name) VALUES (?)", tag.name)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Fetch all tags
    pub async fn fetch_all(pool: &SqlitePool) -> Result<Vec<Tag>, sqlx::Error> {
        let rows = sqlx::query_as!(Tag, "SELECT id, name FROM tags")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    /// Fetch a tag by name
    pub async fn fetch_by_name(
        pool: &SqlitePool,
        tag_name: &str,
    ) -> Result<Option<Tag>, sqlx::Error> {
        sqlx::query_as!(Tag, "SELECT id, name FROM tags WHERE name = ?", tag_name)
            .fetch_optional(pool)
            .await
    }

    /// Associate a tag with an event via the `event_tags` table
    pub async fn associate_tag_with_event(
        pool: &SqlitePool,
        event_id: i64,
        tag_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR IGNORE INTO event_tags (event_id, tag_id) VALUES (?, ?)",
            event_id,
            tag_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Fetch all tags associated with a specifiec event
    pub async fn fetch_tags_for_event(
        pool: &SqlitePool,
        event_id: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Tag,
            "SELECT t.id, t.name
            FROM tags t
            JOIN event_tags et ON t.id = et.tag_id
            WHERE et.event_id = ?",
            event_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
