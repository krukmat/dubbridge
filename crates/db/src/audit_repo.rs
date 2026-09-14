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
    platform_ingest_session_id: Option<Uuid>,
    correlation_id: Option<Uuid>,
    publication_id: Option<Uuid>,
    lineage_id: Option<Uuid>,
}

fn parse_event_kind(value: &str) -> Result<AuditEventKind, DbError> {
    match value {
        "ingestion_finalized" => Ok(AuditEventKind::IngestionFinalized),
        "ingestion_rejected_missing_rights" => Ok(AuditEventKind::IngestionRejectedMissingRights),
        "ingestion_rejected_missing_uploader_context" => Ok(AuditEventKind::IngestionRejectedMissingUploaderContext),
        "ingestion_rejected_duplicate_token" => Ok(AuditEventKind::IngestionRejectedDuplicateToken),
        "recording_session_created" => Ok(AuditEventKind::RecordingSessionCreated),
        "recording_rejected_missing_rights" => Ok(AuditEventKind::RecordingRejectedMissingRights),
        "recording_capture_started" => Ok(AuditEventKind::RecordingCaptureStarted),
        "recording_recorded" => Ok(AuditEventKind::RecordingRecorded),
        "recording_failed" => Ok(AuditEventKind::RecordingFailed),
        "recording_bridged_to_asset" => Ok(AuditEventKind::RecordingBridgedToAsset),
        "platform_ingest_session_created" => Ok(AuditEventKind::PlatformIngestSessionCreated),
        "platform_ingest_rejected_missing_rights" => Ok(AuditEventKind::PlatformIngestRejectedMissingRights),
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
        "auth_login_succeeded" => Ok(AuditEventKind::AuthLoginSucceeded),
        "auth_login_failed" => Ok(AuditEventKind::AuthLoginFailed),
        "auth_registered" => Ok(AuditEventKind::AuthRegistered),
        "p2p_publication_intent_created" => Ok(AuditEventKind::P2pPublicationIntentCreated),
        "p2p_lineage_sealed" => Ok(AuditEventKind::P2pLineageSealed),
        "p2p_publication_confirmed" => Ok(AuditEventKind::P2pPublicationConfirmed),
        "p2p_publication_reconciliation_entered" => Ok(AuditEventKind::P2pPublicationReconciliationEntered),
        "p2p_publication_ready" => Ok(AuditEventKind::P2pPublicationReady),
        "p2p_publication_failed" => Ok(AuditEventKind::P2pPublicationFailed),
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
        platform_ingest_session_id: row.platform_ingest_session_id,
        correlation_id: row.correlation_id,
        publication_id: row.publication_id,
        lineage_id: row.lineage_id,
        detail: row.detail,
        happened_at: row.happened_at,
    })
}

pub async fn insert_audit_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &AuditEvent,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (
            id, asset_id, event_kind, ingest_token, detail, happened_at,
            recording_session_id, platform_ingest_session_id,
            correlation_id, publication_id, lineage_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .bind(event.recording_session_id)
    .bind(event.platform_ingest_session_id)
    .bind(event.correlation_id)
    .bind(event.publication_id)
    .bind(event.lineage_id)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn insert_audit_event(pool: &PgPool, event: &AuditEvent) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (
            id, asset_id, event_kind, ingest_token, detail, happened_at,
            recording_session_id, platform_ingest_session_id,
            correlation_id, publication_id, lineage_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .bind(event.recording_session_id)
    .bind(event.platform_ingest_session_id)
    .bind(event.correlation_id)
    .bind(event.publication_id)
    .bind(event.lineage_id)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn list_audit_events_for_owned_asset(
    pool: &PgPool,
    asset_id: AssetId,
    owner_id: Uuid,
) -> Result<Vec<AuditEvent>, DbError> {
    let owned: Option<i32> = sqlx::query_scalar("SELECT 1 FROM assets WHERE id = $1 AND uploader_id = $2")
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
        SELECT id, asset_id, event_kind, ingest_token, detail, happened_at,
               recording_session_id, platform_ingest_session_id,
               correlation_id, publication_id, lineage_id
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

    fn base_row(event_kind: &str) -> AuditEventRow {
        AuditEventRow {
            id: Uuid::new_v4(),
            asset_id: None,
            event_kind: event_kind.to_string(),
            ingest_token: None,
            detail: None,
            happened_at: OffsetDateTime::now_utc(),
            recording_session_id: None,
            platform_ingest_session_id: None,
            correlation_id: None,
            publication_id: None,
            lineage_id: None,
        }
    }

    #[test]
    fn p2p_row_round_trips_correlation() {
        let asset_id = Uuid::new_v4();
        let publication_id = Uuid::new_v4();
        let lineage_id = Uuid::new_v4();
        let mut row = base_row("p2p_publication_confirmed");
        row.asset_id = Some(asset_id);
        row.correlation_id = Some(publication_id);
        row.publication_id = Some(publication_id);
        row.lineage_id = Some(lineage_id);

        let event = row_to_event(row).expect("event");
        assert_eq!(event.asset_id, Some(AssetId(asset_id)));
        assert_eq!(event.publication_id, Some(publication_id));
        assert_eq!(event.lineage_id, Some(lineage_id));
        assert!(event.has_valid_p2p_correlation());
    }
}
