use sqlx::sqlite::SqlitePoolOptions;
use sqlx:: SqlitePool;

pub async fn establish_connection(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
    .max_connections(5)
    .connect(database_url)
    .await
}
