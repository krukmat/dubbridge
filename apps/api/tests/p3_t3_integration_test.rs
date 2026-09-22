use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_p2p::key_wrap::wrap_ck;
use dubbridge_storage::LocalFsAdapter;
use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

static KEK_ENV_LOCK: Mutex<()> = Mutex::new(());

const OWNER_TOKEN: &str = "p3-t3-owner-token";
const VIEWER_TOKEN: &str = "p3-t3-viewer-token";
const OUTSIDER_TOKEN: &str = "p3-t3-outsider-token";
const KEK_ID: &str = "kek-test";
const KEK_VERSION: i32 = 1;
const DEVICE_KEY_ID: &str = "dubbridge-p2p-k1-v1";
const DEVICE_SPKI_BASE64: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEaxfR8uEsQkf4vOblY6RA8ncDfYEt6zOg9KE5RdiYwpZP40Li/hp/m47n60p8D54WK84zV2sxXs7LtkBoN79R9Q==";

#[derive(Clone, Default)]
struct StubTokenVerifier {
    responses: HashMap<String, Result<AuthenticatedPrincipal, TokenVerificationError>>,
}

impl StubTokenVerifier {
    fn with_subject(mut self, token: &str, subject_id: Uuid) -> Self {
        self.responses.insert(
            token.to_owned(),
            Ok(AuthenticatedPrincipal::new(subject_id, Vec::<String>::new())),
        );
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
    owner: Uuid,
    viewer: Uuid,
    outsider: Uuid,
    _storage: TempDir,
}

struct ClaimedFixture {
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    invitation_id: Uuid,
    authorization_id: Uuid,
    device_id: Uuid,
    token: String,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let database_url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let owner = Uuid::new_v4();
        let viewer = Uuid::new_v4();
        let outsider = Uuid::new_v4();
        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_subject(OWNER_TOKEN, owner)
                .with_subject(VIEWER_TOKEN, viewer)
                .with_subject(OUTSIDER_TOKEN, outsider),
        );

        let storage = TempDir::new().expect("temp storage");
        let storage_path = PathBuf::from(storage.path());
        let state = Arc::new(AppState::new(
            pool.clone(),
            Box::new(LocalFsAdapter::new(storage_path)),
            verifier.clone(),
            dubbridge_config::AppConfig::from_env(),
        ));

        Some(Self {
            pool,
            app: build_app(state, verifier),
            owner,
            viewer,
            outsider,
            _storage: storage,
        })
    }

    async fn claimed_fixture(&self) -> ClaimedFixture {
        let (asset_id, publication_id, lineage_id) =
            insert_ready_publication(&self.pool, self.owner).await;

        let device_response = send_json(
            &self.app,
            Method::POST,
            "/p2p/devices",
            VIEWER_TOKEN,
            json!({
                "key_id": DEVICE_KEY_ID,
                "public_key_spki_base64": DEVICE_SPKI_BASE64,
            }),
        )
        .await;
        assert_eq!(device_response.status(), StatusCode::OK);
        let device_body = json_body(device_response).await;
        let device_id = parse_uuid(&device_body["id"]);

        let invitation_response = send_json(
            &self.app,
            Method::POST,
            &format!("/assets/{asset_id}/p2p/invitations"),
            OWNER_TOKEN,
            json!({"ttl_seconds": 3600}),
        )
        .await;
        assert_eq!(invitation_response.status(), StatusCode::CREATED);
        let invitation_body = json_body(invitation_response).await;
        let invitation_id = parse_uuid(&invitation_body["invitation"]["id"]);
        let token = invitation_body["token"]
            .as_str()
            .expect("raw invitation token returned once")
            .to_owned();

        let stored_hash: Vec<u8> =
            sqlx::query_scalar("SELECT token_hash FROM p2p_invitations WHERE id = $1")
                .bind(invitation_id)
                .fetch_one(&self.pool)
                .await
                .expect("load persisted token hash");
        assert_eq!(stored_hash.len(), 32);
        assert_ne!(stored_hash.as_slice(), token.as_bytes());

        let claim_response = send_json(
            &self.app,
            Method::POST,
            "/p2p/invitations/claim",
            VIEWER_TOKEN,
            json!({
                "token": token,
                "device_id": device_id,
            }),
        )
        .await;
        assert_eq!(claim_response.status(), StatusCode::OK);
        let claim_body = json_body(claim_response).await;
        let authorization_id = parse_uuid(&claim_body["authorization"]["id"]);

        assert_eq!(parse_uuid(&claim_body["invitation"]["id"]), invitation_id);
        assert_eq!(parse_uuid(&claim_body["authorization"]["invitation_id"]), invitation_id);
        assert_eq!(parse_uuid(&claim_body["authorization"]["asset_id"]), asset_id);
        assert_eq!(
            parse_uuid(&claim_body["authorization"]["publication_id"]),
            publication_id
        );
        assert_eq!(parse_uuid(&claim_body["authorization"]["lineage_id"]), lineage_id);
        assert_eq!(
            parse_uuid(&claim_body["authorization"]["viewer_subject_id"]),
            self.viewer
        );
        assert_eq!(parse_uuid(&claim_body["authorization"]["device_id"]), device_id);
        assert_eq!(parse_uuid(&claim_body["descriptor"]["asset_id"]), asset_id);
        assert_eq!(
            parse_uuid(&claim_body["descriptor"]["publication_id"]),
            publication_id
        );
        assert_eq!(parse_uuid(&claim_body["descriptor"]["lineage_id"]), lineage_id);

        ClaimedFixture {
            asset_id,
            publication_id,
            lineage_id,
            invitation_id,
            authorization_id,
            device_id,
            token: claim_body["invitation"]["id"]
                .as_str()
                .map(|_| String::new())
                .unwrap_or_default(),
        }
    }
}

#[tokio::test]
async fn p3_t3a_owner_invite_claim_o3_and_envelope_binding_are_integrated() {
    let _env_guard = KEK_ENV_LOCK.lock().expect("KEK env lock");
    configure_kek_env();

    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping P3.T3 integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let fixture = ctx.claimed_fixture().await;

    let envelope_response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/p2p/authorizations/{}/device-envelope",
            fixture.authorization_id
        ),
        VIEWER_TOKEN,
        None,
    )
    .await;
    assert_eq!(envelope_response.status(), StatusCode::OK);
    let envelope = json_body(envelope_response).await;
    let binding: Value = serde_json::from_str(
        envelope["binding_json"]
            .as_str()
            .expect("binding json"),
    )
    .expect("parse envelope binding");

    assert_eq!(envelope["profile_version"], "p2p-k1-hpke-v1");
    assert_eq!(envelope["key_id"], DEVICE_KEY_ID);
    assert_eq!(binding["device_key_id"], DEVICE_KEY_ID);
    assert_eq!(binding["invitation_id"], fixture.invitation_id.to_string());
    assert_eq!(binding["viewer_id"], ctx.viewer.to_string());
    assert_eq!(binding["asset_id"], fixture.asset_id.to_string());
    assert_eq!(binding["publication_id"], fixture.publication_id.to_string());
    assert_eq!(binding["lineage_id"], fixture.lineage_id.to_string());
    assert_eq!(
        binding["authorization_id"],
        fixture.authorization_id.to_string()
    );
    assert!(
        binding["expires_at_unix"]
            .as_i64()
            .expect("binding expiry")
            > OffsetDateTime::now_utc().unix_timestamp()
    );
    assert!(
        !envelope["encapsulated_key_base64"]
            .as_str()
            .expect("encapsulated key")
            .is_empty()
    );
    assert!(
        !envelope["ciphertext_base64"]
            .as_str()
            .expect("ciphertext")
            .is_empty()
    );

    let release_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE event_kind = 'p2p_device_envelope_released' AND correlation_id = $1",
    )
    .bind(fixture.authorization_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("count release audit");
    assert_eq!(release_events, 1);
}

#[tokio::test]
async fn p3_t3b_envelope_release_fails_closed_across_live_o3_device_and_package_boundaries() {
    let _env_guard = KEK_ENV_LOCK.lock().expect("KEK env lock");
    configure_kek_env();

    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping P3.T3 integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let wrong_viewer = ctx.claimed_fixture().await;
    assert_denied(&ctx.app, wrong_viewer.authorization_id, OUTSIDER_TOKEN).await;

    let missing_authorization = ctx.claimed_fixture().await;
    sqlx::query("DELETE FROM p2p_audience_authorizations WHERE id = $1")
        .bind(missing_authorization.authorization_id)
        .execute(&ctx.pool)
        .await
        .expect("delete authorization");
    assert_denied(
        &ctx.app,
        missing_authorization.authorization_id,
        VIEWER_TOKEN,
    )
    .await;

    let revoked_authorization = ctx.claimed_fixture().await;
    sqlx::query("UPDATE p2p_audience_authorizations SET revoked_at = now() WHERE id = $1")
        .bind(revoked_authorization.authorization_id)
        .execute(&ctx.pool)
        .await
        .expect("revoke authorization");
    assert_denied(
        &ctx.app,
        revoked_authorization.authorization_id,
        VIEWER_TOKEN,
    )
    .await;

    let expired_authorization = ctx.claimed_fixture().await;
    sqlx::query(
        "UPDATE p2p_audience_authorizations SET expires_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(expired_authorization.authorization_id)
    .execute(&ctx.pool)
    .await
    .expect("expire authorization");
    assert_denied(
        &ctx.app,
        expired_authorization.authorization_id,
        VIEWER_TOKEN,
    )
    .await;

    let revoked_device = ctx.claimed_fixture().await;
    sqlx::query("UPDATE p2p_devices SET revoked_at = now() WHERE id = $1")
        .bind(revoked_device.device_id)
        .execute(&ctx.pool)
        .await
        .expect("revoke device");
    assert_denied(&ctx.app, revoked_device.authorization_id, VIEWER_TOKEN).await;

    let non_ready = ctx.claimed_fixture().await;
    sqlx::query("UPDATE p2p_publications SET state = 'failed' WHERE id = $1")
        .bind(non_ready.publication_id)
        .execute(&ctx.pool)
        .await
        .expect("fail publication");
    assert_denied(&ctx.app, non_ready.authorization_id, VIEWER_TOKEN).await;

    let undelivered = ctx.claimed_fixture().await;
    sqlx::query(
        "UPDATE p2p_publication_outbox SET delivery_state = 'pending', delivered_at = NULL WHERE publication_id = $1 AND lineage_id = $2",
    )
    .bind(undelivered.publication_id)
    .bind(undelivered.lineage_id)
    .execute(&ctx.pool)
    .await
    .expect("remove durable delivery evidence");
    assert_denied(&ctx.app, undelivered.authorization_id, VIEWER_TOKEN).await;
}

async fn assert_denied(app: &axum::Router, authorization_id: Uuid, token: &str) {
    let response = send_request(
        app,
        Method::GET,
        &format!("/p2p/authorizations/{authorization_id}/device-envelope"),
        token,
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("read denial body");
    assert!(bytes.is_empty(), "denial must not return envelope material");
}

async fn insert_ready_publication(pool: &PgPool, owner: Uuid) -> (Uuid, Uuid, Uuid) {
    let asset_id = Uuid::new_v4();
    let publication_id = Uuid::new_v4();
    let lineage_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let kek = [3_u8; 32];
    let wrapped = wrap_ck(&[7_u8; 32], &kek, KEK_ID, KEK_VERSION as u32).expect("wrap CK");

    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, 'finalized')",
    )
    .bind(asset_id)
    .bind("P3 T3 integration asset")
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert asset");

    sqlx::query(
        r#"
        INSERT INTO p2p_publications (
            id, asset_id, lineage_id, state,
            external_publication_id, confirmed_lineage_id, external_confirmed_at,
            sealed_kek_id, sealed_kek_version, sealed_nonce, sealed_wrapped_ck, sealed_at,
            manifest_digest_sha256
        )
        VALUES (
            $1, $2, $3, 'ready',
            $4, $3, $5,
            $6, $7, $8, $9, $5,
            $10
        )
        "#,
    )
    .bind(publication_id)
    .bind(asset_id)
    .bind(lineage_id)
    .bind(format!("hyperdrive:{publication_id}"))
    .bind(now)
    .bind(&wrapped.kek_id)
    .bind(KEK_VERSION)
    .bind(wrapped.nonce.to_vec())
    .bind(wrapped.ciphertext)
    .bind("a".repeat(64))
    .execute(pool)
    .await
    .expect("insert ready publication");

    sqlx::query(
        r#"
        INSERT INTO p2p_publication_outbox (
            id, publication_id, lineage_id, delivery_state, delivered_at
        )
        VALUES ($1, $2, $3, 'delivered', $4)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(publication_id)
    .bind(lineage_id)
    .bind(now)
    .execute(pool)
    .await
    .expect("insert delivered outbox");

    (asset_id, publication_id, lineage_id)
}

async fn send_json(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: &str,
    value: Value,
) -> axum::response::Response {
    send_request(
        app,
        method,
        uri,
        token,
        Some(serde_json::to_vec(&value).expect("encode json")),
    )
    .await
}

async fn send_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: &str,
    body: Option<Vec<u8>>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    let request_body = if let Some(bytes) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(bytes)
    } else {
        Body::empty()
    };
    app.clone()
        .oneshot(builder.body(request_body).expect("request"))
        .await
        .expect("response")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read json body"),
    )
    .expect("parse json response")
}

fn parse_uuid(value: &Value) -> Uuid {
    Uuid::parse_str(value.as_str().expect("uuid string")).expect("valid uuid")
}

fn configure_kek_env() {
    unsafe {
        env::set_var("DUBBRIDGE_P2P_KEK_ID", KEK_ID);
        env::set_var("DUBBRIDGE_P2P_KEK_VERSION", KEK_VERSION.to_string());
        env::set_var("DUBBRIDGE_P2P_KEK_HEX", "03".repeat(32));
    }
}
