use std::env;

use dubbridge_db::notification_repo::{
    self, NotificationKind, NotificationRow, PushTokenRow, RefEntityType,
};
use sqlx::PgPool;
use time::OffsetDateTime;
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

fn new_notification(recipient_subject_id: Uuid, ref_entity_id: Uuid) -> NotificationRow {
    NotificationRow {
        id: Uuid::new_v4(),
        recipient_subject_id,
        kind: NotificationKind::ReviewTaskAssigned,
        ref_entity_type: RefEntityType::ReviewTask,
        ref_entity_id,
        actor_subject_id: Some(Uuid::new_v4()),
        read_at: None,
        created_at: OffsetDateTime::now_utc(),
    }
}

fn new_push_token(subject_id: Uuid, device_token: &str) -> PushTokenRow {
    let now = OffsetDateTime::now_utc();
    PushTokenRow {
        id: Uuid::new_v4(),
        subject_id,
        provider: "expo".to_string(),
        device_token: device_token.to_string(),
        platform: "ios".to_string(),
        created_at: now,
        updated_at: now,
    }
}

// HP-1: insert notification → list for recipient → row appears unread with correct kind and ref
#[tokio::test]
async fn insert_and_list_notifications_returns_unread_row() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let recipient = Uuid::new_v4();
    let ref_id = Uuid::new_v4();
    let notification = new_notification(recipient, ref_id);

    notification_repo::insert_notification(&pool, &notification)
        .await
        .expect("insert notification");

    let rows = notification_repo::list_notifications_for_recipient(&pool, recipient)
        .await
        .expect("list notifications");

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, notification.id);
    assert_eq!(row.recipient_subject_id, recipient);
    assert_eq!(row.kind, NotificationKind::ReviewTaskAssigned);
    assert_eq!(row.ref_entity_type, RefEntityType::ReviewTask);
    assert_eq!(row.ref_entity_id, ref_id);
    assert!(row.read_at.is_none(), "new notification must be unread");
}

// HP-2: mark-read → read_at is set; re-list reflects the row as read
#[tokio::test]
async fn mark_notifications_read_sets_read_at() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let recipient = Uuid::new_v4();
    let notification = new_notification(recipient, Uuid::new_v4());

    notification_repo::insert_notification(&pool, &notification)
        .await
        .expect("insert notification");

    notification_repo::mark_notifications_read(&pool, recipient, &[notification.id])
        .await
        .expect("mark read");

    let rows = notification_repo::list_notifications_for_recipient(&pool, recipient)
        .await
        .expect("list after mark-read");

    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].read_at.is_some(),
        "read_at must be set after mark-read"
    );
}

// HP-3: insert push token → list by subject → token returned with correct provider/platform
#[tokio::test]
async fn insert_and_list_push_tokens_round_trips() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let subject_id = Uuid::new_v4();
    let device_token = format!("ExponentPushToken[repo-test-{}]", Uuid::new_v4());
    let token = new_push_token(subject_id, &device_token);

    notification_repo::insert_push_token(&pool, &token)
        .await
        .expect("insert push token");

    let tokens = notification_repo::list_push_tokens_for_subject(&pool, subject_id)
        .await
        .expect("list push tokens");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].id, token.id);
    assert_eq!(tokens[0].subject_id, subject_id);
    assert_eq!(tokens[0].provider, "expo");
    assert_eq!(tokens[0].device_token, device_token);
    assert_eq!(tokens[0].platform, "ios");
}

// EC-1: mark_notifications_read with IDs from another recipient → their read_at remains NULL
#[tokio::test]
async fn mark_notifications_read_does_not_touch_other_recipients() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let recipient_a = Uuid::new_v4();
    let recipient_b = Uuid::new_v4();

    let notif_a = new_notification(recipient_a, Uuid::new_v4());
    let notif_b = new_notification(recipient_b, Uuid::new_v4());

    notification_repo::insert_notification(&pool, &notif_a)
        .await
        .expect("insert notif_a");
    notification_repo::insert_notification(&pool, &notif_b)
        .await
        .expect("insert notif_b");

    // recipient_a tries to mark notif_b as read — must have no effect
    notification_repo::mark_notifications_read(&pool, recipient_a, &[notif_b.id])
        .await
        .expect("mark read (cross-recipient attempt)");

    let rows_b = notification_repo::list_notifications_for_recipient(&pool, recipient_b)
        .await
        .expect("list notif_b");

    assert_eq!(rows_b.len(), 1);
    assert!(
        rows_b[0].read_at.is_none(),
        "notif_b read_at must remain NULL; another recipient cannot mark it read"
    );
}

// EC-2: list for subject with no notifications → empty list, no error
#[tokio::test]
async fn list_notifications_for_unknown_recipient_returns_empty() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let unknown_recipient = Uuid::new_v4();
    let rows = notification_repo::list_notifications_for_recipient(&pool, unknown_recipient)
        .await
        .expect("list for unknown recipient");

    assert!(rows.is_empty());
}

// EC-3: upsert same (provider, device_token) from a new subject → subject_id updated, no error
#[tokio::test]
async fn upsert_push_token_reassigns_subject_on_conflict() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let device_token = format!("ExponentPushToken[upsert-test-{}]", Uuid::new_v4());
    let first_subject = Uuid::new_v4();
    let second_subject = Uuid::new_v4();

    let first = new_push_token(first_subject, &device_token);
    notification_repo::upsert_push_token(&pool, &first)
        .await
        .expect("upsert first push token");

    let second = new_push_token(second_subject, &device_token);
    notification_repo::upsert_push_token(&pool, &second)
        .await
        .expect("upsert same token for new subject must succeed");

    let tokens = notification_repo::list_push_tokens_for_subject(&pool, second_subject)
        .await
        .expect("list tokens for second subject");

    assert_eq!(tokens.len(), 1, "exactly one row after upsert");
    assert_eq!(tokens[0].device_token, device_token);
    assert_eq!(
        tokens[0].subject_id, second_subject,
        "subject_id must be updated"
    );
}
