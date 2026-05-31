use std::{borrow::Cow, sync::Arc};

use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};

// T1-T6: maximum accepted upload body size. Sized at 500 MB to cover professional
// audio/video assets while bounding server memory exposure per concurrent request.
pub const MAX_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer, require_scope,
};
use dubbridge_db::pending_ingestion_repo::PENDING_INGESTION_TTL_HOURS; // T1-T2
use dubbridge_domain::{
    artifact::ArtifactKind,
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    ingestion::IngestionError,
};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    dto::ingestion::{
        AssetSummaryResponse, IngestSessionResponse, RightsSubmissionResponse, SubmitRightsRequest,
    },
    ingestion_service::{IngestionServiceError, finalize_ingestion_core},
    state::AppState,
};

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    let mutation_routes = Router::new()
        .route("/ingest", post(create_ingestion))
        .route("/ingest/{token}/rights", post(submit_rights))
        .route("/ingest/{token}/finalize", post(finalize_ingestion))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)) // T1-T6: enforce before auth layers
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("assets:ingest"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let read_routes = Router::new()
        .route("/assets/{id}", get(get_asset))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("assets:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ));

    Router::new().merge(mutation_routes).merge(read_routes)
}

async fn create_ingestion(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<IngestSessionResponse>), ApiError> {
    let ingest_token = Uuid::new_v4();
    let mut title: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::bad_request(format!("invalid multipart payload: {error}")))?
    {
        match field.name() {
            Some("title") => {
                title = Some(field.text().await.map_err(|error| {
                    ApiError::bad_request(format!("invalid title field: {error}"))
                })?);
            }
            Some("file") => {
                filename = field.file_name().map(ToOwned::to_owned);
                content_type = field.content_type().map(ToOwned::to_owned);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|error| {
                            ApiError::bad_request(format!("invalid file field: {error}"))
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::bad_request("multipart field 'file' is required"))?;
    let storage_key = build_storage_key(ingest_token, filename.as_deref());
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let resolved_title = title
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
        .or_else(|| filename.clone())
        .unwrap_or_else(|| format!("upload-{ingest_token}"));
    let checksum = sha256_hex(&file_bytes);
    let size_bytes = i64::try_from(file_bytes.len())
        .map_err(|_| ApiError::internal("uploaded file is too large"))?;

    state
        .storage
        .put(&storage_key, file_bytes)
        .await
        .map_err(|error| ApiError::internal(format!("failed to store upload: {error}")))?;

    let now = OffsetDateTime::now_utc();
    let pending_ingestion = dubbridge_db::pending_ingestion_repo::PendingIngestionRecord {
        ingest_token,
        title: resolved_title.clone(),
        storage_key: storage_key.clone(),
        content_type: content_type.clone(),
        file_size_bytes: size_bytes,
        checksum: checksum.clone(),
        rights_basis: None,
        created_at: now,
        updated_at: now,
        expires_at: now + Duration::hours(PENDING_INGESTION_TTL_HOURS), // T1-T2
    };

    if let Err(error) = dubbridge_db::pending_ingestion_repo::insert_pending_ingestion(
        &state.pool,
        &pending_ingestion,
    )
    .await
    {
        let delete_result = state.storage.delete(&storage_key).await;
        if let Err(delete_error) = delete_result {
            tracing::warn!(
                ingest_token = %ingest_token,
                storage_key = %storage_key,
                error = %delete_error,
                "failed to clean up stored upload after pending-ingestion persistence error"
            );
        }

        return Err(ApiError::from_db(error));
    }

    Ok((
        StatusCode::CREATED,
        Json(IngestSessionResponse {
            ingest_token,
            title: pending_ingestion.title,
            storage_key: pending_ingestion.storage_key,
            content_type: pending_ingestion.content_type,
            size_bytes: pending_ingestion.file_size_bytes,
        }),
    ))
}

async fn submit_rights(
    Path(token): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubmitRightsRequest>,
) -> Result<Json<RightsSubmissionResponse>, ApiError> {
    // T1-T2: load record first so we can enforce expiration before mutating.
    let pending = dubbridge_db::pending_ingestion_repo::find_pending_ingestion(&state.pool, token)
        .await
        .map_err(ApiError::from_db)?
        .ok_or_else(|| ApiError::not_found("ingestion session not found"))?;

    if pending.expires_at < OffsetDateTime::now_utc() {
        tracing::info!(ingest_token = %token, "rights submission rejected: session expired");
        return Err(ApiError::gone("ingestion session has expired"));
    }

    let rights_basis = request.into();
    let updated =
        dubbridge_db::pending_ingestion_repo::attach_rights(&state.pool, token, &rights_basis)
            .await
            .map_err(ApiError::from_db)?;
    if !updated {
        return Err(ApiError::not_found("ingestion session not found"));
    }

    Ok(Json(RightsSubmissionResponse {
        ingest_token: token,
        status: "rights_recorded",
    }))
}

// S3-T0: thin wrapper — load pending record, enforce expiry, then delegate to core.
async fn finalize_ingestion(
    Path(token): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<(StatusCode, Json<AssetSummaryResponse>), ApiError> {
    let pending = dubbridge_db::pending_ingestion_repo::find_pending_ingestion(&state.pool, token)
        .await
        .map_err(ApiError::from_db)?
        .ok_or_else(|| ApiError::not_found("ingestion session not found"))?;

    // T1-T2: reject finalization of expired sessions (fail-closed per ADR-008).
    if pending.expires_at < OffsetDateTime::now_utc() {
        tracing::info!(ingest_token = %token, "finalization rejected: session expired");
        return Err(ApiError::gone("ingestion session has expired"));
    }

    let asset = finalize_ingestion_core(
        &state.pool,
        token,
        principal.subject_id,
        ArtifactKind::OriginalMedia,
        pending,
    )
    .await
    .map_err(|error| map_service_error(&state, token, error))?;

    Ok((StatusCode::CREATED, Json(asset.into())))
}

fn map_service_error(state: &AppState, ingest_token: Uuid, error: IngestionServiceError) -> ApiError {
    match error {
        IngestionServiceError::AlreadyFinalized => {
            ApiError::conflict("ingestion token has already been finalized")
        }
        IngestionServiceError::SessionExpired => ApiError::gone("ingestion session has expired"),
        IngestionServiceError::SessionNotFound => {
            ApiError::not_found("ingestion session not found")
        }
        IngestionServiceError::Validation(validation_error) => {
            // Fire-and-forget audit — log if the audit write itself fails.
            let pool = state.pool.clone();
            let error_str = validation_error.to_string();
            let event_kind = match &validation_error {
                IngestionError::MissingUploaderContext => {
                    AuditEventKind::IngestionRejectedMissingUploaderContext
                }
                _ => AuditEventKind::IngestionRejectedMissingRights,
            };
            tokio::spawn(async move {
                let audit_event = AuditEvent::new(None, event_kind, ingest_token, Some(error_str));
                if let Err(db_error) =
                    dubbridge_db::audit_repo::insert_audit_event(&pool, &audit_event).await
                {
                    tracing::warn!(
                        ingest_token = %ingest_token,
                        error = %db_error,
                        "failed to persist validation-rejection audit event"
                    );
                }
            });
            ApiError::unprocessable(validation_error.to_string())
        }
        IngestionServiceError::Db(db_error) => ApiError::from_db(db_error),
        IngestionServiceError::Internal(message) => ApiError::internal(message),
    }
}

async fn get_asset(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<AssetSummaryResponse>, ApiError> {
    let asset = dubbridge_db::asset_repo::find_asset_by_id(&state.pool, AssetId(id))
        .await
        .map_err(ApiError::from_db)?
        .ok_or_else(|| ApiError::not_found("asset not found"))?;

    Ok(Json(asset.into()))
}

fn build_storage_key(ingest_token: Uuid, filename: Option<&str>) -> String {
    let filename = sanitize_filename(filename.unwrap_or("upload.bin"));
    format!("ingests/{ingest_token}/{filename}")
}

fn sanitize_filename(filename: &str) -> Cow<'_, str> {
    if filename.contains('/') || filename.contains('\\') {
        Cow::Owned(filename.replace(['/', '\\'], "_"))
    } else {
        Cow::Borrowed(filename)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
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

    // T1-T2: session existed but its lifecycle window is closed.
    fn gone(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::GONE,
            message: message.into(),
        }
    }

    fn from_db(error: dubbridge_db::error::DbError) -> Self {
        match error {
            dubbridge_db::error::DbError::NotFound => Self::not_found("record not found"),
            dubbridge_db::error::DbError::ConnectionFailed(source)
            | dubbridge_db::error::DbError::QueryFailed(source) => {
                Self::internal(format!("database operation failed: {source}"))
            }
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message,
            })),
        )
            .into_response()
    }
}
