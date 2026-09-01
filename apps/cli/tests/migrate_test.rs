// S-230-T2-c: integration test proving the migration runner applies all
// infra/migrations and is idempotent on a second run. Mirrors the
// setup_pool() pattern used throughout apps/api/tests/*.rs.
use std::env;

use sqlx::{PgPool, Row};

async fn connect() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    Some(PgPool::connect(&url).await.expect("connect"))
}

// HP-1: against a reachable database, all 31 migration files apply and are
// tracked in _sqlx_migrations.
// HP-2: a second run against the same already-migrated database is a
// no-op and still returns Ok.
#[tokio::test]
async fn migrations_apply_and_are_idempotent_on_second_run() {
    let Some(pool) = connect().await else {
        return;
    };

    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("first migration run should succeed");

    let row = sqlx::query("SELECT count(*) AS count FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("query _sqlx_migrations count");
    let count: i64 = row.get("count");
    assert_eq!(
        count, 31,
        "expected exactly 31 applied migrations, found {count}"
    );

    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("second migration run should be a no-op and succeed");
}
