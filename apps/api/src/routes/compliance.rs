use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer, require_scope,
};
use dubbridge_db::{asset_repo, audit_repo, consent_repo, error::DbError, rights_repo};
use dubbridge_domain::{
    asset::AssetId,
    consent::{ConsentError, ConsentRow, ConsentStatus, derive_status, new_grant, new_revoke},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    consent_gate::{ConsentGateError, append_consent_audited},
    dto::compliance::{
        AuditEventResponse, AuditTimelineResponse, ConsentLedgerResponse, ConsentMutationRequest,
        ConsentMutationResponse, ConsentRowResponse, RightsLedgerResponse, RightsRecordResponse,
    },
    state::AppState,
};

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    let read_routes = Router::new()
        .route("/assets/{id}/audit", get(get_audit_timeline))
        .route("/assets/{id}/rights", get(get_rights_ledger))
        .route("/assets/{id}/consents", get(get_consent_ledger))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("assets:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let mutation_routes = Router::new()
        .route("/consents", post(record_consent))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("assets:ingest"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ));

    Router::new().merge(read_routes).merge(mutation_routes)
}

async fn get_audit_timeline(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<Json<AuditTimelineResponse>, ApiError> {
    let asset_id = AssetId(id);
    let events =
        audit_repo::list_audit_events_for_owned_asset(&state.pool, asset_id, principal.subject_id)
            .await
            .map_err(ApiError::from_scoped_db)?;

    Ok(Json(AuditTimelineResponse {
        asset_id: id,
        events: events.into_iter().map(AuditEventResponse::from).collect(),
    }))
}

async fn get_rights_ledger(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<Json<RightsLedgerResponse>, ApiError> {
    let asset_id = AssetId(id);
    let entries = rights_repo::list_rights_records_for_owned_asset(
        &state.pool,
        asset_id,
        principal.subject_id,
    )
    .await
    .map_err(ApiError::from_scoped_db)?;

    Ok(Json(RightsLedgerResponse {
        asset_id: id,
        entries: entries
            .into_iter()
            .map(RightsRecordResponse::from)
            .collect(),
    }))
}

async fn get_consent_ledger(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<Json<ConsentLedgerResponse>, ApiError> {
    let asset_id = AssetId(id);
    ensure_owned_asset(&state.pool, asset_id, principal.subject_id).await?;

    let rows = consent_repo::list_consents_for_asset(&state.pool, asset_id)
        .await
        .map_err(ApiError::from_db)?;
    let current_status = derive_status(&rows);

    Ok(Json(ConsentLedgerResponse {
        asset_id: id,
        current_status,
        rows: rows.into_iter().map(ConsentRowResponse::from).collect(),
    }))
}

async fn record_consent(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<ConsentMutationRequest>,
) -> Result<(StatusCode, Json<ConsentMutationResponse>), ApiError> {
    let asset_id = AssetId(request.asset_id);
    ensure_owned_asset(&state.pool, asset_id, principal.subject_id).await?;

    let row = build_consent_row(request, principal.subject_id)?;
    append_consent_audited(&state.pool, &row)
        .await
        .map_err(ApiError::from_consent_gate)?;

    Ok((
        StatusCode::CREATED,
        Json(ConsentMutationResponse {
            asset_id: row.asset_id.0,
            scope: row.scope,
            current_status: row.status,
            happened_at: row.happened_at,
        }),
    ))
}

async fn ensure_owned_asset(
    pool: &sqlx::PgPool,
    asset_id: AssetId,
    owner_id: Uuid,
) -> Result<(), ApiError> {
    let asset = asset_repo::find_asset_by_id(pool, asset_id)
        .await
        .map_err(ApiError::from_db)?;

    match asset {
        Some(asset) if asset.uploader_id == owner_id => Ok(()),
        _ => Err(ApiError::forbidden("asset not found")),
    }
}

fn build_consent_row(
    request: ConsentMutationRequest,
    actor_id: Uuid,
) -> Result<ConsentRow, ApiError> {
    match request.status {
        ConsentStatus::Grant => new_grant(
            AssetId(request.asset_id),
            request.scope,
            request.evidence_ref.as_deref().unwrap_or_default(),
            actor_id,
        )
        .map_err(ApiError::from_consent_validation),
        ConsentStatus::Revoke => Ok(new_revoke(
            AssetId(request.asset_id),
            request.scope,
            actor_id,
        )),
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn unprocessable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn from_db(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::forbidden("asset not found"),
            DbError::ConnectionFailed(source) | DbError::QueryFailed(source) => {
                Self::internal(format!("database operation failed: {source}"))
            }
            DbError::UnknownStoredValue { field, value } => {
                Self::internal(format!("corrupt stored value in {field}: {value}"))
            }
        }
    }

    fn from_scoped_db(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::forbidden("asset not found"),
            other => Self::from_db(other),
        }
    }

    fn from_consent_validation(error: ConsentError) -> Self {
        Self::unprocessable(error.to_string())
    }

    fn from_consent_gate(error: ConsentGateError) -> Self {
        match error {
            ConsentGateError::NoActiveConsent { .. } => {
                Self::unprocessable("no active consent for the requested asset and scope")
            }
            ConsentGateError::Db(message) => {
                Self::internal(format!("database operation failed: {message}"))
            }
            ConsentGateError::AuditFailed(message) => {
                Self::internal(format!("audit persistence failed: {message}"))
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "error": self.message,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
#[path = "compliance_tests.rs"]
mod tests;
