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

#[tokio::test]
async fn no_overlap_returns_zero() {
    let (pool, ids) = fresh_pool().await;

    let ev_start = 2_000_000_000;
    let ev_end = ev_start + 120; // 2 min
    insert_event(&pool, &ids, ev_start, ev_end).await;

    let range_start = ev_start - 300; // 5 min before
    let range_end = ev_start - 60; // ends 1 min before event starts

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(") AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 0);
}

#[tokio::test]
async fn full_containment_counts_full_event() {
    let (pool, ids) = fresh_pool().await;

    let ev_start = 2_100_000_000;
    let ev_end = ev_start + 180; // 3 min
    insert_event(&pool, &ids, ev_start, ev_end).await;

    let range_start = ev_start - 60;
    let range_end = ev_end + 60;

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(") AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 180);
}

#[tokio::test]
async fn range_within_event_counts_range_length() {
    let (pool, ids) = fresh_pool().await;

    let ev_start = 2_200_000_000;
    let ev_end = ev_start + 600; // 10 min event
    insert_event(&pool, &ids, ev_start, ev_end).await;

    let range_start = ev_start + 120; // inside
    let range_end = ev_start + 300; // inside (3 min length)

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(") AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 180);
}

#[tokio::test]
async fn multiple_events_sum_overlaps() {
    let (pool, ids) = fresh_pool().await;

    // Event A: 00:00..00:03 (180s)
    let a_start = 2_300_000_000;
    let a_end = a_start + 180;
    insert_event(&pool, &ids, a_start, a_end).await;

    // Event B: 00:02..00:05 (180s)
    let b_start = a_start + 120;
    let b_end = b_start + 180;
    insert_event(&pool, &ids, b_start, b_end).await;

    // Range: 00:01..00:04 (180s). Overlaps:
    // - A ∩ range = 120s (00:01..00:03)
    // - B ∩ range = 120s (00:02..00:04)
    // Sum = 240
    let range_start = a_start + 60;
    let range_end = a_start + 240;

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(") AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 240);
}

#[tokio::test]
async fn touching_boundary_is_zero() {
    let (pool, ids) = fresh_pool().await;

    let ev_start = 2_400_000_000;
    let ev_end = ev_start + 60; // 1 min
    insert_event(&pool, &ids, ev_start, ev_end).await;

    let range_start = ev_end; // exactly at event end
    let range_end = ev_end + 120;

    let mut qb = QueryBuilder::<Sqlite>::new("SELECT SUM(");
    push_overlap_with(
        &mut qb,
        |q| {
            q.push_bind(range_start);
        },
        |q| {
            q.push_bind(range_end);
        },
    );
    qb.push(") AS ov FROM events");

    let ov: i64 = qb.build_query_scalar().fetch_one(&pool).await.unwrap();
    assert_eq!(ov, 0);
}
