use std::{sync::Arc, time::Duration};

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::{AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer};
use dubbridge_db::{
    error::DbError,
    p2p_audience_repo::{
        P2pAudienceAuthorizationRecord, P2pClaimResult, P2pDeviceRecord, P2pInvitationRecord,
        claim_invitation, create_invitation, get_active_authorization, list_viewer_invitations,
        register_or_get_active_device,
    },
    p2p_ready_repo::get_ready_descriptor_by_asset,
};
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    p2p_ready_descriptor::P2pReadyDescriptor,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;

const DEFAULT_INVITE_TTL_SECONDS: u64 = 24 * 60 * 60;
const MAX_INVITE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const MAX_TOKEN_LENGTH: usize = 256;
const MAX_DEVICE_PUBLIC_KEY_BYTES: usize = 2048;

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    Router::new()
        .route("/p2p/devices", post(register_device))
        .route(
            "/assets/{id}/p2p/invitations",
            post(create_asset_invitation),
        )
        .route("/p2p/invitations/claim", post(claim))
        .route("/p2p/invitations", get(list_invitations))
        .route("/p2p/authorizations/{id}", get(get_authorization))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ))
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceRequest {
    key_id: String,
    public_key_spki_base64: String,
}

#[derive(Debug, Serialize)]
struct DeviceResponse {
    id: Uuid,
    key_id: String,
    created_at_unix: i64,
}

#[derive(Debug, Deserialize)]
struct CreateInvitationRequest {
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
struct InvitationCreatedResponse {
    invitation: InvitationResponse,
    token: String,
}

#[derive(Debug, Deserialize)]
struct ClaimInvitationRequest {
    token: String,
    device_id: Uuid,
}

#[derive(Debug, Serialize)]
struct ClaimInvitationResponse {
    invitation: InvitationResponse,
    authorization: AuthorizationResponse,
    descriptor: P2pReadyDescriptor,
}

#[derive(Debug, Serialize)]
struct InvitationResponse {
    id: Uuid,
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    owner_subject_id: Uuid,
    status: &'static str,
    expires_at_unix: i64,
    claimed_at_unix: Option<i64>,
}

#[derive(Debug, Serialize)]
struct AuthorizationResponse {
    id: Uuid,
    invitation_id: Uuid,
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    viewer_subject_id: Uuid,
    device_id: Uuid,
    expires_at_unix: i64,
}

async fn register_device(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<RegisterDeviceRequest>,
) -> Response {
    let key_id = request.key_id.trim();
    if key_id.is_empty() || key_id.len() > 200 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let public_key = match BASE64_STANDARD.decode(request.public_key_spki_base64.as_bytes()) {
        Ok(bytes) if !bytes.is_empty() && bytes.len() <= MAX_DEVICE_PUBLIC_KEY_BYTES => bytes,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    match register_or_get_active_device(&state.pool, principal.subject_id, key_id, &public_key)
        .await
    {
        Ok(device) => {
            let event = AuditEvent::new_p3_event(
                None,
                AuditEventKind::P2pDeviceRegistered,
                device.id,
                None,
                None,
                Some(
                    json!({
                        "device_id": device.id,
                        "subject_id": principal.subject_id,
                        "key_id": device.key_id.clone(),
                    })
                    .to_string(),
                ),
            );
            if emit_p3_audit(&state, &event).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (StatusCode::OK, Json(device_response(&device))).into_response()
        }
        Err(error) => db_error_response(error),
    }
}

async fn create_asset_invitation(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(asset_id): Path<Uuid>,
    Json(request): Json<CreateInvitationRequest>,
) -> Response {
    let ttl_seconds = request.ttl_seconds.unwrap_or(DEFAULT_INVITE_TTL_SECONDS);
    if ttl_seconds == 0 || ttl_seconds > MAX_INVITE_TTL_SECONDS {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let raw_token = new_invitation_token();
    let token_hash = hash_token(&raw_token);
    let expires_at = OffsetDateTime::now_utc() + Duration::from_secs(ttl_seconds);
    match create_invitation(
        &state.pool,
        principal.subject_id,
        AssetId(asset_id),
        &token_hash,
        expires_at,
    )
    .await
    {
        Ok(invitation) => {
            let event = p3_package_event(
                &invitation,
                AuditEventKind::P2pInvitationCreated,
                invitation.id,
                Some(
                    json!({
                        "invitation_id": invitation.id,
                        "owner_subject_id": principal.subject_id,
                        "expires_at_unix": invitation.expires_at.unix_timestamp(),
                    })
                    .to_string(),
                ),
            );
            if emit_p3_audit(&state, &event).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (
                StatusCode::CREATED,
                Json(InvitationCreatedResponse {
                    invitation: invitation_response(&invitation, OffsetDateTime::now_utc()),
                    token: raw_token,
                }),
            )
                .into_response()
        }
        Err(error) => {
            if matches!(&error, DbError::NotFound | DbError::Conflict) {
                let denial = AuditEvent::new_p3_event(
                    Some(AssetId(asset_id)),
                    AuditEventKind::P2pAudienceAccessDenied,
                    asset_id,
                    None,
                    None,
                    Some(
                        json!({
                            "operation": "create_invitation",
                            "reason": "not_owner_or_not_ready",
                            "actor_subject_id": principal.subject_id,
                        })
                        .to_string(),
                    ),
                );
                if emit_p3_audit(&state, &denial).await.is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
            db_error_response(error)
        }
    }
}

async fn claim(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<ClaimInvitationRequest>,
) -> Response {
    if request.token.is_empty() || request.token.len() > MAX_TOKEN_LENGTH {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let now = OffsetDateTime::now_utc();
    let token_hash = hash_token(&request.token);
    let result = match claim_with_audit(
        &state,
        principal.subject_id,
        request.device_id,
        &token_hash,
        now,
    )
    .await
    {
        Ok(result) => result,
        Err(response) => return response,
    };
    let descriptor = match descriptor_for_claim(&state, &result).await {
        Ok(descriptor) => descriptor,
        Err(response) => return response,
    };
    if emit_claim_success_audits(&state, &result).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(ClaimInvitationResponse {
            invitation: invitation_response(&result.invitation, now),
            authorization: authorization_response(&result.authorization),
            descriptor,
        }),
    )
        .into_response()
}

async fn claim_with_audit(
    state: &AppState,
    viewer_subject_id: Uuid,
    device_id: Uuid,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<P2pClaimResult, Response> {
    match claim_invitation(&state.pool, token_hash, viewer_subject_id, device_id, now).await {
        Ok(result) => Ok(result),
        Err(error) => {
            if matches!(&error, DbError::NotFound | DbError::Conflict) {
                let denial = AuditEvent::new_p3_event(
                    None,
                    AuditEventKind::P2pAudienceAccessDenied,
                    device_id,
                    None,
                    None,
                    Some(
                        json!({
                            "operation": "claim_invitation",
                            "reason": "claim_denied",
                            "actor_subject_id": viewer_subject_id,
                            "device_id": device_id,
                        })
                        .to_string(),
                    ),
                );
                if emit_p3_audit(state, &denial).await.is_err() {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
            }
            Err(db_error_response(error))
        }
    }
}

async fn descriptor_for_claim(
    state: &AppState,
    result: &P2pClaimResult,
) -> Result<P2pReadyDescriptor, Response> {
    let descriptor = match get_ready_descriptor_by_asset(&state.pool, result.invitation.asset_id())
        .await
    {
        Ok(Some(descriptor)) => descriptor,
        Ok(None) => {
            return Err(claim_handoff_denial_response(state, result, "descriptor_missing").await);
        }
        Err(error) => return Err(db_error_response(error)),
    };

    if descriptor_matches_claim(&descriptor, &result.invitation, &result.authorization) {
        Ok(descriptor)
    } else {
        Err(claim_handoff_denial_response(state, result, "identity_mismatch").await)
    }
}

async fn claim_handoff_denial_response(
    state: &AppState,
    result: &P2pClaimResult,
    reason: &'static str,
) -> Response {
    let denial = p3_package_event(
        &result.invitation,
        AuditEventKind::P2pAudienceAccessDenied,
        result.authorization.id,
        Some(
            json!({
                "operation": "claim_descriptor_handoff",
                "reason": reason,
                "authorization_id": result.authorization.id,
            })
            .to_string(),
        ),
    );
    if emit_p3_audit(state, &denial).await.is_err() {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    } else {
        StatusCode::CONFLICT.into_response()
    }
}

async fn emit_claim_success_audits(
    state: &AppState,
    result: &P2pClaimResult,
) -> Result<(), dubbridge_audit::AuditEmitError> {
    let claimed = p3_package_event(
        &result.invitation,
        AuditEventKind::P2pInvitationClaimed,
        result.invitation.id,
        Some(
            json!({
                "invitation_id": result.invitation.id,
                "viewer_subject_id": result.authorization.viewer_subject_id,
                "device_id": result.authorization.device_id,
            })
            .to_string(),
        ),
    );
    let authorized = p3_package_event(
        &result.invitation,
        AuditEventKind::P2pAudienceAuthorizationIssued,
        result.authorization.id,
        Some(
            json!({
                "authorization_id": result.authorization.id,
                "invitation_id": result.invitation.id,
                "viewer_subject_id": result.authorization.viewer_subject_id,
                "device_id": result.authorization.device_id,
                "expires_at_unix": result.authorization.expires_at.unix_timestamp(),
            })
            .to_string(),
        ),
    );

    emit_p3_audit(state, &claimed).await?;
    emit_p3_audit(state, &authorized).await
}

async fn list_invitations(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Response {
    let now = OffsetDateTime::now_utc();
    match list_viewer_invitations(&state.pool, principal.subject_id).await {
        Ok(invitations) => Json(
            invitations
                .iter()
                .map(|invitation| invitation_response(invitation, now))
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(error) => db_error_response(error),
    }
}

async fn get_authorization(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(authorization_id): Path<Uuid>,
) -> Response {
    match get_active_authorization(
        &state.pool,
        authorization_id,
        principal.subject_id,
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(authorization) => Json(authorization_response(&authorization)).into_response(),
        Err(error) => db_error_response(error),
    }
}

fn p3_package_event(
    invitation: &P2pInvitationRecord,
    event_kind: AuditEventKind,
    correlation_id: Uuid,
    detail: Option<String>,
) -> AuditEvent {
    AuditEvent::new_p3_event(
        Some(invitation.asset_id()),
        event_kind,
        correlation_id,
        Some(invitation.publication_id),
        Some(invitation.lineage_id),
        detail,
    )
}

async fn emit_p3_audit(
    state: &AppState,
    event: &AuditEvent,
) -> Result<(), dubbridge_audit::AuditEmitError> {
    emit_governance_audit(&state.pool, event).await
}

fn descriptor_matches_claim(
    descriptor: &P2pReadyDescriptor,
    invitation: &P2pInvitationRecord,
    authorization: &P2pAudienceAuthorizationRecord,
) -> bool {
    descriptor.asset_id.0 == invitation.asset_id
        && descriptor.publication_id.0 == invitation.publication_id
        && descriptor.lineage_id.0 == invitation.lineage_id
        && authorization.asset_id == invitation.asset_id
        && authorization.publication_id == invitation.publication_id
        && authorization.lineage_id == invitation.lineage_id
        && authorization.invitation_id == invitation.id
}

fn new_invitation_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

fn device_response(device: &P2pDeviceRecord) -> DeviceResponse {
    DeviceResponse {
        id: device.id,
        key_id: device.key_id.clone(),
        created_at_unix: device.created_at.unix_timestamp(),
    }
}

fn invitation_response(
    invitation: &P2pInvitationRecord,
    now: OffsetDateTime,
) -> InvitationResponse {
    InvitationResponse {
        id: invitation.id,
        asset_id: invitation.asset_id,
        publication_id: invitation.publication_id,
        lineage_id: invitation.lineage_id,
        owner_subject_id: invitation.owner_subject_id,
        status: invitation_status(invitation, now),
        expires_at_unix: invitation.expires_at.unix_timestamp(),
        claimed_at_unix: invitation.claimed_at.map(OffsetDateTime::unix_timestamp),
    }
}

fn invitation_status(invitation: &P2pInvitationRecord, now: OffsetDateTime) -> &'static str {
    if invitation.revoked_at.is_some() {
        "revoked"
    } else if invitation.expires_at <= now {
        "expired"
    } else if invitation.claimed_by_subject_id.is_some() {
        "claimed"
    } else {
        "pending"
    }
}

fn authorization_response(authorization: &P2pAudienceAuthorizationRecord) -> AuthorizationResponse {
    AuthorizationResponse {
        id: authorization.id,
        invitation_id: authorization.invitation_id,
        asset_id: authorization.asset_id,
        publication_id: authorization.publication_id,
        lineage_id: authorization.lineage_id,
        viewer_subject_id: authorization.viewer_subject_id,
        device_id: authorization.device_id,
        expires_at_unix: authorization.expires_at.unix_timestamp(),
    }
}

fn db_error_response(error: DbError) -> Response {
    match error {
        DbError::NotFound => StatusCode::NOT_FOUND.into_response(),
        DbError::Conflict => StatusCode::CONFLICT.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::{descriptor_matches_claim, hash_token, invitation_status, new_invitation_token};
    use dubbridge_db::p2p_audience_repo::{P2pAudienceAuthorizationRecord, P2pInvitationRecord};
    use dubbridge_domain::{
        asset::AssetId,
        p2p_publication::{K1LineageId, P2pPublicationId},
        p2p_ready_descriptor::P2pReadyDescriptor,
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[test]
    fn invitation_token_is_high_entropy_hex_and_hash_is_sha256() {
        let left = new_invitation_token();
        let right = new_invitation_token();
        assert_eq!(left.len(), 64);
        assert!(left.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(left, right);
        assert_eq!(hash_token(&left).len(), 32);
    }

    #[test]
    fn claim_descriptor_must_match_exact_invitation_and_authorization_lineage() {
        let asset_id = Uuid::new_v4();
        let publication_id = Uuid::new_v4();
        let lineage_id = Uuid::new_v4();
        let invitation_id = Uuid::new_v4();
        let viewer_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        let invitation = P2pInvitationRecord {
            id: invitation_id,
            asset_id,
            publication_id,
            lineage_id,
            owner_subject_id: Uuid::new_v4(),
            expires_at: now,
            claimed_by_subject_id: Some(viewer_id),
            claimed_device_id: Some(device_id),
            claimed_at: Some(now),
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };
        let authorization = P2pAudienceAuthorizationRecord {
            id: Uuid::new_v4(),
            invitation_id,
            asset_id,
            publication_id,
            lineage_id,
            viewer_subject_id: viewer_id,
            device_id,
            expires_at: now,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };
        let descriptor = P2pReadyDescriptor {
            descriptor_version: "p2p-ready-descriptor-v1".to_owned(),
            asset_id: AssetId(asset_id),
            publication_id: P2pPublicationId(publication_id),
            lineage_id: K1LineageId(lineage_id),
            manifest_version: "p2p-manifest-v1".to_owned(),
            manifest_digest_sha256: "a".repeat(64),
            external_publication_id: "hyperdrive:test".to_owned(),
            ck_wrap_ref: "p2p-k1-wrap/test".to_owned(),
            kek_id: "kek-test".to_owned(),
            kek_version: 1,
            ready_at: "2026-09-22T00:00:00Z".to_owned(),
        };

        assert!(descriptor_matches_claim(
            &descriptor,
            &invitation,
            &authorization
        ));

        let mut wrong_lineage = descriptor.clone();
        wrong_lineage.lineage_id = K1LineageId(Uuid::new_v4());
        assert!(!descriptor_matches_claim(
            &wrong_lineage,
            &invitation,
            &authorization
        ));

        let mut wrong_publication = descriptor.clone();
        wrong_publication.publication_id = P2pPublicationId(Uuid::new_v4());
        assert!(!descriptor_matches_claim(
            &wrong_publication,
            &invitation,
            &authorization
        ));
    }

    #[test]
    fn invitation_status_expires_fail_closed() {
        let now = OffsetDateTime::now_utc();
        let invitation = P2pInvitationRecord {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            publication_id: Uuid::new_v4(),
            lineage_id: Uuid::new_v4(),
            owner_subject_id: Uuid::new_v4(),
            expires_at: now,
            claimed_by_subject_id: None,
            claimed_device_id: None,
            claimed_at: None,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(invitation_status(&invitation, now), "expired");
    }
}
