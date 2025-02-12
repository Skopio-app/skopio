use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

#[derive(Clone)]
pub struct DBContext {
    pool: SqlitePool,
}

impl DBContext {
    /// Creates a new `DBContext` with an optional database URL.
    pub async fn new(database_url: Option<&str>) -> Result<Self, sqlx::Error> {
        let db_url = database_url.unwrap_or("sqlite://./data/timestack.db");

        let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

        Ok(Self { pool })
    }

    /// Return a reference to the internal `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
