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
use dubbridge_audit::emit_governance_audit;
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

    while let Some(field) = multipart.next_field().await.map_err(|error| {
        if is_body_size_limit_error(&error) {
            ApiError::payload_too_large("request body exceeds the maximum allowed size")
        } else {
            ApiError::bad_request(format!("invalid multipart payload: {error}"))
        }
    })? {
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
                            // multer wraps DefaultBodyLimit's LengthLimitError as a
                            // body-read failure. Surface it as 413 so clients know the
                            // body was too large, not malformed.
                            if is_body_size_limit_error(&error) {
                                ApiError::payload_too_large(
                                    "request body exceeds the maximum allowed size",
                                )
                            } else {
                                ApiError::bad_request(format!("invalid file field: {error}"))
                            }
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

// H1-T1: delegate entirely to the atomic core — it acquires the row lock,
// re-checks expiry inside the transaction, and commits all writes atomically.
// H1-T3: uses match instead of .map_err() so rejection audit can be awaited.
async fn finalize_ingestion(
    Path(token): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<(StatusCode, Json<AssetSummaryResponse>), ApiError> {
    match finalize_ingestion_core(
        &state.pool,
        token,
        principal.subject_id,
        ArtifactKind::OriginalMedia,
    )
    .await
    {
        Ok(asset) => Ok((StatusCode::CREATED, Json(asset.into()))),
        Err(error) => Err(map_service_error(&state, token, error).await),
    }
}

// H1-T3: async so rejection audit can be awaited before returning.
// Fail-closed: if audit persistence fails, returns 500 rather than swallowing the error.
async fn map_service_error(
    state: &AppState,
    ingest_token: Uuid,
    error: IngestionServiceError,
) -> ApiError {
    match error {
        IngestionServiceError::AlreadyFinalized => {
            ApiError::conflict("ingestion token has already been finalized")
        }
        IngestionServiceError::SessionExpired => ApiError::gone("ingestion session has expired"),
        IngestionServiceError::SessionNotFound => {
            ApiError::not_found("ingestion session not found")
        }
        IngestionServiceError::Validation(validation_error) => {
            let event_kind = match &validation_error {
                IngestionError::MissingUploaderContext => {
                    AuditEventKind::IngestionRejectedMissingUploaderContext
                }
                _ => AuditEventKind::IngestionRejectedMissingRights,
            };
            let audit_event = AuditEvent::new(
                None,
                event_kind,
                ingest_token,
                Some(validation_error.to_string()),
            );
            // H1-T3: awaited — durable row guaranteed before response (ADR-018).
            if let Err(audit_err) = emit_governance_audit(&state.pool, &audit_event).await {
                tracing::error!(
                    ingest_token = %ingest_token,
                    error = %audit_err,
                    "audit persistence failed for validation rejection"
                );
                return ApiError::internal("audit persistence failed");
            }
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

/// Traverses the error source chain to detect a body-size limit error surfaced
/// by DefaultBodyLimit (LengthLimitError) or multer's own stream-size gate.
fn is_body_size_limit_error(error: &dyn std::error::Error) -> bool {
    let mut maybe: Option<&dyn std::error::Error> = Some(error);
    while let Some(e) = maybe {
        let msg = e.to_string().to_lowercase();
        if msg.contains("length limit") || msg.contains("stream size exceeded") {
            return true;
        }
        maybe = e.source();
    }
    false
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

    fn payload_too_large(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
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
            // H1-T2: unknown persisted governance value is a data integrity error, not a client error.
            dubbridge_db::error::DbError::UnknownStoredValue { field, value } => {
                Self::internal(format!("corrupt stored value in {field}: {value}"))
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
