use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer, require_scope,
};
use dubbridge_db::{error::DbError, notification_repo, notification_repo::PushTokenRow};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    dto::notifications::{
        MarkNotificationsReadRequest, NotificationListResponse, NotificationResponse,
        RegisterPushTokenRequest,
    },
    state::AppState,
};

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    let read_routes = Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/mark-read", post(mark_notifications_read))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let write_routes = Router::new()
        .route("/notifications/push-tokens", post(register_push_token))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:write"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ));

    Router::new().merge(read_routes).merge(write_routes)
}

async fn list_notifications(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<Json<NotificationListResponse>, ApiError> {
    let rows =
        notification_repo::list_notifications_for_recipient(&state.pool, principal.subject_id)
            .await
            .map_err(ApiError::from_db)?;

    Ok(Json(NotificationListResponse {
        notifications: rows.into_iter().map(NotificationResponse::from).collect(),
    }))
}

async fn mark_notifications_read(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<MarkNotificationsReadRequest>,
) -> Result<StatusCode, ApiError> {
    notification_repo::mark_notifications_read(&state.pool, principal.subject_id, &request.ids)
        .await
        .map_err(ApiError::from_db)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn register_push_token(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<RegisterPushTokenRequest>,
) -> Result<StatusCode, ApiError> {
    if request.token.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "token must not be empty".into(),
        });
    }
    if !matches!(request.platform.as_str(), "ios" | "android") {
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: format!("unsupported platform: {}", request.platform),
        });
    }

    let now = OffsetDateTime::now_utc();
    let row = PushTokenRow {
        id: Uuid::new_v4(),
        subject_id: principal.subject_id,
        provider: "expo".to_owned(),
        device_token: request.token,
        platform: request.platform,
        created_at: now,
        updated_at: now,
    };

    notification_repo::insert_push_token(&state.pool, &row)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                return ApiError {
                    status: StatusCode::CONFLICT,
                    message: "token already registered".into(),
                };
            }
            ApiError::from_db(e)
        })?;

    Ok(StatusCode::CREATED)
}

fn is_unique_violation(e: &DbError) -> bool {
    let sqlx_err = match e {
        DbError::QueryFailed(inner) => inner,
        _ => return false,
    };
    sqlx_err
        .as_database_error()
        .and_then(|d| d.code())
        .as_deref()
        == Some("23505")
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn from_db(error: DbError) -> Self {
        match error {
            DbError::Conflict => Self {
                status: StatusCode::CONFLICT,
                message: "conflict".into(),
            },
            DbError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                message: "not found".into(),
            },
            DbError::ConnectionFailed(source) | DbError::QueryFailed(source) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("database error: {source}"),
            },
            DbError::UnknownStoredValue { field, value } => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("corrupt stored value in {field}: {value}"),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}
