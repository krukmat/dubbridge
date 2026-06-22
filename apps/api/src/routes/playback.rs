use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::SharedTokenVerifier;
use dubbridge_domain::asset::AssetId;
use serde_json::json;
use uuid::Uuid;

use crate::{
    playback_audit::playback_grant_refused_event,
    playback_service::{get_playback_manifest, get_playback_segment, issue_playback_grant},
    state::AppState,
};

const PLAYBACK_REQUIRED_SCOPE: &str = "workspaces:write";

pub fn router(state: Arc<AppState>, _verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    Router::new()
        .merge(
            Router::new()
                .route("/assets/{id}/playback-grants", post(issue_playback_grant))
                .route_layer(middleware::from_fn_with_state(
                    state,
                    authorize_playback_grant_request,
                )),
        )
        .route(
            "/assets/{id}/playback/{grant_id}/manifest",
            get(get_playback_manifest),
        )
        .route(
            "/assets/{id}/playback/segments/{filename}",
            get(get_playback_segment),
        )
}

async fn authorize_playback_grant_request(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let asset_id = playback_asset_id_from_request(&request);
    let token = match bearer_token(request.headers()) {
        Some(token) => token,
        None => {
            return auth_boundary_denial_response(
                &state,
                asset_id,
                None,
                StatusCode::UNAUTHORIZED,
                "auth_unauthorized",
            )
            .await;
        }
    };

    let principal = match state.verifier.verify_access_token(token) {
        Ok(principal) => principal,
        Err(_) => {
            return auth_boundary_denial_response(
                &state,
                asset_id,
                None,
                StatusCode::UNAUTHORIZED,
                "auth_unauthorized",
            )
            .await;
        }
    };

    if !principal.has_scope(PLAYBACK_REQUIRED_SCOPE) {
        return auth_boundary_denial_response(
            &state,
            asset_id,
            Some(principal.subject_id),
            StatusCode::FORBIDDEN,
            "missing_scope",
        )
        .await;
    }

    request.extensions_mut().insert(principal);
    next.run(request).await
}

async fn auth_boundary_denial_response(
    state: &AppState,
    asset_id: Option<AssetId>,
    actor_subject_id: Option<Uuid>,
    status: StatusCode,
    reason: &'static str,
) -> Response {
    if let Some(asset_id) = asset_id {
        let event = playback_grant_refused_event(asset_id, actor_subject_id, None, None, reason);

        if let Err(error) = emit_governance_audit(&state.pool, &event).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": error.to_string(),
                })),
            )
                .into_response();
        }
    }

    status.into_response()
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    let header_value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let mut parts = header_value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;

    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(token)
}

fn playback_asset_id_from_request(request: &Request) -> Option<AssetId> {
    let mut segments = request.uri().path().split('/');
    let _ = segments.next()?;
    let collection = segments.next()?;
    let asset_id = segments.next()?;
    let action = segments.next()?;

    if collection != "assets" || action != "playback-grants" {
        return None;
    }

    Uuid::parse_str(asset_id).ok().map(AssetId)
}
