//! Inert, opt-in DubBridge -> Fenix service-to-service client.
//! This module is not wired into any DubBridge route or background worker.
//! The Fenix provider admission HTTP route has not been activated or approved.

use std::fmt;
use std::future::Future;
use std::time::Duration;

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const CONTRACT_VERSION: &str = "1";
const SERVICE_TOKEN_PREFIX: &str = "fenix_s2s_v1_";
const OPERATIONS_PATH: &str = "/integrations/dubbridge/operations";
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_REPLY_BYTES: usize = 8 * 1024;
const MAX_INPUT_NODES: usize = 4096;
const MAX_INPUT_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FenixClientError {
    #[error("Fenix integration is not configured")]
    Configuration,
    #[error("Fenix service authorization denied")]
    Denied,
    #[error("invalid Fenix operation request")]
    InvalidRequest,
    #[error("Fenix reported an operation conflict")]
    Conflict,
    #[error("Fenix operation outcome requires reconciliation")]
    AmbiguousOutcome,
    #[error("Fenix returned an invalid admission acknowledgement")]
    InvalidResponse,
}

/// An unknown outcome is never automatically retried: the remote operation
/// may already have been durably reserved. The caller retains its operation ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAdvice {
    Never,
    ReconcileBeforeAnyRetry,
}

impl FenixClientError {
    pub fn retry_advice(self) -> RetryAdvice {
        match self {
            Self::AmbiguousOutcome | Self::InvalidResponse => {
                RetryAdvice::ReconcileBeforeAnyRetry
            }
            _ => RetryAdvice::Never,
        }
    }
}

/// An opaque Fenix-issued service token; never reuse a DubBridge human JWT.
/// Debug intentionally redacts it, and the connector never stores it in config.
pub struct FenixBearer(String);

impl FenixBearer {
    pub fn from_secret(secret: String) -> Result<Self, FenixClientError> {
        let Some(encoded) = secret.strip_prefix(SERVICE_TOKEN_PREFIX) else {
            return Err(FenixClientError::Configuration);
        };
        if encoded.len() != 43
            || !encoded
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'-' | b'_'))
        {
            return Err(FenixClientError::Configuration);
        }
        Ok(Self(secret))
    }

    fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for FenixBearer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FenixBearer([redacted])")
    }
}

/// Implemented by the operator-managed secret adapter, never a user JWT store.
pub trait FenixCredentialProvider: Send + Sync {
    fn service_token(&self) -> Result<FenixBearer, FenixClientError>;
}

/// Local human/workspace identity must come from DubBridge's own auth context,
/// not from the untrusted body of an S2S request.
#[derive(Debug, Clone, Copy)]
pub struct DubBridgeActor {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
}

/// Requested local scope is not an authorization grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantSelection {
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
}

/// The implementation MUST query DubBridge's trusted local permission store.
/// A public API handler must never use an unconditional allow implementation.
pub trait FenixScopeAuthorizer: Send + Sync {
    fn authorize_scope<'a>(
        &'a self,
        actor: &'a DubBridgeActor,
        requested: &'a TenantSelection,
    ) -> impl Future<Output = Result<TenantSelection, FenixClientError>> + Send + 'a;
}

pub struct OperationParts {
    /// Persisted local idempotency identity, reused verbatim after ambiguity.
    pub operation_id: String,
    pub trace_id: String,
    pub capability_key: String,
    pub input_schema_version: String,
    pub input: Value,
}

impl OperationParts {
    /// Derives both wire identifiers deterministically from an ALREADY
    /// persisted DubBridge work-item UUID. Caller owns the durable local
    /// record; this method does not create or validate that record.
    /// On ambiguous delivery use the same local work ID and capability key.
    pub fn from_persisted_work_item(
        work_item_id: Uuid,
        capability_key: String,
        input_schema_version: String,
        input: Value,
    ) -> Result<Self, FenixClientError> {
        if work_item_id.is_nil() || !valid_capability_key(&capability_key) {
            return Err(FenixClientError::InvalidRequest);
        }
        let stable_name = format!("{work_item_id}:{capability_key}");
        let operation_namespace = Uuid::from_u128(0x8a06362792aa4fe291199ce58636f012);
        let trace_namespace = Uuid::from_u128(0xcac39fc79892481ab9fed8c12b0b0a7e);
        Ok(Self {
            operation_id: Uuid::new_v5(&operation_namespace, stable_name.as_bytes()).to_string(),
            trace_id: Uuid::new_v5(&trace_namespace, stable_name.as_bytes()).to_string(),
            capability_key,
            input_schema_version,
            input,
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct FenixSource {
    product: &'static str,
    org_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FenixOperationRequest {
    contract_version: &'static str,
    operation_id: String,
    capability_key: String,
    input_schema_version: String,
    trace_id: String,
    source: FenixSource,
    input: Value,
}

impl FenixOperationRequest {
    /// Every request must cross a DubBridge local authorization gate.
    /// This does not itself implement org/project permission checks.
    pub async fn build<P: FenixScopeAuthorizer>(
        policy: &P,
        actor: &DubBridgeActor,
        requested: &TenantSelection,
        parts: OperationParts,
    ) -> Result<Self, FenixClientError> {
        if actor.user_id.is_nil() || actor.workspace_id.is_nil() {
            return Err(FenixClientError::Denied);
        }
        let approved = policy.authorize_scope(actor, requested).await?;
        if approved != *requested {
            return Err(FenixClientError::Denied);
        }
        Self::from_authorized(approved, parts)
    }

    fn from_authorized(
        scope: TenantSelection,
        parts: OperationParts,
    ) -> Result<Self, FenixClientError> {
        if !valid_operation_id(&parts.operation_id, 16)
            || !valid_operation_id(&parts.trace_id, 8)
            || !valid_capability_key(&parts.capability_key)
            || !valid_schema_version(&parts.input_schema_version)
        {
            return Err(FenixClientError::InvalidRequest);
        }
        let Value::Object(fields) = &parts.input else {
            return Err(FenixClientError::InvalidRequest);
        };
        if fields.len() > 64 || !valid_input_tree(&parts.input, 0, &mut 0) {
            return Err(FenixClientError::InvalidRequest);
        }
        Ok(Self {
            contract_version: CONTRACT_VERSION,
            operation_id: parts.operation_id,
            capability_key: parts.capability_key,
            input_schema_version: parts.input_schema_version,
            trace_id: parts.trace_id,
            source: FenixSource {
                product: "dubbridge",
                org_id: scope.org_id,
                project_id: scope.project_id,
            },
            input: parts.input,
        })
    }

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    fn bounded_json(&self) -> Result<Vec<u8>, FenixClientError> {
        let bytes = serde_json::to_vec(self).map_err(|_| FenixClientError::InvalidRequest)?;
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(FenixClientError::InvalidRequest);
        }
        Ok(bytes)
    }
}

fn valid_operation_id(value: &str, min: usize) -> bool {
    (min..=96).contains(&value.len())
        && value
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'-' | b'_'))
}

fn valid_schema_version(value: &str) -> bool {
    (1..=3).contains(&value.len())
        && value.starts_with(|ch: char| matches!(ch, '1'..='9'))
        && value.bytes().all(|ch| ch.is_ascii_digit())
}

fn valid_capability_key(value: &str) -> bool {
    if value.len() > 96 || !value.starts_with(|ch: char| ch.is_ascii_lowercase()) {
        return false;
    }
    let segments: Vec<&str> = value.split(['.', '_', '-']).collect();
    (2..=7).contains(&segments.len())
        && segments.iter().all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        })
}

fn valid_input_tree(value: &Value, depth: usize, visited: &mut usize) -> bool {
    *visited += 1;
    if depth > MAX_INPUT_DEPTH || *visited > MAX_INPUT_NODES {
        return false;
    }
    match value {
        Value::Object(map) => {
            map.len() <= 64 && map.values().all(|item| valid_input_tree(item, depth + 1, visited))
        }
        Value::Array(items) => items.iter().all(|item| valid_input_tree(item, depth + 1, visited)),
        _ => true,
    }
}

/// Narrow interface permits deterministic tests without a network or token.
pub trait FenixTransport: Send + Sync {
    fn post_operation<'a>(
        &'a self,
        operation_id: &'a str,
        bearer: &'a FenixBearer,
        body: Vec<u8>,
    ) -> impl Future<Output = Result<FenixTransportReply, FenixClientError>> + Send + 'a;
}

pub struct FenixTransportReply {
    pub status: u16,
    pub body: Vec<u8>,
}

/// This transport exists only after a trusted operator explicitly constructs
/// it. No app or worker currently instantiates it, and the Fenix HTTP route
/// does not yet exist. Redirects are forbidden to protect the bearer.
pub struct FenixHttpsTransport {
    client: Client,
    base_url: Url,
}

impl fmt::Debug for FenixHttpsTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FenixHttpsTransport([operator-configured HTTPS])")
    }
}

impl FenixHttpsTransport {
    pub fn opt_in(base_url: &str, timeout: Duration) -> Result<Self, FenixClientError> {
        let url = Url::parse(base_url).map_err(|_| FenixClientError::Configuration)?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
            || timeout.is_zero()
            || timeout > Duration::from_secs(30)
        {
            return Err(FenixClientError::Configuration);
        }
        let client = Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| FenixClientError::Configuration)?;
        Ok(Self {
            client,
            base_url: url,
        })
    }
}

impl FenixTransport for FenixHttpsTransport {
    fn post_operation<'a>(
        &'a self,
        operation_id: &'a str,
        bearer: &'a FenixBearer,
        body: Vec<u8>,
    ) -> impl Future<Output = Result<FenixTransportReply, FenixClientError>> + Send + 'a {
        async move {
            let mut url = self.base_url.clone();
            url.set_path(OPERATIONS_PATH);
            let mut response = self
                .client
                .post(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", bearer.expose()))
                .header("Idempotency-Key", operation_id)
                .body(body)
                .send()
                .await
                .map_err(|_| FenixClientError::AmbiguousOutcome)?;
            let status = response.status().as_u16();
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| FenixClientError::AmbiguousOutcome)?
            {
                if chunk.len() > MAX_REPLY_BYTES.saturating_sub(bytes.len()) {
                    return Err(FenixClientError::AmbiguousOutcome);
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(FenixTransportReply { status, body: bytes })
        }
    }
}

/// Proposed admission acknowledgement is not the deferred C4 status envelope.
/// No HTTP endpoint serving this contract is currently active in Fenix.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FenixAdmissionAck {
    pub contract_version: String,
    pub operation_id: String,
    pub trace_id: String,
    pub replayed: bool,
}

/// No automatic re-send; errors keep the local durable operation ID for a
/// later approved reconciliation channel. C4 GET/polling is deferred.
pub struct FenixClient<T, C> {
    transport: T,
    credentials: C,
}

impl<T: FenixTransport, C: FenixCredentialProvider> FenixClient<T, C> {
    pub fn new(transport: T, credentials: C) -> Self {
        Self {
            transport,
            credentials,
        }
    }

    pub async fn submit_once(
        &self,
        request: &FenixOperationRequest,
    ) -> Result<FenixAdmissionAck, FenixClientError> {
        let body = request.bounded_json()?;
        let bearer = self.credentials.service_token()?;
        let response = self
            .transport
            .post_operation(request.operation_id(), &bearer, body)
            .await?;
        parse_admission_reply(response, request)
    }
}

fn parse_admission_reply(
    reply: FenixTransportReply,
    request: &FenixOperationRequest,
) -> Result<FenixAdmissionAck, FenixClientError> {
    if reply.body.len() > MAX_REPLY_BYTES {
        return Err(FenixClientError::AmbiguousOutcome);
    }
    match reply.status {
        200 | 202 => {
            let ack: FenixAdmissionAck =
                serde_json::from_slice(&reply.body).map_err(|_| FenixClientError::InvalidResponse)?;
            if ack.contract_version != CONTRACT_VERSION
                || ack.operation_id != request.operation_id
                || ack.trace_id != request.trace_id
            {
                return Err(FenixClientError::InvalidResponse);
            }
            Ok(ack)
        }
        400 => Err(FenixClientError::InvalidRequest),
        401 | 403 | 404 | 422 => Err(FenixClientError::Denied),
        409 => Err(FenixClientError::Conflict),
        _ => Err(FenixClientError::AmbiguousOutcome),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestPolicy {
        allow: bool,
    }

    impl FenixScopeAuthorizer for TestPolicy {
        fn authorize_scope<'a>(
            &'a self,
            _actor: &'a DubBridgeActor,
            requested: &'a TenantSelection,
        ) -> impl Future<Output = Result<TenantSelection, FenixClientError>> + Send + 'a {
            async move {
                if self.allow {
                    Ok(*requested)
                } else {
                    Err(FenixClientError::Denied)
                }
            }
        }
    }

    fn parts() -> OperationParts {
        OperationParts {
            operation_id: "operation_00000001".into(),
            trace_id: "trace_b3_example".into(),
            capability_key: "integration.diagnostics".into(),
            input_schema_version: "1".into(),
            input: serde_json::json!({}),
        }
    }

    fn actor() -> DubBridgeActor {
        DubBridgeActor {
            user_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
            workspace_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
        }
    }

    fn scope() -> TenantSelection {
        TenantSelection {
            org_id: Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap(),
            project_id: Some(Uuid::parse_str("44444444-4444-4444-8444-444444444444").unwrap()),
        }
    }

    async fn request() -> FenixOperationRequest {
        FenixOperationRequest::build(&TestPolicy { allow: true }, &actor(), &scope(), parts())
            .await
            .unwrap()
    }


    #[test]
    fn stable_operation_identity_reuses_persisted_local_work_item() {
        let local_id = Uuid::parse_str("c93c0764-5069-4887-b454-038b39fc826f").unwrap();
        let first = OperationParts::from_persisted_work_item(
            local_id, "integration.diagnostics".into(), "1".into(), serde_json::json!({"v": 1})
        ).unwrap();
        let changed = OperationParts::from_persisted_work_item(
            local_id, "integration.diagnostics".into(), "1".into(), serde_json::json!({"v": 2})
        ).unwrap();
        assert_eq!(first.operation_id, changed.operation_id);
        assert_eq!(first.trace_id, changed.trace_id);
        assert!(valid_operation_id(&first.operation_id, 16));
        assert!(valid_operation_id(&first.trace_id, 8));
        assert!(OperationParts::from_persisted_work_item(
            Uuid::nil(), "integration.diagnostics".into(), "1".into(), serde_json::json!({})
        ).is_err());
    }

    #[tokio::test]
    async fn local_denial_prevents_envelope_and_no_human_jwt_is_accepted() {
        assert!(matches!(
            FenixOperationRequest::build(&TestPolicy { allow: false }, &actor(), &scope(), parts()).await,
            Err(FenixClientError::Denied)
        ));
        assert!(matches!(
            FenixBearer::from_secret("human.jwt.value".into()),
            Err(FenixClientError::Configuration)
        ));
        let secret = format!("{SERVICE_TOKEN_PREFIX}{}", "A".repeat(43));
        let bearer = FenixBearer::from_secret(secret.clone()).unwrap();
        assert!(!format!("{bearer:?}").contains(&secret));
    }

    #[tokio::test]
    async fn builder_is_bounded_and_capability_neutral() {
        let req = request().await;
        let raw = req.bounded_json().unwrap();
        let value: Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(value["source"]["org_id"], scope().org_id.to_string());
        assert_eq!(value["source"]["product"], "dubbridge");
        assert_eq!(value["contract_version"], CONTRACT_VERSION);
        assert!(value.get("workflow_id").is_none());

        let mut invalid = parts();
        invalid.capability_key = "review".into();
        assert!(matches!(
            FenixOperationRequest::build(&TestPolicy { allow: true }, &actor(), &scope(), invalid).await,
            Err(FenixClientError::InvalidRequest)
        ));
        let mut too_large = parts();
        too_large.input = serde_json::json!({"payload":"x".repeat(MAX_REQUEST_BYTES)});
        assert!(FenixOperationRequest::build(
            &TestPolicy { allow: true },
            &actor(),
            &scope(),
            too_large
        ).await.unwrap().bounded_json().is_err());
    }

    #[tokio::test]
    async fn fake_http_responses_never_manufacture_an_operation() {
        let req = request().await;
        let good = serde_json::to_vec(&serde_json::json!({
            "contract_version": CONTRACT_VERSION,
            "operation_id": req.operation_id(),
            "trace_id": req.trace_id(),
            "replayed": false,
        })).unwrap();
        let ack = parse_admission_reply(
            FenixTransportReply { status: 202, body: good },
            &req
        ).unwrap();
        assert_eq!(ack.operation_id, req.operation_id());
        for code in [500, 502, 503, 504, 302, 429] {
            assert_eq!(
                parse_admission_reply(
                    FenixTransportReply { status: code, body: Vec::new() },
                    &req
                ),
                Err(FenixClientError::AmbiguousOutcome)
            );
        }
        assert_eq!(
            FenixClientError::AmbiguousOutcome.retry_advice(),
            RetryAdvice::ReconcileBeforeAnyRetry
        );
        assert_eq!(
            FenixClientError::Conflict.retry_advice(),
            RetryAdvice::Never
        );
    }

    #[derive(Default)]
    struct FakeTransport {
        sent: Arc<Mutex<Vec<String>>>,
    }

    impl FenixTransport for FakeTransport {
        fn post_operation<'a>(
            &'a self,
            operation_id: &'a str,
            _bearer: &'a FenixBearer,
            body: Vec<u8>,
        ) -> impl Future<Output = Result<FenixTransportReply, FenixClientError>> + Send + 'a {
            async move {
                self.sent.lock().unwrap().push(operation_id.to_string());
                let request: Value = serde_json::from_slice(&body)
                    .map_err(|_| FenixClientError::InvalidRequest)?;
                Ok(FenixTransportReply {
                    status: 202,
                    body: serde_json::to_vec(&serde_json::json!({
                        "contract_version": "1",
                        "operation_id": request["operation_id"],
                        "trace_id": request["trace_id"],
                        "replayed": false,
                    }))
                    .map_err(|_| FenixClientError::InvalidResponse)?,
                })
            }
        }
    }

    struct TestCredential;

    impl FenixCredentialProvider for TestCredential {
        fn service_token(&self) -> Result<FenixBearer, FenixClientError> {
            FenixBearer::from_secret(format!("{SERVICE_TOKEN_PREFIX}{}", "A".repeat(43)))
        }
    }

    #[tokio::test]
    async fn submit_once_uses_stable_key_without_status_polling() {
        let transport = FakeTransport::default();
        let sent = Arc::clone(&transport.sent);
        let client = FenixClient::new(transport, TestCredential);
        let req = request().await;
        let ack = client.submit_once(&req).await.unwrap();
        assert_eq!(ack.operation_id, req.operation_id());
        assert_eq!(sent.lock().unwrap().as_slice(), &["operation_00000001"]);
    }

    #[test]
    fn https_transport_requires_opt_in_and_denies_redirect_risks() {
        for url in [
            "http://fenix.example/",
            "https://user:secret@fenix.example/",
            "https://fenix.example/?token=foo",
            "https://fenix.example/surprising-path",
        ] {
            assert!(matches!(
                FenixHttpsTransport::opt_in(url, Duration::from_secs(5)),
                Err(FenixClientError::Configuration)
            ));
        }
        assert!(FenixHttpsTransport::opt_in("https://fenix.example/", Duration::from_secs(5)).is_ok());
    }
}
