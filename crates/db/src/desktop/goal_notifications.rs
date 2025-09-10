use crate::{desktop::goals::TimeSpan, error::DBError, DBContext};

pub async fn has_shown_goal_notification(
    db: &DBContext,
    goal_id: i64,
    time_span: &TimeSpan,
    period_key: &str,
) -> Result<bool, DBError> {
    let time_span = time_span.to_string();
    let exists = sqlx::query_scalar!(
        "
        SELECT EXISTS (
            SELECT 1 FROM shown_goal_notifications
            WHERE goal_id = ? AND TIME_SPAN = ? AND period_key = ?
        )
        ",
        goal_id,
        time_span,
        period_key
    )
    .fetch_one(db.pool())
    .await?;

    Ok(exists == 1)
}

pub async fn insert_shown_goal_notification(
    db: &DBContext,
    goal_id: i64,
    time_span: &TimeSpan,
    period_key: &str,
) -> Result<(), DBError> {
    let time_span = time_span.to_string();
    sqlx::query!(
        "
        INSERT INTO shown_goal_notifications (goal_id, time_span, period_key)
        VALUES (?, ?, ?)
        ",
        goal_id,
        time_span,
        period_key,
    )
    .execute(db.pool())
    .await?;

    Ok(())
}
