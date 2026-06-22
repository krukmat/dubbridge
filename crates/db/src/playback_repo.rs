// S-125-T2b-i/ii: grant lifecycle CRUD and readiness-gate resolution (ADR-032)
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    artifact::{ArtifactKind, DerivedArtifact, PreparationStatus},
    asset::AssetId,
    playback::{
        GrantPrincipal, GrantStatus, PlaybackDenial, PlaybackGrant, PlaybackGrantId, PlaybackScope,
    },
    workspace::{OrgId, ProjectId},
};

use crate::error::DbError;

// ── T2b-i: grant lifecycle CRUD ───────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct PlaybackGrantRow {
    grant_id: Uuid,
    asset_id: Uuid,
    principal_id: Uuid,
    org_id: Uuid,
    project_id: Uuid,
    scope: String,
    status: String,
    issued_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

fn parse_status(s: &str) -> Result<GrantStatus, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "playback_grants.status",
        value: s.to_owned(),
    })
}

fn parse_scope(s: &str) -> Result<PlaybackScope, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "playback_grants.scope",
        value: s.to_owned(),
    })
}

fn grant_from_row(r: PlaybackGrantRow) -> Result<PlaybackGrant, DbError> {
    Ok(PlaybackGrant {
        id: PlaybackGrantId(r.grant_id),
        asset_id: AssetId(r.asset_id),
        scope: parse_scope(&r.scope)?,
        principal: GrantPrincipal {
            principal_id: r.principal_id,
            org_id: OrgId(r.org_id),
            project_id: ProjectId(r.project_id),
        },
        status: parse_status(&r.status)?,
        issued_at: r.issued_at,
        expires_at: r.expires_at,
    })
}

/// Persist a new playback grant. The grant must be `Active` at the call site.
pub async fn issue_grant(pool: &PgPool, grant: &PlaybackGrant) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO playback_grants
            (grant_id, asset_id, principal_id, org_id, project_id, scope, status, issued_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(grant.id.0)
    .bind(grant.asset_id.0)
    .bind(grant.principal.principal_id)
    .bind(grant.principal.org_id.0)
    .bind(grant.principal.project_id.0)
    .bind(grant.scope.to_string())
    .bind(grant.status.to_string())
    .bind(grant.issued_at)
    .bind(grant.expires_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

/// Return the grant if it is active and has not yet expired (wall-clock).
/// Returns `None` for unknown, expired-by-clock, or explicitly expired/revoked grants.
pub async fn get_active_grant(
    pool: &PgPool,
    grant_id: PlaybackGrantId,
) -> Result<Option<PlaybackGrant>, DbError> {
    let row = sqlx::query_as::<_, PlaybackGrantRow>(
        r#"
        SELECT grant_id, asset_id, principal_id, org_id, project_id, scope, status, issued_at, expires_at
          FROM playback_grants
         WHERE grant_id = $1
           AND status = 'active'
           AND expires_at > now()
        "#,
    )
    .bind(grant_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(grant_from_row).transpose()
}

/// Mark an active grant as expired. No-op (no error) if already expired or revoked.
pub async fn expire_grant(pool: &PgPool, grant_id: PlaybackGrantId) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE playback_grants
           SET status = 'expired'
         WHERE grant_id = $1 AND status = 'active'
        "#,
    )
    .bind(grant_id.0)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

// ── T2b-ii: resolve_grant_target ─────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct PreparationStatusRow {
    status: String,
}

#[derive(sqlx::FromRow)]
struct HlsManifestRow {
    id: Uuid,
    asset_id: Uuid,
    parent_artifact_id: Uuid,
    storage_key: String,
    content_type: String,
    size_bytes: i64,
    checksum: String,
    created_at: OffsetDateTime,
}

fn parse_preparation_status(s: &str) -> Result<PreparationStatus, DbError> {
    match s {
        "pending" => Ok(PreparationStatus::Pending),
        "in_progress" => Ok(PreparationStatus::InProgress),
        "ready" => Ok(PreparationStatus::Ready),
        "failed" => Ok(PreparationStatus::Failed),
        other => Err(DbError::UnknownStoredValue {
            field: "asset_preparation_status.status",
            value: other.to_owned(),
        }),
    }
}

/// Resolve an active grant to its asset's prepared HLS manifest artifact.
///
/// Fails closed at every step:
///  1. Grant not active or past expiry  → `PlaybackDenial::GrantInvalid`
///  2. Asset preparation status != Ready → `PlaybackDenial::NotReady`
///  3. No HLS manifest lineage row       → `PlaybackDenial::MissingManifest`
pub async fn resolve_grant_target(
    pool: &PgPool,
    grant_id: PlaybackGrantId,
) -> Result<DerivedArtifact, PlaybackDenial> {
    // Step 1: verify the grant is still active.
    let grant = get_active_grant(pool, grant_id)
        .await
        .map_err(|_| PlaybackDenial::GrantInvalid)?
        .ok_or(PlaybackDenial::GrantInvalid)?;

    // Step 2: check asset preparation status.
    let status_row = sqlx::query_as::<_, PreparationStatusRow>(
        r#"
        SELECT status
          FROM asset_preparation_status
         WHERE asset_id = $1
        "#,
    )
    .bind(grant.asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(|_| PlaybackDenial::NotReady)?
    .ok_or(PlaybackDenial::NotReady)?;

    let preparation_status =
        parse_preparation_status(&status_row.status).map_err(|_| PlaybackDenial::NotReady)?;

    if preparation_status != PreparationStatus::Ready {
        return Err(PlaybackDenial::NotReady);
    }

    // Step 3: fetch the HLS manifest lineage row.
    let manifest_row = sqlx::query_as::<_, HlsManifestRow>(
        r#"
        SELECT id, asset_id, parent_artifact_id, storage_key, content_type, size_bytes, checksum, created_at
          FROM artifact_records
         WHERE asset_id = $1
           AND kind = 'hls_manifest'
           AND parent_artifact_id IS NOT NULL
         ORDER BY created_at ASC
         LIMIT 1
        "#,
    )
    .bind(grant.asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(|_| PlaybackDenial::MissingManifest)?
    .ok_or(PlaybackDenial::MissingManifest)?;

    Ok(DerivedArtifact {
        id: manifest_row.id,
        asset_id: AssetId(manifest_row.asset_id),
        parent_artifact_id: manifest_row.parent_artifact_id,
        kind: ArtifactKind::HlsManifest,
        storage_key: manifest_row.storage_key,
        content_type: manifest_row.content_type,
        size_bytes: manifest_row.size_bytes,
        checksum: manifest_row.checksum,
        created_at: manifest_row.created_at,
    })
}
