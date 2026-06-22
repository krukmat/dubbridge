use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::{AuthenticatedPrincipal, Hs256Issuer, parse_jwt};
use dubbridge_db::{error::DbError, playback_repo, preparation_repo};
use dubbridge_domain::{
    artifact::PreparationStatus,
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    playback::{GrantPrincipal, PlaybackGrant, PlaybackGrantId, PlaybackScope},
    workspace::{OrgId, OrgRole, ProjectId, parse_org_role},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tracing::debug;
use uuid::Uuid;

use crate::{
    playback_api_error::ApiError,
    playback_audit::playback_grant_refused_event,
    playback_policy::{PlaybackAudiencePolicyContext, apply_audience_policy_hook},
    state::AppState,
};
use dubbridge_playback::{manifest_segment_names, rewrite_manifest_with_refs};
use dubbridge_storage::{get_hls_manifest, get_hls_segment};

const PLAYBACK_GRANT_TTL_HOURS: i64 = 1;
const PLAYBACK_SEGMENT_TTL_SECONDS: u64 = 60;
const LOCAL_DEV_JWT_SECRET_PLACEHOLDER: &str = "local-dev-jwt-secret-placeholder";
const SEGMENT_SCOPE_PREFIX: &str = "playback_segment:";

#[derive(Debug, Deserialize)]
pub struct AssetPath {
    pub id: Uuid,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PlaybackGrantIssueResponse {
    pub grant_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PlaybackManifestPath {
    pub id: Uuid,
    pub grant_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PlaybackSegmentPath {
    pub id: Uuid,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct SegmentTokenQuery {
    pub token: String,
}

pub async fn issue_playback_grant(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(path): Path<AssetPath>,
) -> Result<(StatusCode, Json<PlaybackGrantIssueResponse>), ApiError> {
    let asset_id = AssetId(path.id);
    let (org_id, project_id) =
        match resolve_authorized_asset_scope(&state.pool, asset_id, principal.subject_id).await {
            Ok(scope) => scope,
            Err(error) => {
                if let Some(reason) = error.audit_reason() {
                    emit_refusal_audit(
                        &state.pool,
                        asset_id,
                        principal.subject_id,
                        None,
                        None,
                        reason,
                    )
                    .await?;
                }
                return Err(error);
            }
        };
    if let Err(error) = ensure_asset_ready_for_playback(&state.pool, asset_id).await {
        if let Some(reason) = error.audit_reason() {
            emit_refusal_audit(
                &state.pool,
                asset_id,
                principal.subject_id,
                Some(org_id),
                Some(project_id),
                reason,
            )
            .await?;
        }
        return Err(error);
    }
    apply_audience_policy_hook(&PlaybackAudiencePolicyContext {
        asset_id,
        actor_subject_id: principal.subject_id,
        org_id,
        project_id,
        scope: PlaybackScope::Review,
    })?;

    let issued_at = OffsetDateTime::now_utc();
    let grant = PlaybackGrant::new(
        PlaybackGrantId::new(),
        asset_id,
        PlaybackScope::Review,
        GrantPrincipal {
            principal_id: principal.subject_id,
            org_id,
            project_id,
        },
        issued_at,
        issued_at + Duration::hours(PLAYBACK_GRANT_TTL_HOURS),
    )
    .map_err(|error| ApiError::internal(format!("grant construction failed: {error}")))?;

    playback_repo::issue_grant(&state.pool, &grant)
        .await
        .map_err(ApiError::from_db)?;
    emit_success_audit(&state.pool, &grant).await?;

    Ok((
        StatusCode::CREATED,
        Json(PlaybackGrantIssueResponse {
            grant_id: grant.id.0,
        }),
    ))
}

pub async fn get_playback_manifest(
    State(state): State<Arc<AppState>>,
    Path(path): Path<PlaybackManifestPath>,
) -> Result<impl IntoResponse, ApiError> {
    let asset_id = AssetId(path.id);
    let grant_id = PlaybackGrantId(path.grant_id);
    let manifest_artifact = playback_repo::resolve_grant_target(&state.pool, grant_id)
        .await
        .map_err(ApiError::from_playback_denial)?;

    if manifest_artifact.asset_id != asset_id {
        return Err(ApiError::forbidden("asset not found"));
    }

    let manifest_bytes = get_hls_manifest(state.storage.as_ref(), &asset_id.0.to_string())
        .await
        .map_err(ApiError::from_storage)?;
    let manifest = String::from_utf8(manifest_bytes.0).map_err(|error| {
        ApiError::internal(format!("stored manifest is not valid UTF-8: {error}"))
    })?;
    let segment_refs = build_segment_refs(&state, asset_id, &manifest)?;
    let rewritten = rewrite_manifest_with_refs(&manifest, &segment_refs)
        .map_err(|error| ApiError::internal(format!("manifest rewrite failed: {error}")))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, manifest_artifact.content_type)],
        rewritten,
    ))
}

pub async fn get_playback_segment(
    State(state): State<Arc<AppState>>,
    Path(path): Path<PlaybackSegmentPath>,
    Query(query): Query<SegmentTokenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let asset_id = AssetId(path.id);
    validate_segment_reference(&state, asset_id, &path.filename, &query.token)?;

    let segment_bytes = get_hls_segment(
        state.storage.as_ref(),
        &asset_id.0.to_string(),
        &path.filename,
    )
    .await
    .map_err(ApiError::from_storage)?;
    debug!(
        asset_id = %asset_id.0,
        filename = %path.filename,
        "served short-lived playback segment reference"
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, segment_content_type(&path.filename))],
        segment_bytes.0,
    ))
}

fn build_segment_refs(
    state: &AppState,
    asset_id: AssetId,
    manifest: &str,
) -> Result<HashMap<String, String>, ApiError> {
    manifest_segment_names(manifest)
        .into_iter()
        .try_fold(HashMap::new(), |mut refs, filename| {
            let reference = mint_segment_reference(state, asset_id, &filename)?;
            refs.insert(filename, reference);
            Ok(refs)
        })
}

fn mint_segment_reference(
    state: &AppState,
    asset_id: AssetId,
    filename: &str,
) -> Result<String, ApiError> {
    let secret = playback_reference_secret(&state.config)?;
    let issuer = Hs256Issuer::new(
        &secret,
        StdDuration::from_secs(PLAYBACK_SEGMENT_TTL_SECONDS),
    )
    .map_err(|error| {
        ApiError::internal(format!("segment reference issuer init failed: {error}"))
    })?;
    let token = issuer
        .generate_jwt(asset_id.0, asset_id.0, &[segment_scope(filename)])
        .map_err(|error| {
            ApiError::internal(format!("segment reference signing failed: {error}"))
        })?;

    Ok(format!(
        "/assets/{}/playback/segments/{}?token={token}",
        asset_id.0, filename
    ))
}

fn validate_segment_reference(
    state: &AppState,
    asset_id: AssetId,
    filename: &str,
    token: &str,
) -> Result<(), ApiError> {
    let secret = playback_reference_secret(&state.config)?;
    let claims = parse_jwt(token, &secret)
        .map_err(|_| ApiError::forbidden("segment reference expired or invalid"))?;
    let claims_asset_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::forbidden("segment reference expired or invalid"))?;
    let claims_filename = claims
        .scope
        .split_whitespace()
        .find_map(|scope| scope.strip_prefix(SEGMENT_SCOPE_PREFIX))
        .ok_or_else(|| ApiError::forbidden("segment reference expired or invalid"))?;

    if claims_asset_id != asset_id.0 || claims_filename != filename {
        return Err(ApiError::forbidden("segment reference expired or invalid"));
    }

    Ok(())
}

fn segment_scope(filename: &str) -> String {
    format!("{SEGMENT_SCOPE_PREFIX}{filename}")
}

fn playback_reference_secret(config: &dubbridge_config::AppConfig) -> Result<String, ApiError> {
    match config
        .auth
        .as_ref()
        .and_then(|settings| settings.jwt_secret.clone())
    {
        Some(secret) => Ok(secret),
        None if !config.env.is_production_like() => {
            Ok(LOCAL_DEV_JWT_SECRET_PLACEHOLDER.to_string())
        }
        None => Err(ApiError::internal(
            "playback segment signing secret unavailable",
        )),
    }
}

fn segment_content_type(filename: &str) -> &'static str {
    if filename.ends_with(".ts") {
        "video/mp2t"
    } else if filename.ends_with(".m4s") {
        "video/iso.segment"
    } else {
        "application/octet-stream"
    }
}

#[derive(sqlx::FromRow)]
struct AuthorizedAssetScopeRow {
    org_id: Uuid,
    project_id: Uuid,
    role: String,
}

async fn resolve_authorized_asset_scope(
    pool: &PgPool,
    asset_id: AssetId,
    subject_id: Uuid,
) -> Result<(OrgId, ProjectId), ApiError> {
    let row = sqlx::query_as::<_, AuthorizedAssetScopeRow>(
        r#"
        SELECT p.org_id, p.id AS project_id, om.role
        FROM project_assets pa
        JOIN projects p ON p.id = pa.project_id
        JOIN org_members om ON om.org_id = p.org_id
        WHERE pa.asset_id = $1
          AND om.subject_id = $2
        ORDER BY p.created_at ASC, p.id ASC
        LIMIT 1
        "#,
    )
    .bind(asset_id.0)
    .bind(subject_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| ApiError::from_db(DbError::QueryFailed(error)))?;

    let Some(row) = row else {
        return Err(ApiError::forbidden("asset not found"));
    };

    let role = parse_org_role(&row.role).ok_or_else(|| {
        ApiError::internal(format!(
            "corrupt stored role in org_members.role: {}",
            row.role
        ))
    })?;
    if !role.satisfies(OrgRole::Reviewer) {
        return Err(ApiError::forbidden("asset not found"));
    }

    Ok((OrgId(row.org_id), ProjectId(row.project_id)))
}

async fn ensure_asset_ready_for_playback(pool: &PgPool, asset_id: AssetId) -> Result<(), ApiError> {
    let status = preparation_repo::get_preparation_status(pool, asset_id)
        .await
        .map_err(ApiError::from_db)?;
    let evidence = preparation_repo::get_preparation_readiness_evidence(pool, asset_id)
        .await
        .map_err(ApiError::from_db)?;

    match status {
        Some(status_row)
            if status_row.status == PreparationStatus::Ready
                && evidence.hls_manifest_count >= 1 =>
        {
            Ok(())
        }
        _ => Err(ApiError::conflict("asset not ready for playback")),
    }
}

async fn emit_success_audit(pool: &PgPool, grant: &PlaybackGrant) -> Result<(), ApiError> {
    let event = AuditEvent::new_playback_event(
        grant.asset_id,
        AuditEventKind::PlaybackGrantIssued,
        Some(
            json!({
                "grant_id": grant.id.0,
                "asset_id": grant.asset_id.0,
                "actor_subject_id": grant.principal.principal_id,
                "org_id": grant.principal.org_id.0,
                "project_id": grant.principal.project_id.0,
                "scope": grant.scope.to_string(),
            })
            .to_string(),
        ),
    );
    emit_governance_audit(pool, &event)
        .await
        .map_err(ApiError::from_audit_emit)
}

async fn emit_refusal_audit(
    pool: &PgPool,
    asset_id: AssetId,
    actor_subject_id: Uuid,
    org_id: Option<OrgId>,
    project_id: Option<ProjectId>,
    reason: &'static str,
) -> Result<(), ApiError> {
    let event =
        playback_grant_refused_event(asset_id, Some(actor_subject_id), org_id, project_id, reason);
    emit_governance_audit(pool, &event)
        .await
        .map_err(ApiError::from_audit_emit)
}
