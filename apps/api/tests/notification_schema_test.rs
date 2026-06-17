use std::env;

use sqlx::PgPool;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

#[tokio::test]
async fn notifications_accept_reference_only_unread_rows() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let notification_id = Uuid::new_v4();
    let recipient_subject_id = Uuid::new_v4();
    let review_task_id = Uuid::new_v4();
    let actor_subject_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO notifications (
            id, recipient_subject_id, notification_kind, ref_entity_type, ref_entity_id, actor_subject_id
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(notification_id)
    .bind(recipient_subject_id)
    .bind("review_task_assigned")
    .bind("review_task")
    .bind(review_task_id)
    .bind(actor_subject_id)
    .execute(&pool)
    .await
    .expect("insert notification");

    let stored: (String, String, Option<time::OffsetDateTime>) = sqlx::query_as(
        "SELECT notification_kind, ref_entity_type, read_at FROM notifications WHERE id = $1",
    )
    .bind(notification_id)
    .fetch_one(&pool)
    .await
    .expect("stored notification");

    assert_eq!(stored.0, "review_task_assigned");
    assert_eq!(stored.1, "review_task");
    assert!(
        stored.2.is_none(),
        "read_at should default to NULL for unread"
    );
}

#[tokio::test]
async fn notifications_reject_unknown_kind() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let err = sqlx::query(
        "INSERT INTO notifications (
            id, recipient_subject_id, notification_kind, ref_entity_type, ref_entity_id
        ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind("review_task_viewed")
    .bind("review_task")
    .bind(Uuid::new_v4())
    .execute(&pool)
    .await
    .expect_err("unknown notification kind must fail");

    assert!(
        err.to_string().contains("notifications_kind_check"),
        "expected kind check failure, got {err}"
    );
}

#[tokio::test]
async fn notifications_reject_unknown_ref_entity_type() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let err = sqlx::query(
        "INSERT INTO notifications (
            id, recipient_subject_id, notification_kind, ref_entity_type, ref_entity_id
        ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind("review_task_decided")
    .bind("asset")
    .bind(Uuid::new_v4())
    .execute(&pool)
    .await
    .expect_err("unknown ref entity type must fail");

    assert!(
        err.to_string()
            .contains("notifications_ref_entity_type_check"),
        "expected ref entity type check failure, got {err}"
    );
}

#[tokio::test]
async fn push_tokens_accept_valid_rows_and_reject_duplicate_provider_device_pairs() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let device_token = "ExponentPushToken[test-token]";

    sqlx::query(
        "INSERT INTO push_tokens (id, subject_id, provider, device_token, platform)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind("expo")
    .bind(device_token)
    .bind("ios")
    .execute(&pool)
    .await
    .expect("insert push token");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM push_tokens WHERE device_token = $1")
        .bind(device_token)
        .fetch_one(&pool)
        .await
        .expect("push token count");
    assert_eq!(count, 1);

    let err = sqlx::query(
        "INSERT INTO push_tokens (id, subject_id, provider, device_token, platform)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind("expo")
    .bind(device_token)
    .bind("android")
    .execute(&pool)
    .await
    .expect_err("duplicate provider/device token must fail");

    assert!(
        err.to_string()
            .contains("push_tokens_provider_device_unique"),
        "expected push token uniqueness error, got {err}"
    );
}

#[tokio::test]
async fn notifications_schema_has_no_freeform_or_pii_columns() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name
         FROM information_schema.columns
         WHERE table_schema = 'public' AND table_name = 'notifications'
         ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .expect("notification columns");

    for forbidden in [
        "title",
        "asset_title",
        "body",
        "message",
        "detail",
        "payload_json",
        "comment",
        "reviewer_comment",
        "recipient_name",
        "actor_name",
    ] {
        assert!(
            !columns.iter().any(|column| column == forbidden),
            "notifications schema must not expose freeform or PII column {forbidden}"
        );
    }

    assert_eq!(
        columns,
        vec![
            "id",
            "recipient_subject_id",
            "notification_kind",
            "ref_entity_type",
            "ref_entity_id",
            "actor_subject_id",
            "read_at",
            "created_at",
        ]
    );
}
