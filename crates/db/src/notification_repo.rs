// S-160-T4b: notification repository — insert/list/mark-read + push-token persistence (ADR-018)
use std::str::FromStr;

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

// ---------------------------------------------------------------------------
// Domain types — reference-only payload; no freeform/PII fields (ADR-018)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationKind {
    ReviewTaskAssigned,
    ReviewTaskDecided,
    ReviewTaskPublished,
}

impl FromStr for NotificationKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "review_task_assigned" => Ok(Self::ReviewTaskAssigned),
            "review_task_decided" => Ok(Self::ReviewTaskDecided),
            "review_task_published" => Ok(Self::ReviewTaskPublished),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ReviewTaskAssigned => "review_task_assigned",
            Self::ReviewTaskDecided => "review_task_decided",
            Self::ReviewTaskPublished => "review_task_published",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefEntityType {
    ReviewTask,
}

impl FromStr for RefEntityType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "review_task" => Ok(Self::ReviewTask),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for RefEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ReviewTask => "review_task",
        })
    }
}

#[derive(Debug, Clone)]
pub struct NotificationRow {
    pub id: Uuid,
    pub recipient_subject_id: Uuid,
    pub kind: NotificationKind,
    pub ref_entity_type: RefEntityType,
    pub ref_entity_id: Uuid,
    pub actor_subject_id: Option<Uuid>,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct PushTokenRow {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub provider: String,
    pub device_token: String,
    pub platform: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// ---------------------------------------------------------------------------
// Internal DB row types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct NotificationRowDb {
    id: Uuid,
    recipient_subject_id: Uuid,
    notification_kind: String,
    ref_entity_type: String,
    ref_entity_id: Uuid,
    actor_subject_id: Option<Uuid>,
    read_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct PushTokenRowDb {
    id: Uuid,
    subject_id: Uuid,
    provider: String,
    device_token: String,
    platform: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

// ---------------------------------------------------------------------------
// Fail-closed parsers
// ---------------------------------------------------------------------------

fn parse_notification_kind(s: &str) -> Result<NotificationKind, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "notifications.notification_kind",
        value: s.to_owned(),
    })
}

fn parse_ref_entity_type(s: &str) -> Result<RefEntityType, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "notifications.ref_entity_type",
        value: s.to_owned(),
    })
}

fn notification_from_db(r: NotificationRowDb) -> Result<NotificationRow, DbError> {
    Ok(NotificationRow {
        id: r.id,
        recipient_subject_id: r.recipient_subject_id,
        kind: parse_notification_kind(&r.notification_kind)?,
        ref_entity_type: parse_ref_entity_type(&r.ref_entity_type)?,
        ref_entity_id: r.ref_entity_id,
        actor_subject_id: r.actor_subject_id,
        read_at: r.read_at,
        created_at: r.created_at,
    })
}

fn push_token_from_db(r: PushTokenRowDb) -> PushTokenRow {
    PushTokenRow {
        id: r.id,
        subject_id: r.subject_id,
        provider: r.provider,
        device_token: r.device_token,
        platform: r.platform,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Notification operations
// ---------------------------------------------------------------------------

pub async fn insert_notification(pool: &PgPool, row: &NotificationRow) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO notifications (
            id, recipient_subject_id, notification_kind,
            ref_entity_type, ref_entity_id, actor_subject_id
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(row.id)
    .bind(row.recipient_subject_id)
    .bind(row.kind.to_string())
    .bind(row.ref_entity_type.to_string())
    .bind(row.ref_entity_id)
    .bind(row.actor_subject_id)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn list_notifications_for_recipient(
    pool: &PgPool,
    recipient_subject_id: Uuid,
) -> Result<Vec<NotificationRow>, DbError> {
    let rows = sqlx::query_as::<_, NotificationRowDb>(
        r#"
        SELECT id, recipient_subject_id, notification_kind,
               ref_entity_type, ref_entity_id, actor_subject_id, read_at, created_at
        FROM notifications
        WHERE recipient_subject_id = $1
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(recipient_subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(notification_from_db).collect()
}

/// Sets `read_at = now()` only on rows owned by `recipient_subject_id`.
/// Rows belonging to other recipients are never touched.
pub async fn mark_notifications_read(
    pool: &PgPool,
    recipient_subject_id: Uuid,
    ids: &[Uuid],
) -> Result<(), DbError> {
    if ids.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        UPDATE notifications
        SET read_at = now()
        WHERE id = ANY($1)
          AND recipient_subject_id = $2
          AND read_at IS NULL
        "#,
    )
    .bind(ids)
    .bind(recipient_subject_id)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Push-token operations
// ---------------------------------------------------------------------------

pub async fn insert_push_token(pool: &PgPool, row: &PushTokenRow) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO push_tokens (id, subject_id, provider, device_token, platform, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(row.id)
    .bind(row.subject_id)
    .bind(&row.provider)
    .bind(&row.device_token)
    .bind(&row.platform)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn list_push_tokens_for_subject(
    pool: &PgPool,
    subject_id: Uuid,
) -> Result<Vec<PushTokenRow>, DbError> {
    let rows = sqlx::query_as::<_, PushTokenRowDb>(
        r#"
        SELECT id, subject_id, provider, device_token, platform, created_at, updated_at
        FROM push_tokens
        WHERE subject_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(rows.into_iter().map(push_token_from_db).collect())
}

// ---------------------------------------------------------------------------
// Unit tests — fail-closed parsers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_kind_known_variants_round_trip() {
        for (s, expected) in [
            ("review_task_assigned", NotificationKind::ReviewTaskAssigned),
            ("review_task_decided", NotificationKind::ReviewTaskDecided),
            (
                "review_task_published",
                NotificationKind::ReviewTaskPublished,
            ),
        ] {
            let parsed: NotificationKind = s.parse().expect(s);
            assert_eq!(parsed, expected);
            assert_eq!(parsed.to_string(), s);
        }
    }

    #[test]
    fn notification_kind_unknown_value_fails_closed() {
        let err = parse_notification_kind("review_task_viewed").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "notifications.notification_kind",
                ..
            }
        ));
        assert!(err.to_string().contains("review_task_viewed"));
    }

    #[test]
    fn ref_entity_type_known_variant_round_trips() {
        let parsed: RefEntityType = "review_task".parse().expect("review_task");
        assert_eq!(parsed, RefEntityType::ReviewTask);
        assert_eq!(parsed.to_string(), "review_task");
    }

    #[test]
    fn ref_entity_type_unknown_value_fails_closed() {
        let err = parse_ref_entity_type("asset").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "notifications.ref_entity_type",
                ..
            }
        ));
        assert!(err.to_string().contains("asset"));
    }

    #[test]
    fn notification_from_db_unknown_kind_fails_closed() {
        let row = NotificationRowDb {
            id: Uuid::new_v4(),
            recipient_subject_id: Uuid::new_v4(),
            notification_kind: "unknown_kind".to_string(),
            ref_entity_type: "review_task".to_string(),
            ref_entity_id: Uuid::new_v4(),
            actor_subject_id: None,
            read_at: None,
            created_at: OffsetDateTime::now_utc(),
        };
        let err = notification_from_db(row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "notifications.notification_kind",
                ..
            }
        ));
    }

    #[test]
    fn notification_from_db_unknown_ref_entity_type_fails_closed() {
        let row = NotificationRowDb {
            id: Uuid::new_v4(),
            recipient_subject_id: Uuid::new_v4(),
            notification_kind: "review_task_assigned".to_string(),
            ref_entity_type: "asset".to_string(),
            ref_entity_id: Uuid::new_v4(),
            actor_subject_id: None,
            read_at: None,
            created_at: OffsetDateTime::now_utc(),
        };
        let err = notification_from_db(row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "notifications.ref_entity_type",
                ..
            }
        ));
    }
}
