//! Inert, opt-in DubBridge -> Fenix service-to-service client.
//! This module is not wired into any DubBridge route or background worker.
//! The Fenix provider admission HTTP route has not been activated or approved.

use std::fmt;
use std::future::Future; // RPITIT public transport/authorization trait signatures retain explicit Send
use std::time::Duration;

use dubbridge_domain::fenix_operation_identity::derive_fenix_operation_identity;
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
// Reserve headroom for the B1 envelope/source nodes counted by Fenix.
const MAX_INPUT_NODES: usize = 4080;
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
        // Fenix's ParseServiceToken requires *canonical* unpadded base64url
        // of 32 bytes. The final sextet has two padding bits that MUST be zero.
        // Character-class validation alone accepts tokens Fenix will reject.
        if encoded.len() != 43
            || !encoded
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'-' | b'_'))
            || !encoded
                .bytes()
                .last()
                .is_some_and(|ch| b"AEIMQUYcgkosw048".contains(&ch))
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
#[derive(Clone, Copy)]
pub struct DubBridgeActor {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
}

/// Requested local scope is not an authorization grant.
#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Callers cannot invent a fresh operation ID on each retry. Construct
    /// exclusively from a durable DubBridge work-item via the public factory.
    operation_id: String,
    trace_id: String,
    capability_key: String,
    input_schema_version: String,
    input: Value,
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
        let (operation_id, trace_id) = derive_fenix_operation_identity(work_item_id, &capability_key);
        Ok(Self {
            operation_id: operation_id.to_string(),
            trace_id: trace_id.to_string(),
            capability_key,
            input_schema_version,
            input,
        })
    }
}

#[derive(Clone, Serialize, PartialEq)]
struct FenixSource {
    product: &'static str,
    org_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<Uuid>,
}

#[derive(Clone, Serialize, PartialEq)]
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
        let mut visited = 0;
        if fields.len() > 64 || !valid_input_tree(&parts.input, 1, &mut visited) {
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

    /// Exact, deterministic JSON bytes sent to Fenix. An intent ledger
    /// must fingerprint this result before its single send claim.
    /// Do not independently reserialize the request in an application adapter.
    pub fn outbound_json_bytes(&self) -> Result<Vec<u8>, FenixClientError> {
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
        && value
            .as_bytes()
            .first()
            .is_some_and(|ch| (b'1'..=b'9').contains(ch))
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
    async fn post_operation(
        &self,
        operation_id: &str,
        bearer: &FenixBearer,
        body: Vec<u8>,
    ) -> Result<FenixTransportReply, FenixClientError> {
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FenixAdmissionFailure {
    contract_version: String,
    error_code: String,
    message: String,
}

const SAFE_ADMISSION_MESSAGE: &str = "Request could not be accepted";


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
        let body = request.outbound_json_bytes()?;
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
                || ack.replayed != (reply.status == 200)
            {
                return Err(FenixClientError::InvalidResponse);
            }
            Ok(ack)
        }
        400 | 401 | 403 | 404 | 409 | 503 => parse_admission_denial(reply.status, &reply.body),
        _ => Err(FenixClientError::AmbiguousOutcome),
    }
}

fn parse_admission_denial(
    status: u16,
    body: &[u8],
) -> Result<FenixAdmissionAck, FenixClientError> {
    let error: FenixAdmissionFailure = serde_json::from_slice(body)
        .map_err(|_| FenixClientError::AmbiguousOutcome)?;
    if error.contract_version != CONTRACT_VERSION || error.message != SAFE_ADMISSION_MESSAGE {
        return Err(FenixClientError::AmbiguousOutcome);
    }
    match (status, error.error_code.as_str()) {
        (400, "INVALID_REQUEST") => Err(FenixClientError::InvalidRequest),
        (401 | 403 | 404, "REQUEST_DENIED") => Err(FenixClientError::Denied),
        (409, "OPERATION_CONFLICT") => Err(FenixClientError::Conflict),
        (503, "PROVIDER_UNAVAILABLE") => Err(FenixClientError::AmbiguousOutcome),
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
        async fn authorize_scope(
            &self,
            _actor: &DubBridgeActor,
            requested: &TenantSelection,
        ) -> Result<TenantSelection, FenixClientError> {
            if self.allow {
                Ok(*requested)
            } else {
                Err(FenixClientError::Denied)
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


    struct WrongScopePolicy;

    impl FenixScopeAuthorizer for WrongScopePolicy {
        async fn authorize_scope(
            &self,
            _actor: &DubBridgeActor,
            requested: &TenantSelection,
        ) -> Result<TenantSelection, FenixClientError> {
            Ok(TenantSelection {
                org_id: Uuid::nil(),
                project_id: requested.project_id,
            })
        }
    }

    #[tokio::test]
    async fn a_policy_result_for_another_org_cannot_override_the_request() {
        let result = FenixOperationRequest::build(
            &WrongScopePolicy, &actor(), &scope(), parts(),
        ).await;
        assert!(matches!(result, Err(FenixClientError::Denied)));
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
        // Same length/alphabet but nonzero pad bits: Fenix Go rejects these.
        let noncanonical = format!("{SERVICE_TOKEN_PREFIX}{}B", "A".repeat(42));
        assert!(matches!(
            FenixBearer::from_secret(noncanonical),
            Err(FenixClientError::Configuration)
        ));
        let canonical_tail = format!("{SERVICE_TOKEN_PREFIX}{}E", "A".repeat(42));
        assert!(FenixBearer::from_secret(canonical_tail).is_ok());
    }

    #[tokio::test]
    async fn builder_is_bounded_and_capability_neutral() {
        let req = request().await;
        let raw = req.outbound_json_bytes().unwrap();
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
        ).await.unwrap().outbound_json_bytes().is_err());
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

    #[tokio::test]
    async fn replay_flag_and_http_status_must_agree() {
        let request = request().await;
        for (status, replayed, expected_ok) in [
            (202, false, true),
            (200, true, true),
            (202, true, false),
            (200, false, false),
        ] {
            let body = serde_json::to_vec(&serde_json::json!({
                "contract_version": CONTRACT_VERSION,
                "operation_id": request.operation_id(),
                "trace_id": request.trace_id(),
                "replayed": replayed,
            })).unwrap();
            let outcome = parse_admission_reply(
                FenixTransportReply { status, body }, &request,
            );
            assert_eq!(outcome.is_ok(), expected_ok);
        }
    }

    #[tokio::test]
    async fn proposed_denial_contract_requires_status_and_allowlisted_body() {
        let request = request().await;
        for (status, code, expected) in [
            (400, "INVALID_REQUEST", FenixClientError::InvalidRequest),
            (401, "REQUEST_DENIED", FenixClientError::Denied),
            (403, "REQUEST_DENIED", FenixClientError::Denied),
            (404, "REQUEST_DENIED", FenixClientError::Denied),
            (409, "OPERATION_CONFLICT", FenixClientError::Conflict),
            (503, "PROVIDER_UNAVAILABLE", FenixClientError::AmbiguousOutcome),
        ] {
            let body = serde_json::json!({
                "contract_version": "1",
                "error_code": code,
                "message": SAFE_ADMISSION_MESSAGE,
            });
            let result = parse_admission_reply(FenixTransportReply {
                status,
                body: serde_json::to_vec(&body).unwrap(),
            }, &request);
            assert_eq!(result, Err(expected));
        }
        for (status, code) in [
            (409, "REQUEST_DENIED"),
            (403, "OPERATION_CONFLICT"),
            (422, "REQUEST_DENIED"),
            (503, "REQUEST_DENIED"),
        ] {
            let result = parse_admission_reply(FenixTransportReply {
                status,
                body: serde_json::to_vec(&serde_json::json!({
                    "contract_version": "1",
                    "error_code": code,
                    "message": SAFE_ADMISSION_MESSAGE,
                })).unwrap(),
            }, &request);
            assert_eq!(result, Err(FenixClientError::AmbiguousOutcome));
        }
        for body in [
            serde_json::json!({"contract_version":"1","error_code":"REQUEST_DENIED",
                "message":"internal SQL credentials"}),
            serde_json::json!({"contract_version":"2","error_code":"REQUEST_DENIED",
                "message":SAFE_ADMISSION_MESSAGE}),
            serde_json::json!({"contract_version":"1","error_code":"REQUEST_DENIED",
                "message":SAFE_ADMISSION_MESSAGE,"debug":"leak"}),
        ] {
            assert_eq!(parse_admission_reply(FenixTransportReply{
                status:403,body:serde_json::to_vec(&body).unwrap(),
            },&request),Err(FenixClientError::AmbiguousOutcome));
        }
    }

    #[tokio::test]
    async fn mismatched_ack_and_extra_fields_do_not_become_success() {
        let req = request().await;
        let wrong = serde_json::json!({
            "contract_version": "1",
            "operation_id": "forged_0000000001",
            "trace_id": req.trace_id(),
            "replayed": false,
        });
        assert_eq!(
            parse_admission_reply(
                FenixTransportReply {
                    status: 202,
                    body: serde_json::to_vec(&wrong).unwrap(),
                },
                &req,
            ),
            Err(FenixClientError::InvalidResponse)
        );
        let mut extra = wrong;
        extra["operation_id"] = serde_json::json!(req.operation_id());
        extra["run_id"] = serde_json::json!("fake-run");
        assert_eq!(
            parse_admission_reply(
                FenixTransportReply {
                    status: 202,
                    body: serde_json::to_vec(&extra).unwrap(),
                },
                &req,
            ),
            Err(FenixClientError::InvalidResponse)
        );
    }

    #[derive(Default)]
    struct FakeTransport {
        sent: Arc<Mutex<Vec<String>>>,
    }

    impl FenixTransport for FakeTransport {
        async fn post_operation(
            &self,
            operation_id: &str,
            _bearer: &FenixBearer,
            body: Vec<u8>,
        ) -> Result<FenixTransportReply, FenixClientError> {
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

    struct AmbiguousTransport {
        calls: Arc<Mutex<usize>>,
    }

    impl FenixTransport for AmbiguousTransport {
        async fn post_operation(
            &self,
            _operation_id: &str,
            _bearer: &FenixBearer,
            _body: Vec<u8>,
        ) -> Result<FenixTransportReply, FenixClientError> {
            *self.calls.lock().unwrap() += 1;
            Err(FenixClientError::AmbiguousOutcome)
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

    #[tokio::test]
    async fn ambiguous_network_failure_is_never_retried_by_client() {
        let calls = Arc::new(Mutex::new(0));
        let client = FenixClient::new(AmbiguousTransport {
            calls: Arc::clone(&calls),
        }, TestCredential);
        let req = request().await;
        assert_eq!(client.submit_once(&req).await, Err(FenixClientError::AmbiguousOutcome));
        assert_eq!(*calls.lock().unwrap(), 1);
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
