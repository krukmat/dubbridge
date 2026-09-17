use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use dubbridge_auth::{AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer};
use dubbridge_db::{
    error::DbError,
    p2p_dashboard_repo::{
        P2pOwnerContentRecord, P2pViewerInboxRecord, list_owner_p2p_content,
        list_viewer_p2p_inbox,
    },
    p2p_ready_repo::get_ready_descriptor_by_asset,
};
use dubbridge_domain::{
    p2p_publication::PublicationState,
    p2p_ready_descriptor::P2pReadyDescriptor,
};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    Router::new()
        .route("/p2p/content", get(list_owner_content))
        .route("/p2p/inbox", get(list_viewer_inbox))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum OwnerContentState {
    Processing,
    Ready,
    Failed,
}

#[derive(Debug, Serialize)]
struct OwnerContentResponse {
    asset_id: Uuid,
    title: String,
    publication_id: Uuid,
    lineage_id: Uuid,
    state: OwnerContentState,
    descriptor: Option<P2pReadyDescriptor>,
}

#[derive(Debug, Serialize)]
struct InboxItemResponse {
    invitation: InvitationResponse,
    authorization: AuthorizationResponse,
    authorization_active: bool,
    descriptor: Option<P2pReadyDescriptor>,
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

async fn list_owner_content(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Response {
    let records = match list_owner_p2p_content(&state.pool, principal.subject_id).await {
        Ok(records) => records,
        Err(error) => return db_error_response(error),
    };

    let mut response = Vec::with_capacity(records.len());
    for record in records {
        let descriptor = match get_ready_descriptor_by_asset(&state.pool, record.asset_id).await {
            Ok(descriptor) => matching_owner_descriptor(&record, descriptor),
            Err(error) => return db_error_response(error),
        };
        let state = project_owner_state(record.state, descriptor.as_ref());
        response.push(OwnerContentResponse {
            asset_id: record.asset_id.0,
            title: record.asset_title,
            publication_id: record.publication_id.0,
            lineage_id: record.lineage_id.0,
            state,
            descriptor,
        });
    }

    Json(response).into_response()
}

async fn list_viewer_inbox(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Response {
    let records = match list_viewer_p2p_inbox(&state.pool, principal.subject_id).await {
        Ok(records) => records,
        Err(error) => return db_error_response(error),
    };
    let now = OffsetDateTime::now_utc();
    let mut response = Vec::with_capacity(records.len());

    for record in records {
        let descriptor = match get_ready_descriptor_by_asset(&state.pool, record.invitation.asset_id()).await {
            Ok(descriptor) => matching_inbox_descriptor(&record, descriptor),
            Err(error) => return db_error_response(error),
        };
        response.push(inbox_response(record, descriptor, now));
    }

    Json(response).into_response()
}

fn matching_owner_descriptor(
    record: &P2pOwnerContentRecord,
    descriptor: Option<P2pReadyDescriptor>,
) -> Option<P2pReadyDescriptor> {
    descriptor.filter(|descriptor| {
        descriptor.asset_id == record.asset_id
            && descriptor.publication_id == record.publication_id
            && descriptor.lineage_id == record.lineage_id
    })
}

fn matching_inbox_descriptor(
    record: &P2pViewerInboxRecord,
    descriptor: Option<P2pReadyDescriptor>,
) -> Option<P2pReadyDescriptor> {
    descriptor.filter(|descriptor| {
        descriptor.asset_id.0 == record.invitation.asset_id
            && descriptor.publication_id.0 == record.invitation.publication_id
            && descriptor.lineage_id.0 == record.invitation.lineage_id
            && descriptor.publication_id.0 == record.authorization.publication_id
            && descriptor.lineage_id.0 == record.authorization.lineage_id
    })
}

fn project_owner_state(
    publication_state: PublicationState,
    descriptor: Option<&P2pReadyDescriptor>,
) -> OwnerContentState {
    if publication_state == PublicationState::Failed {
        OwnerContentState::Failed
    } else if publication_state == PublicationState::Ready && descriptor.is_some() {
        OwnerContentState::Ready
    } else {
        OwnerContentState::Processing
    }
}

fn inbox_response(
    record: P2pViewerInboxRecord,
    descriptor: Option<P2pReadyDescriptor>,
    now: OffsetDateTime,
) -> InboxItemResponse {
    let invitation = &record.invitation;
    let authorization = &record.authorization;
    InboxItemResponse {
        invitation: InvitationResponse {
            id: invitation.id,
            asset_id: invitation.asset_id,
            publication_id: invitation.publication_id,
            lineage_id: invitation.lineage_id,
            owner_subject_id: invitation.owner_subject_id,
            status: invitation_status(invitation, now),
            expires_at_unix: invitation.expires_at.unix_timestamp(),
            claimed_at_unix: invitation.claimed_at.map(OffsetDateTime::unix_timestamp),
        },
        authorization: AuthorizationResponse {
            id: authorization.id,
            invitation_id: authorization.invitation_id,
            asset_id: authorization.asset_id,
            publication_id: authorization.publication_id,
            lineage_id: authorization.lineage_id,
            viewer_subject_id: authorization.viewer_subject_id,
            device_id: authorization.device_id,
            expires_at_unix: authorization.expires_at.unix_timestamp(),
        },
        authorization_active: authorization.revoked_at.is_none() && authorization.expires_at > now,
        descriptor,
    }
}

fn invitation_status(
    invitation: &dubbridge_db::p2p_audience_repo::P2pInvitationRecord,
    now: OffsetDateTime,
) -> &'static str {
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

fn db_error_response(error: DbError) -> Response {
    match error {
        DbError::NotFound => StatusCode::NOT_FOUND.into_response(),
        DbError::Conflict => StatusCode::CONFLICT.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::{
        asset::AssetId,
        p2p_publication::{K1LineageId, P2pPublicationId},
        p2p_ready_descriptor::{P2pReadyDescriptorInput, P2pReadyDescriptor},
    };

    fn descriptor(
        asset_id: AssetId,
        publication_id: P2pPublicationId,
        lineage_id: K1LineageId,
    ) -> P2pReadyDescriptor {
        P2pReadyDescriptor::try_from(P2pReadyDescriptorInput {
            asset_id,
            publication_id,
            lineage_id,
            manifest_digest_sha256: "a".repeat(64),
            external_publication_id: "hyperdrive:test".to_owned(),
            kek_id: "kek-1".to_owned(),
            kek_version: 1,
            ready_at: "2026-09-17T10:00:00.000000Z".to_owned(),
        })
        .expect("descriptor")
    }

    #[test]
    fn raw_ready_without_authoritative_descriptor_stays_processing() {
        assert_eq!(
            project_owner_state(PublicationState::Ready, None),
            OwnerContentState::Processing
        );
    }

    #[test]
    fn authoritative_ready_descriptor_enables_ready_projection() {
        let descriptor = descriptor(
            AssetId(Uuid::new_v4()),
            P2pPublicationId::new(),
            K1LineageId::new(),
        );
        assert_eq!(
            project_owner_state(PublicationState::Ready, Some(&descriptor)),
            OwnerContentState::Ready
        );
    }

    #[test]
    fn failed_publication_never_projects_ready() {
        let descriptor = descriptor(
            AssetId(Uuid::new_v4()),
            P2pPublicationId::new(),
            K1LineageId::new(),
        );
        assert_eq!(
            project_owner_state(PublicationState::Failed, Some(&descriptor)),
            OwnerContentState::Failed
        );
    }
}
