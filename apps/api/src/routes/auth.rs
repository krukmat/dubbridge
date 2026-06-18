use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::AuthServiceError;
use dubbridge_domain::audit::{AuditEvent, AuditEventKind};
use serde_json::json;

use crate::{
    dto::auth::{AuthSuccessResponse, LoginRequest, RegisterRequest},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
}

async fn login(
    State(state): State<Arc<AppState>>,
    request: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<AuthSuccessResponse>, ApiError> {
    let Json(request) = request
        .map_err(|rejection| ApiError::from_json_rejection("invalid login request", rejection))?;
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("auth service unavailable"))?;

    let result = match auth_service.login(&request.email, &request.password).await {
        Ok(result) => result,
        Err(AuthServiceError::InvalidCredentials) => {
            emit_auth_audit(
                &state,
                AuditEventKind::AuthLoginFailed,
                Some(json!({ "outcome": "invalid_credentials" }).to_string()),
            )
            .await?;
            return Err(ApiError::unauthorized("invalid credentials"));
        }
        Err(error) => return Err(ApiError::from_login_error(error)),
    };

    emit_auth_audit(
        &state,
        AuditEventKind::AuthLoginSucceeded,
        Some(
            json!({
                "user_id": result.user_id,
                "workspace_id": result.workspace_id,
            })
            .to_string(),
        ),
    )
    .await?;

    Ok(Json(AuthSuccessResponse::from(result)))
}

async fn register(
    State(state): State<Arc<AppState>>,
    request: Result<Json<RegisterRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<AuthSuccessResponse>), ApiError> {
    let Json(request) = request.map_err(|rejection| {
        ApiError::from_json_rejection("invalid register request", rejection)
    })?;
    let auth_service = state
        .auth_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("auth service unavailable"))?;

    let result = auth_service
        .register(&request.email, &request.password, &request.workspace_name)
        .await
        .map_err(ApiError::from_register_error)?;

    emit_auth_audit(
        &state,
        AuditEventKind::AuthRegistered,
        Some(
            json!({
                "user_id": result.user_id,
                "workspace_id": result.workspace_id,
            })
            .to_string(),
        ),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(AuthSuccessResponse::from(result))))
}

async fn emit_auth_audit(
    state: &AppState,
    event_kind: AuditEventKind,
    detail: Option<String>,
) -> Result<(), ApiError> {
    let audit_event = AuditEvent::new_auth_event(event_kind, detail);
    emit_governance_audit(&state.pool, &audit_event)
        .await
        .map_err(ApiError::from_audit_emit)
}

#[derive(Debug)]
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

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn from_json_rejection(message: &'static str, _: JsonRejection) -> Self {
        Self::bad_request(message)
    }

    fn from_login_error(error: AuthServiceError) -> Self {
        match error {
            AuthServiceError::Validation(source) => Self::bad_request(source.to_string()),
            AuthServiceError::Db(_) | AuthServiceError::TokenIssue(_) => {
                Self::internal("login failed")
            }
            AuthServiceError::InvalidCredentials | AuthServiceError::Conflict => {
                Self::internal("login failed")
            }
        }
    }

    fn from_register_error(error: AuthServiceError) -> Self {
        match error {
            AuthServiceError::Validation(source) => Self::bad_request(source.to_string()),
            AuthServiceError::Conflict => Self::conflict("account already exists"),
            AuthServiceError::Db(_) | AuthServiceError::TokenIssue(_) => {
                Self::internal("registration failed")
            }
            AuthServiceError::InvalidCredentials => Self::internal("registration failed"),
        }
    }

    fn from_audit_emit(error: dubbridge_audit::AuditEmitError) -> Self {
        Self::internal(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, sync::Arc, time::Duration};

    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
    };
    use dubbridge_auth::{Hs256Issuer, PgAccountStore, TokenVerificationError, TokenVerifier};
    use dubbridge_config::AppConfig;
    use dubbridge_storage::LocalFsAdapter;
    use sqlx::PgPool;
    use tempfile::TempDir;
    use tower::ServiceExt;

    use super::*;
    use crate::state::{ApiAuthService, SharedAuthService};

    #[derive(Clone, Default)]
    struct StubTokenVerifier;

    impl TokenVerifier for StubTokenVerifier {
        fn verify_access_token(
            &self,
            _token: &str,
        ) -> Result<dubbridge_auth::AuthenticatedPrincipal, TokenVerificationError> {
            Err(TokenVerificationError::MalformedToken)
        }
    }

    struct TestContext {
        pool: PgPool,
        app: axum::Router,
        _storage_dir: Arc<TempDir>,
        _storage_path: PathBuf,
    }

    impl TestContext {
        async fn new() -> Option<Self> {
            let database_url = match env::var("DUBBRIDGE_DATABASE_URL") {
                Ok(url) => url,
                Err(_) => return None,
            };

            let pool = PgPool::connect(&database_url)
                .await
                .expect("connect database");
            migrate_and_reset(&pool).await;

            let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
            let storage_path = storage_dir.path().to_path_buf();
            let auth_service = build_auth_service(pool.clone(), Duration::from_secs(24 * 60 * 60));
            let verifier = Arc::new(StubTokenVerifier) as dubbridge_auth::SharedTokenVerifier;
            let state = Arc::new(AppState::with_auth_service(
                pool.clone(),
                Box::new(LocalFsAdapter::new(&storage_path)),
                verifier,
                AppConfig::from_env(),
                auth_service,
            ));

            Some(Self {
                pool,
                app: router().with_state(state),
                _storage_dir: storage_dir,
                _storage_path: storage_path,
            })
        }

        async fn with_closed_audit_pool() -> Option<Self> {
            let database_url = match env::var("DUBBRIDGE_DATABASE_URL") {
                Ok(url) => url,
                Err(_) => return None,
            };

            let auth_pool = PgPool::connect(&database_url)
                .await
                .expect("connect auth database");
            migrate_and_reset(&auth_pool).await;

            let audit_pool = PgPool::connect(&database_url)
                .await
                .expect("connect audit database");
            audit_pool.close().await;

            let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
            let storage_path = storage_dir.path().to_path_buf();
            let auth_service =
                build_auth_service(auth_pool.clone(), Duration::from_secs(24 * 60 * 60));
            let verifier = Arc::new(StubTokenVerifier) as dubbridge_auth::SharedTokenVerifier;
            let state = Arc::new(AppState::with_auth_service(
                audit_pool,
                Box::new(LocalFsAdapter::new(&storage_path)),
                verifier,
                AppConfig::from_env(),
                auth_service,
            ));

            Some(Self {
                pool: auth_pool,
                app: router().with_state(state),
                _storage_dir: storage_dir,
                _storage_path: storage_path,
            })
        }
    }

    fn build_auth_service(pool: PgPool, expiry: Duration) -> SharedAuthService {
        let issuer = Hs256Issuer::new("register-route-test-secret", expiry).expect("issuer");
        Arc::new(ApiAuthService::new(PgAccountStore::new(pool), issuer))
    }

    #[tokio::test]
    async fn login_handler_returns_ok_and_emits_success_audit() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        seed_account(
            &ctx.pool,
            "owner@example.com",
            "correct horse battery staple",
            "DubBridge",
        )
        .await;

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/login",
            r#"{"email":"owner@example.com","password":"correct horse battery staple"}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;
        let detail = latest_audit_detail(&ctx.pool, "auth_login_succeeded")
            .await
            .expect("detail");

        assert_eq!(status, StatusCode::OK);
        assert!(body["token"].as_str().expect("token").len() > 20);
        assert!(body["userId"].as_str().expect("user id").len() > 10);
        assert!(body["workspaceId"].as_str().expect("workspace id").len() > 10);
        assert_eq!(
            count_audit_events(&ctx.pool, "auth_login_succeeded").await,
            1
        );
        assert!(!detail.contains(body["token"].as_str().expect("token")));
    }

    #[tokio::test]
    async fn login_handler_maps_wrong_password_and_unknown_email_to_same_unauthorized() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        seed_account(
            &ctx.pool,
            "owner@example.com",
            "correct horse battery staple",
            "DubBridge",
        )
        .await;

        let wrong_password = send_json(
            &ctx.app,
            Method::POST,
            "/auth/login",
            r#"{"email":"owner@example.com","password":"definitely the wrong password"}"#,
        )
        .await;
        let wrong_password_status = wrong_password.status();
        let wrong_password_body = json_body(wrong_password).await;

        let unknown_email = send_json(
            &ctx.app,
            Method::POST,
            "/auth/login",
            r#"{"email":"missing@example.com","password":"definitely the wrong password"}"#,
        )
        .await;
        let unknown_email_status = unknown_email.status();
        let unknown_email_body = json_body(unknown_email).await;
        let detail = latest_audit_detail(&ctx.pool, "auth_login_failed")
            .await
            .expect("detail");

        assert_eq!(wrong_password_status, StatusCode::UNAUTHORIZED);
        assert_eq!(unknown_email_status, StatusCode::UNAUTHORIZED);
        assert_eq!(wrong_password_body, unknown_email_body);
        assert_eq!(wrong_password_body["error"], "invalid credentials");
        assert_eq!(count_audit_events(&ctx.pool, "auth_login_failed").await, 2);
        assert_eq!(detail, r#"{"outcome":"invalid_credentials"}"#);
    }

    #[tokio::test]
    async fn login_handler_maps_validation_errors_to_bad_request() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/login",
            r#"{"email":"owner@example.com","password":" "}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "password is required");
        assert_eq!(
            count_audit_events(&ctx.pool, "auth_login_succeeded").await,
            0
        );
        assert_eq!(count_audit_events(&ctx.pool, "auth_login_failed").await, 0);
    }

    #[tokio::test]
    async fn login_handler_fails_closed_when_audit_persistence_fails() {
        let Some(ctx) = TestContext::with_closed_audit_pool().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        seed_account(
            &ctx.pool,
            "owner@example.com",
            "correct horse battery staple",
            "DubBridge",
        )
        .await;

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/login",
            r#"{"email":"owner@example.com","password":"correct horse battery staple"}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            body["error"]
                .as_str()
                .expect("error")
                .contains("audit persistence failed")
        );
    }

    #[tokio::test]
    async fn register_handler_returns_created_and_emits_audit() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/register",
            r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;
        let detail = latest_audit_detail(&ctx.pool, "auth_registered")
            .await
            .expect("detail");

        assert_eq!(status, StatusCode::CREATED);
        assert!(body["token"].as_str().expect("token").len() > 20);
        assert!(body["userId"].as_str().expect("user id").len() > 10);
        assert!(body["workspaceId"].as_str().expect("workspace id").len() > 10);
        assert_eq!(count_audit_events(&ctx.pool, "auth_registered").await, 1);
        assert!(!detail.contains(body["token"].as_str().expect("token")));
    }

    #[tokio::test]
    async fn register_handler_maps_duplicate_email_to_conflict() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let first = send_json(
            &ctx.app,
            Method::POST,
            "/auth/register",
            r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#,
        )
        .await;
        assert_eq!(first.status(), StatusCode::CREATED);

        let second = send_json(
            &ctx.app,
            Method::POST,
            "/auth/register",
            r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#,
        )
        .await;
        let status = second.status();
        let body = json_body(second).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"], "account already exists");
        assert_eq!(count_audit_events(&ctx.pool, "auth_registered").await, 1);
    }

    #[tokio::test]
    async fn register_handler_maps_validation_errors_to_bad_request() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/register",
            r#"{"email":" ","password":"short","workspaceName":"DubBridge"}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "email is required");
        assert_eq!(
            count_accounts_by_email(&ctx.pool, "owner@example.com").await,
            0
        );
        assert_eq!(count_audit_events(&ctx.pool, "auth_registered").await, 0);
    }

    #[tokio::test]
    async fn register_handler_fails_closed_when_audit_persistence_fails() {
        let Some(ctx) = TestContext::with_closed_audit_pool().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let response = send_json(
            &ctx.app,
            Method::POST,
            "/auth/register",
            r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            body["error"]
                .as_str()
                .expect("error")
                .contains("audit persistence failed")
        );
        assert_eq!(
            count_accounts_by_email(&ctx.pool, "owner@example.com").await,
            1
        );
    }

    async fn send_json(
        app: &axum::Router,
        method: Method,
        uri: &str,
        body: &str,
    ) -> axum::response::Response {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response")
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("json body")
    }

    async fn migrate_and_reset(pool: &PgPool) {
        sqlx::migrate!("../../infra/migrations")
            .run(pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE user_account, organizations, audit_events RESTART IDENTITY CASCADE",
        )
        .execute(pool)
        .await
        .expect("truncate auth tables");
    }

    async fn count_audit_events(pool: &PgPool, event_kind: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE event_kind = $1")
            .bind(event_kind)
            .fetch_one(pool)
            .await
            .expect("audit count")
    }

    async fn latest_audit_detail(pool: &PgPool, event_kind: &str) -> Option<String> {
        sqlx::query_scalar(
            "SELECT detail FROM audit_events WHERE event_kind = $1 ORDER BY happened_at DESC LIMIT 1",
        )
        .bind(event_kind)
        .fetch_optional(pool)
        .await
        .expect("audit detail")
    }

    async fn count_accounts_by_email(pool: &PgPool, email: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM user_account WHERE email = $1")
            .bind(email)
            .fetch_one(pool)
            .await
            .expect("account count")
    }

    async fn seed_account(pool: &PgPool, email: &str, password: &str, workspace_name: &str) {
        let auth_service = build_auth_service(pool.clone(), Duration::from_secs(24 * 60 * 60));
        auth_service
            .register(email, password, workspace_name)
            .await
            .expect("seed account");
    }
}
