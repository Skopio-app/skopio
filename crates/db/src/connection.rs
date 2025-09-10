use crate::error::DBError;
use crate::utils::extract_db_file_path;
use common::keyring::Keyring;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

#[cfg(all(feature = "desktop", not(feature = "server")))]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/desktop");

#[cfg(all(feature = "server", not(feature = "desktop")))]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/server");

#[derive(Clone)]
pub struct DBContext {
    pool: SqlitePool,
}

impl DBContext {
    /// Creates a new `DBContext` with a database URL.
    pub async fn new(database_url: &str) -> Result<Self, DBError> {
        if let Some(parent_dir) = extract_db_file_path(database_url).parent() {
            std::fs::create_dir_all(parent_dir).expect("Failed to create database directory");
        }

        let mut connection_options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        if let Some(encryption_key) = get_encryption_key()? {
            connection_options = connection_options.pragma("key", encryption_key)
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        #[cfg(any(
            all(feature = "desktop", not(feature = "server")),
            all(feature = "server", not(feature = "desktop"))
        ))]
        MIGRATOR.run(&pool).await?;

        Ok(Self { pool })
    }

    /// Return a reference to the internal `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn get_encryption_key() -> Result<Option<String>, DBError> {
    if cfg!(debug_assertions) {
        return Ok(None);
    }
    let password = Uuid::new_v4().to_string();
    let key = Keyring::get_or_set_password("skopio-database", "db-master-key", &password).map_err(
        |e| sqlx::Error::Configuration(format!("Failed to get or set encryption key: {e}").into()),
    )?;
    Ok(Some(key))
}
