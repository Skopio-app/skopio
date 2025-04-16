use crate::utils::extract_db_file_path;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::str::FromStr;

#[cfg(all(feature = "desktop", not(feature = "server")))]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/desktop");

#[cfg(all(feature = "server", not(feature = "desktop")))]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/server");

// #[cfg(not(any(
//     all(feature = "desktop", not(feature = "server")),
//     all(feature = "server", not(feature = "desktop"))
// )))]
// compile_error!("You must enable either 'desktop' or 'server', but not both.");

#[derive(Clone)]
pub struct DBContext {
    pool: SqlitePool,
}

#[cfg(any(
    all(feature = "desktop", not(feature = "server")),
    all(feature = "server", not(feature = "desktop"))
))]
async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    MIGRATOR.run(pool).await
}

impl DBContext {
    /// Creates a new `DBContext` with an optional database URL.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        if let Some(parent_dir) = extract_db_file_path(database_url).parent() {
            std::fs::create_dir_all(parent_dir).expect("Failed to create database directory");
        }

        let connection_options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        #[cfg(any(
            all(feature = "desktop", not(feature = "server")),
            all(feature = "server", not(feature = "desktop"))
        ))]
        run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    /// Return a reference to the internal `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
