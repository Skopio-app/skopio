use std::path::Path;

use crate::DBContext;

pub fn extract_db_file_path(database_url: &str) -> std::path::PathBuf {
    let db_path = database_url.trim_start_matches("sqlite://");
    Path::new(db_path).to_path_buf()
}

pub(crate) async fn update_synced_in(
    db_context: &DBContext,
    table: &str,
    ids: &[i64],
) -> Result<(), sqlx::Error> {
    if ids.is_empty() {
        return Ok(());
    }

    let placeholders = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UPDATE {} SET synced = 1 WHERE id IN ({})",
        table, placeholders
    );

    let mut query_builder = sqlx::query(&query);

    for id in ids {
        query_builder = query_builder.bind(id);
    }

    query_builder.execute(db_context.pool()).await?;

    Ok(())
}
