#![cfg(all(test, feature = "server"))]

use db::server::utils::query::push_overlap_with;
use sqlx::{QueryBuilder, Sqlite};
mod helpers;
use helpers::*;

#[tokio::test]
async fn total_overlap_splits_at_midnight() {
    let (pool, ids) = fresh_pool().await;

    let start = 1_700_000_000;
    let end = start + 4 * 60;
    insert_event(&pool, &ids, start, end).await;

    let range_start = start + 2 * 60;
    let range_end = end;

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT ");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(" AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 120, "Should count only the 2 minutes after midnight");
}
