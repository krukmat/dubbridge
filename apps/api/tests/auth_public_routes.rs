use std::{collections::HashMap, env, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, Hs256Issuer, PgAccountStore, SharedTokenVerifier,
    TokenVerificationError, TokenVerifier,
};
use dubbridge_storage::LocalFsAdapter;
use sqlx::PgPool;
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone, Default)]
struct StubTokenVerifier {
    responses: HashMap<String, Result<AuthenticatedPrincipal, TokenVerificationError>>,
}

impl StubTokenVerifier {
    fn with_token(
        mut self,
        token: &str,
        result: Result<AuthenticatedPrincipal, TokenVerificationError>,
    ) -> Self {
        self.responses.insert(token.to_string(), result);
        self
    }
}

impl TokenVerifier for StubTokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
        self.responses
            .get(token)
            .cloned()
            .unwrap_or(Err(TokenVerificationError::MalformedToken))
    }
}

struct TestContext {
    pool: PgPool,
    app: axum::Router,
    read_token: String,
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
        let read_token = "assets-read-token".to_string();
        let principal_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid");
        let verifier: SharedTokenVerifier = Arc::new(StubTokenVerifier::default().with_token(
            &read_token,
            Ok(AuthenticatedPrincipal::new(
                principal_id,
                ["assets:read"].into_iter().map(str::to_string),
            )),
        ));
        let auth_service = build_auth_service(pool.clone(), Duration::from_secs(24 * 60 * 60));
        let state = Arc::new(AppState::with_auth_service(
            pool.clone(),
            Box::new(LocalFsAdapter::new(&storage_path)),
            verifier.clone(),
            dubbridge_config::AppConfig::from_env(),
            auth_service,
        ));

        Some(Self {
            pool,
            app: build_app(state, verifier),
            read_token,
            _storage_dir: storage_dir,
            _storage_path: storage_path,
        })
    }
}

#[tokio::test]
async fn auth_routes_are_public_but_api_routes_stay_protected() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let register_response = send_json(
        &ctx.app,
        Method::POST,
        "/auth/register",
        r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#,
    )
    .await;
    let register_status = register_response.status();
    let register_body = json_body(register_response).await;

    let login_response = send_json(
        &ctx.app,
        Method::POST,
        "/auth/login",
        r#"{"email":"owner@example.com","password":"correct horse battery staple"}"#,
    )
    .await;
    let login_status = login_response.status();
    let login_body = json_body(login_response).await;

    let protected_without_bearer =
        send_request(&ctx.app, Method::GET, "/assets", None, Body::empty()).await;

    let protected_with_bearer = send_request(
        &ctx.app,
        Method::GET,
        "/assets",
        Some(&ctx.read_token),
        Body::empty(),
    )
    .await;

    assert_eq!(register_status, StatusCode::CREATED);
    assert!(
        register_body["token"]
            .as_str()
            .expect("register token")
            .len()
            > 20
    );
    assert_eq!(login_status, StatusCode::OK);
    assert!(login_body["token"].as_str().expect("login token").len() > 20);
    assert_eq!(protected_without_bearer.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(protected_with_bearer.status(), StatusCode::OK);
    assert_eq!(
        count_accounts_by_email(&ctx.pool, "owner@example.com").await,
        1
    );
}

fn build_auth_service(pool: PgPool, expiry: Duration) -> dubbridge_api::state::SharedAuthService {
    let issuer = Hs256Issuer::new("public-auth-route-test-secret", expiry).expect("issuer");
    Arc::new(dubbridge_api::state::ApiAuthService::new(
        PgAccountStore::new(pool),
        issuer,
    ))
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

async fn send_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    bearer_token: Option<&str>,
    body: Body,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    app.clone()
        .oneshot(builder.body(body).expect("request"))
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
        "TRUNCATE TABLE user_account, organizations, assets, ingest_sessions, audit_events RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate auth/public route tables");
}

async fn count_accounts_by_email(pool: &PgPool, email: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM user_account WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("account count")
}
