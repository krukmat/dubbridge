use axum::{Json, http::StatusCode, response::IntoResponse};
use dubbridge_db::error::DbError;
use dubbridge_storage::StorageError;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    pub fn from_audit_emit(error: dubbridge_audit::AuditEmitError) -> Self {
        Self::internal(error.to_string())
    }

    pub fn from_db(error: DbError) -> Self {
        match error {
            DbError::Conflict => Self::conflict("conflict"),
            DbError::NotFound => Self::forbidden("asset not found"),
            DbError::ConnectionFailed(source) | DbError::QueryFailed(source) => {
                Self::internal(format!("database operation failed: {source}"))
            }
            DbError::UnknownStoredValue { field, value } => {
                Self::internal(format!("corrupt stored value in {field}: {value}"))
            }
        }
    }

    pub fn from_playback_denial(error: dubbridge_domain::playback::PlaybackDenial) -> Self {
        match error {
            dubbridge_domain::playback::PlaybackDenial::GrantInvalid => {
                Self::forbidden("playback grant expired or revoked")
            }
            dubbridge_domain::playback::PlaybackDenial::NotReady => {
                Self::conflict("asset not ready for playback")
            }
            dubbridge_domain::playback::PlaybackDenial::MissingManifest => {
                Self::internal("prepared HLS manifest not found")
            }
            dubbridge_domain::playback::PlaybackDenial::Unauthenticated => {
                Self::forbidden("authentication required")
            }
            dubbridge_domain::playback::PlaybackDenial::Unauthorized => {
                Self::forbidden("asset not found")
            }
        }
    }

    pub fn from_storage(error: StorageError) -> Self {
        match error {
            StorageError::NotFound { .. } => Self::internal("prepared HLS manifest not found"),
            StorageError::Io { source, .. } => {
                Self::internal(format!("storage read failed: {source}"))
            }
            StorageError::Backend(message) => {
                Self::internal(format!("storage read failed: {message}"))
            }
        }
    }

    pub fn audit_reason(&self) -> Option<&'static str> {
        match (self.status, self.message.as_str()) {
            (StatusCode::FORBIDDEN, "asset not found") => Some("asset_not_found"),
            (StatusCode::CONFLICT, "asset not ready for playback") => Some("asset_not_ready"),
            _ => None,
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
