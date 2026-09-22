use std::{
    collections::{BTreeSet, HashMap},
    env,
    path::PathBuf,
    sync::Arc,
};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::audit_repo::insert_audit_event;
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
};
use dubbridge_p2p::key_wrap::wrap_ck;
use dubbridge_storage::LocalFsAdapter;
use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

const OWNER_TOKEN: &str = "p3-t3-owner-token";
const VIEWER_TOKEN: &str = "p3-t3-viewer-token";
const OUTSIDER_TOKEN: &str = "p3-t3-outsider-token";
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
            Ok(AuthenticatedPrincipal::new(
                subject_id,
                Vec::<String>::new(),
            )),
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

struct TestKek {
    id: String,
    version: i32,
    key: [u8; 32],
}

struct TestContext {
    pool: PgPool,
    app: axum::Router,
    owner: Uuid,
    viewer: Uuid,
    kek: TestKek,
    _storage: TempDir,
}

struct ClaimedFixture {
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    invitation_id: Uuid,
    authorization_id: Uuid,
    device_id: Uuid,
    raw_invitation_token: String,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let database_url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let kek = test_kek_from_env()?;
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
            kek,
            _storage: storage,
        })
    }

    async fn claimed_fixture(&self) -> ClaimedFixture {
        let (asset_id, publication_id, lineage_id) =
            insert_ready_publication(&self.pool, self.owner, &self.kek).await;
        let device_id = self.register_viewer_device().await;
        let (invitation_id, token) = self.create_owner_invitation(asset_id).await;
        self.assert_token_hash_only(invitation_id, &token).await;
        let authorization_id = self
            .claim_and_assert_identity(
                &token,
                device_id,
                invitation_id,
                asset_id,
                publication_id,
                lineage_id,
            )
            .await;

        ClaimedFixture {
            asset_id,
            publication_id,
            lineage_id,
            invitation_id,
            authorization_id,
            device_id,
            raw_invitation_token: token,
        }
    }

    async fn register_viewer_device(&self) -> Uuid {
        let response = send_json(
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
        assert_eq!(response.status(), StatusCode::OK);
        parse_uuid(&json_body(response).await["id"])
    }

    async fn create_owner_invitation(&self, asset_id: Uuid) -> (Uuid, String) {
        let response = send_json(
            &self.app,
            Method::POST,
            &format!("/assets/{asset_id}/p2p/invitations"),
            OWNER_TOKEN,
            json!({"ttl_seconds": 3600}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = json_body(response).await;
        let invitation_id = parse_uuid(&body["invitation"]["id"]);
        let token = body["token"]
            .as_str()
            .expect("raw invitation token returned once")
            .to_owned();
        (invitation_id, token)
    }

    async fn assert_token_hash_only(&self, invitation_id: Uuid, token: &str) {
        let stored_hash: Vec<u8> =
            sqlx::query_scalar("SELECT token_hash FROM p2p_invitations WHERE id = $1")
                .bind(invitation_id)
                .fetch_one(&self.pool)
                .await
                .expect("load persisted token hash");
        assert_eq!(stored_hash.len(), 32);
        assert_ne!(stored_hash.as_slice(), token.as_bytes());
    }

    #[allow(clippy::too_many_arguments)]
    async fn claim_and_assert_identity(
        &self,
        token: &str,
        device_id: Uuid,
        invitation_id: Uuid,
        asset_id: Uuid,
        publication_id: Uuid,
        lineage_id: Uuid,
    ) -> Uuid {
        let response = send_json(
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
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_claim_identity(
            &body,
            invitation_id,
            asset_id,
            publication_id,
            lineage_id,
            self.viewer,
            device_id,
        );
        parse_uuid(&body["authorization"]["id"])
    }
}

#[allow(clippy::too_many_arguments)]
fn assert_claim_identity(
    body: &Value,
    invitation_id: Uuid,
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    viewer_id: Uuid,
    device_id: Uuid,
) {
    assert_eq!(parse_uuid(&body["invitation"]["id"]), invitation_id);
    assert_eq!(
        parse_uuid(&body["authorization"]["invitation_id"]),
        invitation_id
    );
    assert_eq!(parse_uuid(&body["authorization"]["asset_id"]), asset_id);
    assert_eq!(
        parse_uuid(&body["authorization"]["publication_id"]),
        publication_id
    );
    assert_eq!(parse_uuid(&body["authorization"]["lineage_id"]), lineage_id);
    assert_eq!(
        parse_uuid(&body["authorization"]["viewer_subject_id"]),
        viewer_id
    );
    assert_eq!(parse_uuid(&body["authorization"]["device_id"]), device_id);
    assert_eq!(parse_uuid(&body["descriptor"]["asset_id"]), asset_id);
    assert_eq!(
        parse_uuid(&body["descriptor"]["publication_id"]),
        publication_id
    );
    assert_eq!(parse_uuid(&body["descriptor"]["lineage_id"]), lineage_id);
}

#[tokio::test]
async fn p3_t3a_owner_invite_claim_o3_and_envelope_binding_are_integrated() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping P3.T3 integration test: DB or deterministic test KEK not set");
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
    let binding: Value =
        serde_json::from_str(envelope["binding_json"].as_str().expect("binding json"))
            .expect("parse envelope binding");

    assert_eq!(envelope["profile_version"], "p2p-k1-hpke-v1");
    assert_eq!(envelope["key_id"], DEVICE_KEY_ID);
    assert_eq!(binding["device_key_id"], DEVICE_KEY_ID);
    assert_eq!(binding["invitation_id"], fixture.invitation_id.to_string());
    assert_eq!(binding["viewer_id"], ctx.viewer.to_string());
    assert_eq!(binding["asset_id"], fixture.asset_id.to_string());
    assert_eq!(
        binding["publication_id"],
        fixture.publication_id.to_string()
    );
    assert_eq!(binding["lineage_id"], fixture.lineage_id.to_string());
    assert_eq!(
        binding["authorization_id"],
        fixture.authorization_id.to_string()
    );
    assert!(
        binding["expires_at_unix"].as_i64().expect("binding expiry")
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

    assert_partial_p3_package_audit_rejected(&ctx, &fixture).await;
}

async fn assert_partial_p3_package_audit_rejected(ctx: &TestContext, fixture: &ClaimedFixture) {
    let malformed = AuditEvent::new_p3_event(
        Some(AssetId(fixture.asset_id)),
        AuditEventKind::P2pInvitationCreated,
        Uuid::new_v4(),
        Some(fixture.publication_id),
        None,
        None,
    );
    assert!(
        insert_audit_event(&ctx.pool, &malformed).await.is_err(),
        "P3 package audit must reject incomplete publication/lineage identity"
    );
}

#[tokio::test]
async fn p3_t3b_envelope_release_fails_closed_across_live_o3_device_and_package_boundaries() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping P3.T3 integration test: DB or deterministic test KEK not set");
        return;
    };

    assert_o3_and_viewer_denials(&ctx).await;
    assert_package_denials(&ctx).await;
    assert_device_denial(&ctx).await;
}

async fn assert_o3_and_viewer_denials(ctx: &TestContext) {
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

    let revoked = ctx.claimed_fixture().await;
    sqlx::query("UPDATE p2p_audience_authorizations SET revoked_at = now() WHERE id = $1")
        .bind(revoked.authorization_id)
        .execute(&ctx.pool)
        .await
        .expect("revoke authorization");
    assert_denied(&ctx.app, revoked.authorization_id, VIEWER_TOKEN).await;

    let expired = ctx.claimed_fixture().await;
    sqlx::query(
        "UPDATE p2p_audience_authorizations SET expires_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(expired.authorization_id)
    .execute(&ctx.pool)
    .await
    .expect("expire authorization");
    assert_denied(&ctx.app, expired.authorization_id, VIEWER_TOKEN).await;
}

async fn assert_package_denials(ctx: &TestContext) {
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

async fn assert_device_denial(ctx: &TestContext) {
    let revoked = ctx.claimed_fixture().await;
    sqlx::query("UPDATE p2p_devices SET revoked_at = now() WHERE id = $1")
        .bind(revoked.device_id)
        .execute(&ctx.pool)
        .await
        .expect("revoke device");
    assert_denied(&ctx.app, revoked.authorization_id, VIEWER_TOKEN).await;
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

#[tokio::test]
async fn p3_t3c_backend_audit_and_storage_are_secret_boundary_clean() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping P3.T3 integration test: DB or deterministic test KEK not set");
        return;
    };
    let fixture = ctx.claimed_fixture().await;

    let response = send_request(
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
    assert_eq!(response.status(), StatusCode::OK);
    let envelope = json_body(response).await;
    assert_envelope_secret_clean(&ctx, &fixture, &envelope);

    assert_denied(&ctx.app, fixture.authorization_id, OUTSIDER_TOKEN).await;

    let forbidden_values = forbidden_secret_values(&ctx, &fixture).await;
    assert_p3_audit_secret_clean(&ctx, &fixture, &forbidden_values).await;
    assert_p3_storage_secret_clean(&ctx, &fixture).await;
}

fn assert_envelope_secret_clean(ctx: &TestContext, fixture: &ClaimedFixture, envelope: &Value) {
    let serialized = serde_json::to_string(envelope).expect("serialize envelope");
    let raw_ck_base64 = BASE64_STANDARD.encode([7_u8; 32]);
    let raw_kek_hex: String = ctx
        .kek
        .key
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    for secret in [
        fixture.raw_invitation_token.as_str(),
        raw_ck_base64.as_str(),
        raw_kek_hex.as_str(),
        OWNER_TOKEN,
        VIEWER_TOKEN,
        OUTSIDER_TOKEN,
    ] {
        assert!(
            !serialized.contains(secret),
            "device-envelope response leaked forbidden secret material"
        );
    }

    for forbidden_key in [
        "token",
        "token_hash",
        "raw_token",
        "ck",
        "plaintext_ck",
        "kek",
        "kek_hex",
        "wrapped_ck",
        "sealed_wrapped_ck",
        "nonce",
        "sealed_nonce",
        "private_key",
        "shared_secret",
        "jwt",
    ] {
        assert!(
            envelope.get(forbidden_key).is_none(),
            "device-envelope response exposed forbidden field"
        );
    }
}

async fn forbidden_secret_values(ctx: &TestContext, fixture: &ClaimedFixture) -> Vec<String> {
    let token_hash: Vec<u8> =
        sqlx::query_scalar("SELECT token_hash FROM p2p_invitations WHERE id = $1")
            .bind(fixture.invitation_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("load invitation token hash");
    let (sealed_nonce, sealed_wrapped_ck): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT sealed_nonce, sealed_wrapped_ck FROM p2p_publications WHERE id = $1",
    )
    .bind(fixture.publication_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("load sealed publication key material");

    vec![
        fixture.raw_invitation_token.clone(),
        BASE64_STANDARD.encode(token_hash),
        BASE64_STANDARD.encode([7_u8; 32]),
        BASE64_STANDARD.encode(ctx.kek.key),
        ctx.kek
            .key
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        BASE64_STANDARD.encode(sealed_nonce),
        BASE64_STANDARD.encode(sealed_wrapped_ck),
        OWNER_TOKEN.to_owned(),
        VIEWER_TOKEN.to_owned(),
        OUTSIDER_TOKEN.to_owned(),
    ]
}

async fn assert_p3_audit_secret_clean(
    ctx: &TestContext,
    fixture: &ClaimedFixture,
    forbidden_values: &[String],
) {
    let correlations = vec![
        fixture.device_id,
        fixture.invitation_id,
        fixture.authorization_id,
    ];
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT event_kind, detail
          FROM audit_events
         WHERE correlation_id = ANY($1)
         ORDER BY happened_at, id
        "#,
    )
    .bind(correlations)
    .fetch_all(&ctx.pool)
    .await
    .expect("load P3 audit rows");

    let actual: BTreeSet<&str> = rows.iter().map(|(kind, _)| kind.as_str()).collect();
    let expected = BTreeSet::from([
        "p2p_device_registered",
        "p2p_invitation_created",
        "p2p_invitation_claimed",
        "p2p_audience_authorization_issued",
        "p2p_device_envelope_released",
        "p2p_audience_access_denied",
    ]);
    assert_eq!(actual, expected, "P3 audit correlation inventory drifted");

    for (_, detail) in rows {
        let Some(detail) = detail else {
            continue;
        };
        let value: Value = serde_json::from_str(&detail).expect("P3 audit detail JSON");
        assert_json_secret_clean(&value, forbidden_values);
    }
}

fn assert_json_secret_clean(value: &Value, forbidden_values: &[String]) {
    const FORBIDDEN_KEYS: [&str; 17] = [
        "token",
        "raw_token",
        "token_hash",
        "ck",
        "plaintext_ck",
        "content_key",
        "kek",
        "kek_hex",
        "wrapped_ck",
        "sealed_wrapped_ck",
        "nonce",
        "sealed_nonce",
        "private_key",
        "private_key_bytes",
        "shared_secret",
        "hpke_secret",
        "jwt",
    ];

    match value {
        Value::Object(map) => {
            for (key, child) in map {
                assert!(
                    !FORBIDDEN_KEYS.contains(&key.as_str()),
                    "P3 audit detail contains forbidden secret key"
                );
                assert_json_secret_clean(child, forbidden_values);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_json_secret_clean(child, forbidden_values);
            }
        }
        Value::String(text) => {
            for secret in forbidden_values {
                assert!(
                    secret.is_empty() || !text.contains(secret),
                    "P3 audit detail contains forbidden secret value"
                );
            }
        }
        _ => {}
    }
}

async fn assert_p3_storage_secret_clean(ctx: &TestContext, fixture: &ClaimedFixture) {
    let columns: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT table_name, column_name
          FROM information_schema.columns
         WHERE table_schema = 'public'
           AND table_name IN (
               'p2p_invitations',
               'p2p_devices',
               'p2p_audience_authorizations',
               'p2p_publications'
           )
        "#,
    )
    .fetch_all(&ctx.pool)
    .await
    .expect("inspect P3 persistence columns");

    let forbidden_columns = [
        "token",
        "raw_token",
        "ck",
        "plaintext_ck",
        "content_key",
        "kek",
        "kek_bytes",
        "raw_kek",
        "private_key",
        "private_key_bytes",
        "jwt",
        "session_token",
    ];
    for (_, column) in columns {
        assert!(
            !forbidden_columns.contains(&column.as_str()),
            "P3 persistence schema contains forbidden plaintext-secret column"
        );
    }

    let token_hash: Vec<u8> =
        sqlx::query_scalar("SELECT token_hash FROM p2p_invitations WHERE id = $1")
            .bind(fixture.invitation_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("load persisted token hash");
    assert_eq!(token_hash.len(), 32);
    assert_ne!(
        token_hash.as_slice(),
        fixture.raw_invitation_token.as_bytes(),
        "raw invitation token must never be persisted"
    );

    let (sealed_nonce, sealed_wrapped_ck): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT sealed_nonce, sealed_wrapped_ck FROM p2p_publications WHERE id = $1",
    )
    .bind(fixture.publication_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("load persisted sealed K1 material");
    let raw_ck = vec![7_u8; 32];
    assert_ne!(sealed_nonce, raw_ck, "raw CK must not be stored as nonce");
    assert_ne!(
        sealed_wrapped_ck, raw_ck,
        "raw CK must not be stored in the wrapped-CK column"
    );
}

async fn insert_ready_publication(pool: &PgPool, owner: Uuid, kek: &TestKek) -> (Uuid, Uuid, Uuid) {
    let asset_id = Uuid::new_v4();
    let publication_id = Uuid::new_v4();
    let lineage_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let wrapped = wrap_ck(&[7_u8; 32], &kek.key, &kek.id, kek.version as u32).expect("wrap CK");

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
    .bind(kek.version)
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

fn test_kek_from_env() -> Option<TestKek> {
    let id = env::var("DUBBRIDGE_P2P_KEK_ID").ok()?;
    let version = env::var("DUBBRIDGE_P2P_KEK_VERSION")
        .ok()?
        .parse::<i32>()
        .ok()?;
    let hex = env::var("DUBBRIDGE_P2P_KEK_HEX").ok()?;
    if id.trim().is_empty() || version <= 0 || hex.len() != 64 {
        return None;
    }
    let mut key = [0_u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(TestKek { id, version, key })
}
