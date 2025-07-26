use crate::DBContext;

#[derive(sqlx::FromRow)]
struct YearResult {
    year: Option<String>,
}

pub async fn get_active_years(db_context: &DBContext) -> Result<Vec<i32>, sqlx::Error> {
    let rows: Vec<YearResult> = sqlx::query_as!(
        YearResult,
        "
        SELECT DISTINCT strftime('%Y', datetime(timestamp, 'localtime')) as year
        FROM events
        ORDER BY year DESC
        "
    )
    .fetch_all(db_context.pool())
    .await?;

    let years = rows
        .into_iter()
        .filter_map(|row| row.year.and_then(|s| s.parse::<i32>().ok()))
        .collect();

    Ok(years)
}
