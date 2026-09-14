// MVP0-P2P P2.T4c: backend mTLS client for availability-publication-v1.
//
// This module owns only the private publication-control transport contract.
// PostgreSQL state, retries, reconciliation, and readiness remain outside this
// connector so a successful HTTP exchange can never become authority by itself.

use std::time::Duration;

use dubbridge_domain::p2p_publication::{K1LineageId, P2pPublicationId};
use reqwest::{Certificate, Client, Identity, StatusCode, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTRACT_VERSION: &str = "availability-publication-v1";
pub const MANIFEST_VERSION: &str = "p2p-manifest-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailabilityPublicationRequest {
    pub contract_version: &'static str,
    pub publication_id: String,
    pub lineage_id: String,
    pub manifest_version: &'static str,
    pub manifest_digest_sha256: String,
    pub package_ref: String,
}

impl AvailabilityPublicationRequest {
    pub fn new(
        publication_id: P2pPublicationId,
        lineage_id: K1LineageId,
        manifest_digest_sha256: impl Into<String>,
        package_ref: impl Into<String>,
    ) -> Result<Self, AvailabilityPublicationError> {
        let request = Self {
            contract_version: CONTRACT_VERSION,
            publication_id: publication_id.to_string(),
            lineage_id: lineage_id.to_string(),
            manifest_version: MANIFEST_VERSION,
            manifest_digest_sha256: manifest_digest_sha256.into(),
            package_ref: package_ref.into(),
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), AvailabilityPublicationError> {
        if !is_lower_hex_sha256(&self.manifest_digest_sha256)
            || !is_valid_package_ref(&self.package_ref)
        {
            return Err(AvailabilityPublicationError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvailabilityPublicationEvidence {
    pub contract_version: String,
    pub publication_id: String,
    pub lineage_id: String,
    pub manifest_digest_sha256: String,
    pub external_publication_id: String,
    pub evidence_id: String,
    pub confirmed_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AvailabilityErrorBody {
    contract_version: String,
    code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AvailabilityPublicationError {
    #[error("invalid Availability Node client configuration")]
    InvalidConfiguration,
    #[error("invalid availability-publication-v1 request")]
    InvalidRequest,
    #[error("Availability Node service identity was rejected")]
    Unauthorized,
    #[error("Availability Node reported a publication conflict")]
    PublicationConflict,
    #[error("Availability Node rejected the ciphertext package")]
    PackageInvalid,
    #[error("Availability Node is temporarily unavailable")]
    Unavailable,
    #[error("publication transport outcome is ambiguous")]
    AmbiguousOutcome,
    #[error("Availability Node returned an invalid contract response")]
    InvalidResponse,
}

pub struct AvailabilityPublicationClient {
    http: Client,
    base_url: String,
}

impl std::fmt::Debug for AvailabilityPublicationClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvailabilityPublicationClient")
            .field("base_url", &self.base_url)
            .field("http", &"[configured mTLS client]")
            .finish()
    }
}

impl AvailabilityPublicationClient {
    pub fn from_mtls_pem(
        base_url: &str,
        ca_pem: &[u8],
        client_identity_pem: &[u8],
        timeout: Duration,
    ) -> Result<Self, AvailabilityPublicationError> {
        if timeout.is_zero() {
            return Err(AvailabilityPublicationError::InvalidConfiguration);
        }

        let parsed = Url::parse(base_url)
            .map_err(|_| AvailabilityPublicationError::InvalidConfiguration)?;
        if parsed.scheme() != "https"
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(AvailabilityPublicationError::InvalidConfiguration);
        }

        let ca = Certificate::from_pem(ca_pem)
            .map_err(|_| AvailabilityPublicationError::InvalidConfiguration)?;
        let identity = Identity::from_pem(client_identity_pem)
            .map_err(|_| AvailabilityPublicationError::InvalidConfiguration)?;
        let http = Client::builder()
            .https_only(true)
            .add_root_certificate(ca)
            .identity(identity)
            .timeout(timeout)
            .build()
            .map_err(|_| AvailabilityPublicationError::InvalidConfiguration)?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    pub async fn publish(
        &self,
        request: &AvailabilityPublicationRequest,
    ) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError> {
        request.validate()?;
        let body = serde_json::to_vec(request)
            .map_err(|_| AvailabilityPublicationError::InvalidRequest)?;
        let url = format!(
            "{}/v1/publications/{}",
            self.base_url, request.publication_id
        );

        let response = self
            .http
            .put(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| AvailabilityPublicationError::AmbiguousOutcome)?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| AvailabilityPublicationError::AmbiguousOutcome)?;

        parse_response(status, &bytes, request)
    }
}

fn parse_response(
    status: StatusCode,
    body: &[u8],
    request: &AvailabilityPublicationRequest,
) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError> {
    if matches!(status, StatusCode::OK | StatusCode::CREATED) {
        let evidence: AvailabilityPublicationEvidence = serde_json::from_slice(body)
            .map_err(|_| AvailabilityPublicationError::InvalidResponse)?;
        validate_evidence(&evidence, request)?;
        return Ok(evidence);
    }

    let error: AvailabilityErrorBody = serde_json::from_slice(body)
        .map_err(|_| AvailabilityPublicationError::InvalidResponse)?;
    if error.contract_version != CONTRACT_VERSION {
        return Err(AvailabilityPublicationError::InvalidResponse);
    }

    match (status, error.code.as_str()) {
        (StatusCode::BAD_REQUEST, "invalid_contract") => {
            Err(AvailabilityPublicationError::InvalidResponse)
        }
        (StatusCode::FORBIDDEN, "service_identity_rejected") => {
            Err(AvailabilityPublicationError::Unauthorized)
        }
        (StatusCode::CONFLICT, "publication_conflict") => {
            Err(AvailabilityPublicationError::PublicationConflict)
        }
        (StatusCode::UNPROCESSABLE_ENTITY, "package_invalid") => {
            Err(AvailabilityPublicationError::PackageInvalid)
        }
        (StatusCode::SERVICE_UNAVAILABLE, "publication_unavailable") => {
            Err(AvailabilityPublicationError::Unavailable)
        }
        _ => Err(AvailabilityPublicationError::InvalidResponse),
    }
}

fn validate_evidence(
    evidence: &AvailabilityPublicationEvidence,
    request: &AvailabilityPublicationRequest,
) -> Result<(), AvailabilityPublicationError> {
    if evidence.contract_version != CONTRACT_VERSION
        || evidence.publication_id != request.publication_id
        || evidence.lineage_id != request.lineage_id
        || evidence.manifest_digest_sha256 != request.manifest_digest_sha256
        || evidence.external_publication_id.trim().is_empty()
        || evidence.evidence_id.trim().is_empty()
        || evidence.confirmed_at.trim().is_empty()
    {
        return Err(AvailabilityPublicationError::InvalidResponse);
    }
    Ok(())
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn is_valid_package_ref(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.contains('\0')
        || value.contains("//")
    {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return false;
    }
    value
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> AvailabilityPublicationRequest {
        AvailabilityPublicationRequest::new(
            P2pPublicationId(uuid::Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap()),
            K1LineageId(uuid::Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap()),
            "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4",
            "packages/22222222-2222-4222-8222-222222222222/33333333-3333-4333-8333-333333333333",
        )
        .unwrap()
    }

    #[test]
    fn request_rejects_traversal_and_non_hex_digest() {
        let publication_id = P2pPublicationId::new();
        let lineage_id = K1LineageId::new();
        assert!(matches!(
            AvailabilityPublicationRequest::new(
                publication_id,
                lineage_id,
                "z".repeat(64),
                "../outside",
            ),
            Err(AvailabilityPublicationError::InvalidRequest)
        ));
    }

    #[test]
    fn success_response_must_match_requested_identity_and_digest() {
        let request = request();
        let valid = serde_json::json!({
            "contract_version": CONTRACT_VERSION,
            "publication_id": request.publication_id,
            "lineage_id": request.lineage_id,
            "manifest_digest_sha256": request.manifest_digest_sha256,
            "external_publication_id": "hyperdrive:stable-key",
            "evidence_id": "evidence-1",
            "confirmed_at": "2026-09-14T06:00:00Z"
        });
        assert!(parse_response(StatusCode::CREATED, &serde_json::to_vec(&valid).unwrap(), &request).is_ok());

        let mismatched = serde_json::json!({
            "contract_version": CONTRACT_VERSION,
            "publication_id": request.publication_id,
            "lineage_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
            "manifest_digest_sha256": request.manifest_digest_sha256,
            "external_publication_id": "hyperdrive:stable-key",
            "evidence_id": "evidence-1",
            "confirmed_at": "2026-09-14T06:00:00Z"
        });
        assert!(matches!(
            parse_response(StatusCode::OK, &serde_json::to_vec(&mismatched).unwrap(), &request),
            Err(AvailabilityPublicationError::InvalidResponse)
        ));
    }

    #[test]
    fn remote_error_contract_is_strictly_mapped() {
        let request = request();
        let unavailable = serde_json::json!({
            "contract_version": CONTRACT_VERSION,
            "code": "publication_unavailable"
        });
        assert!(matches!(
            parse_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &serde_json::to_vec(&unavailable).unwrap(),
                &request,
            ),
            Err(AvailabilityPublicationError::Unavailable)
        ));

        let wrong_status = serde_json::json!({
            "contract_version": CONTRACT_VERSION,
            "code": "publication_conflict"
        });
        assert!(matches!(
            parse_response(
                StatusCode::SERVICE_UNAVAILABLE,
                &serde_json::to_vec(&wrong_status).unwrap(),
                &request,
            ),
            Err(AvailabilityPublicationError::InvalidResponse)
        ));
    }
}
