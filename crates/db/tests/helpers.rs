use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SeedIds {
    pub app: [u8; 16],
    pub cat: [u8; 16],
    pub source: [u8; 16],
}

fn uuid_bytes(s: &str) -> [u8; 16] {
    *Uuid::parse_str(s).expect("valid UUID literal").as_bytes()
}

pub async fn fresh_pool() -> (SqlitePool, SeedIds) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        r#"
      PRAGMA foreign_keys = ON;

      CREATE TABLE apps      (id BLOB PRIMARY KEY, name TEXT UNIQUE);
      CREATE TABLE projects  (id BLOB PRIMARY KEY, name TEXT UNIQUE);
      CREATE TABLE entities  (id BLOB PRIMARY KEY, name TEXT UNIQUE, type TEXT);
      CREATE TABLE branches  (id BLOB PRIMARY KEY, name TEXT);
      CREATE TABLE languages (id BLOB PRIMARY KEY, name TEXT UNIQUE);
      CREATE TABLE categories(id BLOB PRIMARY KEY, name TEXT UNIQUE);
      CREATE TABLE sources   (id BLOB PRIMARY KEY, name TEXT UNIQUE);

      CREATE TABLE events (
        id            BLOB PRIMARY KEY,
        timestamp     INTEGER NOT NULL,
        duration      INTEGER DEFAULT 0,
        category_id   BLOB NOT NULL,
        app_id        BLOB NOT NULL,
        entity_id     BLOB,
        project_id    BLOB,
        branch_id     BLOB,
        language_id   BLOB,
        source_id     BLOB NOT NULL,
        end_timestamp INTEGER NOT NULL
      );

      CREATE INDEX idx_events_timestamp ON events(timestamp);
      CREATE INDEX idx_events_end       ON events(end_timestamp);
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let app_id = uuid_bytes("11111111-1111-1111-1111-111111111111");
    let cat_id = uuid_bytes("22222222-2222-2222-2222-222222222222");
    let source_id = uuid_bytes("33333333-3333-3333-3333-333333333333");

    sqlx::query("INSERT INTO apps(id,name) VALUES(?, ?)")
        .bind(app_id.as_slice())
        .bind("Code")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO categories(id,name) VALUES(?, ?)")
        .bind(cat_id.as_slice())
        .bind("Coding")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO sources(id,name) VALUES(?, ?)")
        .bind(source_id.as_slice())
        .bind("Desktop")
        .execute(&pool)
        .await
        .unwrap();

    let ids = SeedIds {
        app: app_id,
        cat: cat_id,
        source: source_id,
    };

    (pool, ids)
}

pub async fn insert_event(pool: &SqlitePool, ids: &SeedIds, start: i64, end: i64) {
    assert!(end >= start, "end must be >= start");
    let id = Uuid::new_v4();
    let duration = end - start;

    sqlx::query(
        r#"
        INSERT INTO events(
          id, timestamp, duration, category_id, app_id, source_id, end_timestamp
        ) VALUES(?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(start)
    .bind(duration)
    .bind(ids.cat.as_slice())
    .bind(ids.app.as_slice())
    .bind(ids.source.as_slice())
    .bind(end)
    .execute(pool)
    .await
    .unwrap();
}
