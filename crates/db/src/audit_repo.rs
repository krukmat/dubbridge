// T3: S1 repository — audit event insert per ADR-018
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
};

use crate::error::DbError;

#[derive(sqlx::FromRow)]
struct AuditEventRow {
    id: Uuid,
    asset_id: Option<Uuid>,
    event_kind: String,
    ingest_token: Option<Uuid>,
    detail: Option<String>,
    happened_at: OffsetDateTime,
    recording_session_id: Option<Uuid>,
}

fn parse_event_kind(value: &str) -> Result<AuditEventKind, DbError> {
    match value {
        "ingestion_finalized" => Ok(AuditEventKind::IngestionFinalized),
        "ingestion_rejected_missing_rights" => Ok(AuditEventKind::IngestionRejectedMissingRights),
        "ingestion_rejected_missing_uploader_context" => {
            Ok(AuditEventKind::IngestionRejectedMissingUploaderContext)
        }
        "ingestion_rejected_duplicate_token" => Ok(AuditEventKind::IngestionRejectedDuplicateToken),
        "recording_session_created" => Ok(AuditEventKind::RecordingSessionCreated),
        "recording_rejected_missing_rights" => Ok(AuditEventKind::RecordingRejectedMissingRights),
        "recording_capture_started" => Ok(AuditEventKind::RecordingCaptureStarted),
        "recording_recorded" => Ok(AuditEventKind::RecordingRecorded),
        "recording_failed" => Ok(AuditEventKind::RecordingFailed),
        "recording_bridged_to_asset" => Ok(AuditEventKind::RecordingBridgedToAsset),
        "platform_ingest_session_created" => Ok(AuditEventKind::PlatformIngestSessionCreated),
        "platform_ingest_rejected_missing_rights" => {
            Ok(AuditEventKind::PlatformIngestRejectedMissingRights)
        }
        "platform_ingest_download_started" => Ok(AuditEventKind::PlatformIngestDownloadStarted),
        "platform_ingest_downloaded" => Ok(AuditEventKind::PlatformIngestDownloaded),
        "platform_ingest_failed" => Ok(AuditEventKind::PlatformIngestFailed),
        "platform_ingest_bridged_to_asset" => Ok(AuditEventKind::PlatformIngestBridgedToAsset),
        "org_created" => Ok(AuditEventKind::OrgCreated),
        "org_member_added" => Ok(AuditEventKind::OrgMemberAdded),
        "project_created" => Ok(AuditEventKind::ProjectCreated),
        "consent_granted" => Ok(AuditEventKind::ConsentGranted),
        "consent_revoked" => Ok(AuditEventKind::ConsentRevoked),
        "consent_check_denied" => Ok(AuditEventKind::ConsentCheckDenied),
        "review_approved" => Ok(AuditEventKind::ReviewApproved),
        "review_rejected" => Ok(AuditEventKind::ReviewRejected),
        "publication_succeeded" => Ok(AuditEventKind::PublicationSucceeded),
        "publication_refused" => Ok(AuditEventKind::PublicationRefused),
        "playback_grant_issued" => Ok(AuditEventKind::PlaybackGrantIssued),
        "playback_grant_refused" => Ok(AuditEventKind::PlaybackGrantRefused),
        other => Err(DbError::UnknownStoredValue {
            field: "audit_events.event_kind",
            value: other.to_owned(),
        }),
    }
}

fn row_to_event(row: AuditEventRow) -> Result<AuditEvent, DbError> {
    Ok(AuditEvent {
        id: row.id,
        asset_id: row.asset_id.map(AssetId),
        event_kind: parse_event_kind(&row.event_kind)?,
        ingest_token: row.ingest_token,
        recording_session_id: row.recording_session_id,
        platform_ingest_session_id: None,
        detail: row.detail,
        happened_at: row.happened_at,
    })
}

// H1-T1: transaction-aware variant so the success audit row commits atomically
// with the asset, rights, and artifact rows.
pub async fn insert_audit_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &AuditEvent,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (id, asset_id, event_kind, ingest_token, detail, happened_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn insert_audit_event(pool: &PgPool, event: &AuditEvent) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (id, asset_id, event_kind, ingest_token, detail, happened_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

/// Returns the audit timeline for an owned asset in chronological order.
/// Fails closed with `DbError::NotFound` when the asset does not exist or is not
/// owned by `owner_id`.
pub async fn list_audit_events_for_owned_asset(
    pool: &PgPool,
    asset_id: AssetId,
    owner_id: Uuid,
) -> Result<Vec<AuditEvent>, DbError> {
    let owned: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM assets WHERE id = $1 AND uploader_id = $2")
            .bind(asset_id.0)
            .bind(owner_id)
            .fetch_optional(pool)
            .await
            .map_err(DbError::QueryFailed)?;

    if owned.is_none() {
        return Err(DbError::NotFound);
    }

    let rows = sqlx::query_as::<_, AuditEventRow>(
        r#"
        SELECT id, asset_id, event_kind, ingest_token, detail, happened_at, recording_session_id
        FROM audit_events
        WHERE asset_id = $1
        ORDER BY happened_at ASC, id ASC
        "#,
    )
    .bind(asset_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(row_to_event).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_event_kind_known_variants_succeed() {
        assert!(matches!(
            parse_event_kind("consent_granted"),
            Ok(AuditEventKind::ConsentGranted)
        ));
        assert!(matches!(
            parse_event_kind("recording_recorded"),
            Ok(AuditEventKind::RecordingRecorded)
        ));
        assert!(matches!(
            parse_event_kind("org_created"),
            Ok(AuditEventKind::OrgCreated)
        ));
        assert!(matches!(
            parse_event_kind("review_approved"),
            Ok(AuditEventKind::ReviewApproved)
        ));
        assert!(matches!(
            parse_event_kind("playback_grant_issued"),
            Ok(AuditEventKind::PlaybackGrantIssued)
        ));
        assert!(matches!(
            parse_event_kind("playback_grant_refused"),
            Ok(AuditEventKind::PlaybackGrantRefused)
        ));
    }

    #[test]
    fn parse_event_kind_unknown_value_fails_closed() {
        let err = parse_event_kind("totally_new_event").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "audit_events.event_kind",
                ..
            }
        ));
        assert!(err.to_string().contains("totally_new_event"));
    }

    #[test]
    fn row_to_event_round_trips_recording_fields() {
        let asset_id = Uuid::new_v4();
        let recording_session_id = Uuid::new_v4();
        let row = AuditEventRow {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            event_kind: "recording_recorded".to_string(),
            ingest_token: None,
            detail: Some("ok".to_string()),
            happened_at: OffsetDateTime::now_utc(),
            recording_session_id: Some(recording_session_id),
        };

        let event = row_to_event(row).expect("event");
        assert_eq!(event.asset_id, Some(AssetId(asset_id)));
        assert_eq!(event.event_kind, AuditEventKind::RecordingRecorded);
        assert_eq!(event.recording_session_id, Some(recording_session_id));
        assert!(event.platform_ingest_session_id.is_none());
    }

    #[test]
    fn row_to_event_round_trips_playback_kind() {
        let asset_id = Uuid::new_v4();
        let row = AuditEventRow {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            event_kind: "playback_grant_refused".to_string(),
            ingest_token: None,
            detail: Some("reason=asset_not_ready".to_string()),
            happened_at: OffsetDateTime::now_utc(),
            recording_session_id: None,
        };

        let event = row_to_event(row).expect("event");
        assert_eq!(event.asset_id, Some(AssetId(asset_id)));
        assert_eq!(event.event_kind, AuditEventKind::PlaybackGrantRefused);
        assert_eq!(event.detail.as_deref(), Some("reason=asset_not_ready"));
        assert!(event.recording_session_id.is_none());
        assert!(event.platform_ingest_session_id.is_none());
    }
}
