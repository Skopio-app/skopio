use std::str::FromStr;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use crate::utils::extract_db_file_path;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct DBContext {
    pool: SqlitePool,
}

impl DBContext {
    /// Creates a new `DBContext` with an optional database URL.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        if let Some(parent_dir) = extract_db_file_path(database_url).parent() {
            std::fs::create_dir_all(parent_dir)
                .expect("Failed to create database directory");
        }

        let connection_options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        MIGRATOR.run(&pool).await?;

        Ok(Self { pool })
    }

    /// Return a reference to the internal `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
