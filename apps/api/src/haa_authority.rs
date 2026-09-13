use std::{collections::HashMap, sync::Arc};

use dubbridge_db::{
    error::DbError,
    haa_read_repo::{self, StoredAuthenticator},
    haa_repo::{self, NewApprovalRequest, NewChallenge, NewExecutionGrant, VerifiedApproval},
};
use dubbridge_domain::haa::{
    ActionSpec, ApprovalDecision, ApprovalEvidence, ApprovalIntent, ApprovalReceipt, CanonicalValue,
    ExecutionGrant, PrincipalRef, SignatureEnvelope, VerifiedEvidence, HAA_PROTOCOL_V1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::haa_service::{
    build_challenge_package, canonical_json, challenge_digest, digest_intent, ChallengeSigner,
    HaaServiceError,
};

const RECEIPT_SIGNING_DOMAIN: &[u8] = b"dubbridge.haa.receipt.v1\0";
const GRANT_SIGNING_DOMAIN: &[u8] = b"dubbridge.haa.execution-grant.v1\0";
const DEFAULT_GRANT_TTL: Duration = Duration::seconds(60);

pub trait EvidenceVerifier: Send + Sync {
    fn authenticator_kind(&self) -> &'static str;

    fn verify(
        &self,
        evidence: &ApprovalEvidence,
        authenticator: &StoredAuthenticator,
        expected_challenge_digest: &str,
        now: OffsetDateTime,
    ) -> Result<VerifiedEvidence, HaaAuthorityError>;
}

#[derive(Debug, thiserror::Error)]
pub enum HaaAuthorityError {
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Protocol(#[from] HaaServiceError),
    #[error("unsupported authenticator kind: {0}")]
    UnsupportedAuthenticator(String),
    #[error("authenticator is inactive")]
    InactiveAuthenticator,
    #[error("approval evidence does not match request or challenge")]
    EvidenceMismatch,
    #[error("approval challenge expired or was already satisfied")]
    ChallengeUnavailable,
    #[error("approval request expired")]
    RequestExpired,
    #[error("stored HAA data is invalid: {0}")]
    StoredData(String),
}

#[derive(Debug, Clone)]
pub struct CreatedApproval {
    pub request_id: Uuid,
    pub package: dubbridge_domain::haa::ApprovalChallengePackage,
}

pub struct ApprovalAuthority {
    pool: PgPool,
    signer: Arc<dyn ChallengeSigner>,
    verifiers: HashMap<String, Arc<dyn EvidenceVerifier>>,
}

impl ApprovalAuthority {
    #[must_use]
    pub fn new(pool: PgPool, signer: Arc<dyn ChallengeSigner>) -> Self {
        Self {
            pool,
            signer,
            verifiers: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_verifier(mut self, verifier: Arc<dyn EvidenceVerifier>) -> Self {
        self.verifiers
            .insert(verifier.authenticator_kind().to_owned(), verifier);
        self
    }

    pub async fn create_request(
        &self,
        intent: ApprovalIntent,
        approver: PrincipalRef,
        now: OffsetDateTime,
        challenge_expires_at: OffsetDateTime,
    ) -> Result<CreatedApproval, HaaAuthorityError> {
        if intent.expires_at <= now {
            return Err(HaaAuthorityError::RequestExpired);
        }
        let digested = digest_intent(&intent)?;
        let request_id = Uuid::new_v4();
        let package = build_challenge_package(
            self.signer.as_ref(),
            request_id,
            &intent,
            &digested,
            approver.clone(),
            now,
            challenge_expires_at,
        )?;
        let package_json = canonical_json(&package)?;
        let challenge_hash = challenge_digest(&package.challenge)?;

        haa_repo::insert_request(
            &self.pool,
            &NewApprovalRequest {
                id: request_id,
                protocol_version: HAA_PROTOCOL_V1,
                action_schema: &intent.action.schema,
                action_type: &intent.action.action_type,
                resource: &intent.action.resource,
                environment: intent.action.environment.as_deref(),
                canonical_action: &digested.canonical_action,
                canonical_intent: &digested.canonical_intent,
                action_digest: &digested.action_digest,
                intent_digest: &digested.intent_digest,
                requester_kind: &intent.requester.kind,
                requester_id: &intent.requester.id,
                audience_kind: &intent.audience.kind,
                audience_id: &intent.audience.id,
                approver_kind: &approver.kind,
                approver_id: &approver.id,
                policy_id: &intent.policy.id,
                policy_version: &intent.policy.version,
                created_at: now,
                expires_at: intent.expires_at,
            },
        )
        .await?;

        haa_repo::insert_challenge(
            &self.pool,
            &NewChallenge {
                request_id,
                nonce: &package.challenge.nonce,
                challenge_digest: &challenge_hash,
                package_json: &package_json,
                issued_at: now,
                expires_at: challenge_expires_at,
            },
        )
        .await?;

        Ok(CreatedApproval {
            request_id,
            package,
        })
    }

    pub async fn submit_evidence(
        &self,
        evidence: ApprovalEvidence,
        now: OffsetDateTime,
    ) -> Result<ApprovalReceipt, HaaAuthorityError> {
        if evidence.protocol_version != HAA_PROTOCOL_V1 {
            return Err(HaaAuthorityError::EvidenceMismatch);
        }
        let request = haa_repo::get_request(&self.pool, evidence.request_id).await?;
        if request.expires_at <= now {
            return Err(HaaAuthorityError::RequestExpired);
        }
        let challenge = haa_read_repo::get_challenge(&self.pool, evidence.request_id).await?;
        if challenge.expires_at <= now || challenge.satisfied_at.is_some() {
            return Err(HaaAuthorityError::ChallengeUnavailable);
        }
        if evidence.challenge_digest != challenge.challenge_digest {
            return Err(HaaAuthorityError::EvidenceMismatch);
        }

        let authenticator =
            haa_read_repo::get_authenticator(&self.pool, &evidence.authenticator_id).await?;
        if !authenticator.active {
            return Err(HaaAuthorityError::InactiveAuthenticator);
        }
        let verifier = self
            .verifiers
            .get(&authenticator.kind)
            .ok_or_else(|| HaaAuthorityError::UnsupportedAuthenticator(authenticator.kind.clone()))?;
        let verified = verifier.verify(
            &evidence,
            &authenticator,
            &challenge.challenge_digest,
            now,
        )?;

        if verified.request_id != request.id
            || verified.authenticator_id != authenticator.id
            || verified.approver.kind != request.approver_kind
            || verified.approver.id != request.approver_id
        {
            return Err(HaaAuthorityError::EvidenceMismatch);
        }

        let receipt = self.build_receipt(&request, &verified, now)?;
        let evidence_json = canonical_json(&evidence)?;
        let receipt_json = canonical_json(&receipt)?;
        haa_repo::approve(
            &self.pool,
            &VerifiedApproval {
                evidence_id: Uuid::new_v4(),
                request_id: request.id,
                authenticator_id: &verified.authenticator_id,
                challenge_digest: &verified.challenge_digest,
                evidence_kind: &evidence.kind,
                evidence_json: &evidence_json,
                receipt_id: receipt.receipt_id,
                receipt_json: &receipt_json,
                verified_at: verified.verified_at,
                receipt_expires_at: receipt.expires_at,
            },
        )
        .await?;
        Ok(receipt)
    }

    pub async fn reject(
        &self,
        request_id: Uuid,
        at: OffsetDateTime,
    ) -> Result<(), HaaAuthorityError> {
        haa_repo::reject(&self.pool, request_id, at).await?;
        Ok(())
    }

    pub async fn authorize_and_consume(
        &self,
        request_id: Uuid,
        execution_id: Uuid,
        actual_action: &ActionSpec,
        audience: &PrincipalRef,
        now: OffsetDateTime,
    ) -> Result<ExecutionGrant, HaaAuthorityError> {
        let request = haa_repo::get_request(&self.pool, request_id).await?;
        let actual_intent = ApprovalIntent {
            action: actual_action.clone(),
            requester: PrincipalRef {
                kind: request.requester_kind.clone(),
                id: request.requester_id.clone(),
            },
            audience: PrincipalRef {
                kind: request.audience_kind.clone(),
                id: request.audience_id.clone(),
            },
            policy: dubbridge_domain::haa::PolicyRef {
                id: request.policy_id.clone(),
                version: request.policy_version.clone(),
            },
            created_at: request.created_at,
            expires_at: request.expires_at,
        };
        let actual_digest = digest_intent(&actual_intent)?.action_digest;
        let expires_at = std::cmp::min(now + DEFAULT_GRANT_TTL, request.expires_at);
        let mut grant = ExecutionGrant {
            protocol_version: HAA_PROTOCOL_V1.to_owned(),
            grant_id: Uuid::new_v4(),
            request_id,
            execution_id,
            action_digest: actual_digest,
            audience: audience.clone(),
            issued_at: now,
            expires_at,
            haa_signature: unsigned_signature(),
        };
        grant.haa_signature = self.sign_grant(&grant)?;
        let grant_json = canonical_json(&grant)?;

        let result = haa_repo::authorize_and_consume(
            &self.pool,
            &NewExecutionGrant {
                execution_id,
                request_id,
                action_digest: &grant.action_digest,
                audience_kind: &audience.kind,
                audience_id: &audience.id,
                grant_json: &grant_json,
                issued_at: now,
                expires_at,
            },
            now,
        )
        .await
        .map_err(|error| HaaAuthorityError::StoredData(error.to_string()))?;

        match result {
            haa_repo::ConsumeResult::Consumed { .. } => Ok(grant),
            haa_repo::ConsumeResult::Idempotent { grant_json } => serde_json::from_str(&grant_json)
                .map_err(|error| HaaAuthorityError::StoredData(error.to_string())),
        }
    }

    fn build_receipt(
        &self,
        request: &haa_repo::ApprovalRequestRecord,
        verified: &VerifiedEvidence,
        now: OffsetDateTime,
    ) -> Result<ApprovalReceipt, HaaAuthorityError> {
        let mut receipt = ApprovalReceipt {
            protocol_version: HAA_PROTOCOL_V1.to_owned(),
            receipt_id: Uuid::new_v4(),
            request_id: request.id,
            action_digest: request.action_digest.clone(),
            intent_digest: request.intent_digest.clone(),
            decision: ApprovalDecision::Approved,
            approver: verified.approver.clone(),
            authenticator_id: verified.authenticator_id.clone(),
            issued_at: now,
            expires_at: request.expires_at,
            haa_signature: unsigned_signature(),
        };
        receipt.haa_signature = self.sign_receipt(&receipt)?;
        Ok(receipt)
    }

    fn sign_receipt(
        &self,
        receipt: &ApprovalReceipt,
    ) -> Result<SignatureEnvelope, HaaAuthorityError> {
        let view = ReceiptSigningView::from(receipt);
        let digest = signed_payload_digest(RECEIPT_SIGNING_DOMAIN, &view)?;
        Ok(self.signer.sign_digest(&digest)?)
    }

    fn sign_grant(
        &self,
        grant: &ExecutionGrant,
    ) -> Result<SignatureEnvelope, HaaAuthorityError> {
        let view = GrantSigningView::from(grant);
        let digest = signed_payload_digest(GRANT_SIGNING_DOMAIN, &view)?;
        Ok(self.signer.sign_digest(&digest)?)
    }
}

#[derive(Serialize)]
struct ReceiptSigningView<'a> {
    protocol_version: &'a str,
    receipt_id: Uuid,
    request_id: Uuid,
    action_digest: &'a str,
    intent_digest: &'a str,
    decision: ApprovalDecision,
    approver: &'a PrincipalRef,
    authenticator_id: &'a str,
    issued_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl<'a> From<&'a ApprovalReceipt> for ReceiptSigningView<'a> {
    fn from(value: &'a ApprovalReceipt) -> Self {
        Self {
            protocol_version: &value.protocol_version,
            receipt_id: value.receipt_id,
            request_id: value.request_id,
            action_digest: &value.action_digest,
            intent_digest: &value.intent_digest,
            decision: value.decision,
            approver: &value.approver,
            authenticator_id: &value.authenticator_id,
            issued_at: value.issued_at,
            expires_at: value.expires_at,
        }
    }
}

#[derive(Serialize)]
struct GrantSigningView<'a> {
    protocol_version: &'a str,
    grant_id: Uuid,
    request_id: Uuid,
    execution_id: Uuid,
    action_digest: &'a str,
    audience: &'a PrincipalRef,
    issued_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl<'a> From<&'a ExecutionGrant> for GrantSigningView<'a> {
    fn from(value: &'a ExecutionGrant) -> Self {
        Self {
            protocol_version: &value.protocol_version,
            grant_id: value.grant_id,
            request_id: value.request_id,
            execution_id: value.execution_id,
            action_digest: &value.action_digest,
            audience: &value.audience,
            issued_at: value.issued_at,
            expires_at: value.expires_at,
        }
    }
}

fn signed_payload_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, HaaAuthorityError> {
    let canonical = canonical_json(value)?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(canonical.as_bytes());
    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn unsigned_signature() -> SignatureEnvelope {
    SignatureEnvelope {
        algorithm: "pending".to_owned(),
        key_id: "pending".to_owned(),
        signature_b64: String::new(),
    }
}

#[cfg(test)]
pub(crate) struct DigestEchoVerifier;

#[cfg(test)]
impl EvidenceVerifier for DigestEchoVerifier {
    fn authenticator_kind(&self) -> &'static str {
        "test-digest-echo"
    }

    fn verify(
        &self,
        evidence: &ApprovalEvidence,
        authenticator: &StoredAuthenticator,
        expected_challenge_digest: &str,
        now: OffsetDateTime,
    ) -> Result<VerifiedEvidence, HaaAuthorityError> {
        if evidence.authenticator_id != authenticator.id
            || evidence.challenge_digest != expected_challenge_digest
            || evidence.payload != CanonicalValue::String(expected_challenge_digest.to_owned())
        {
            return Err(HaaAuthorityError::EvidenceMismatch);
        }
        Ok(VerifiedEvidence {
            request_id: evidence.request_id,
            approver: PrincipalRef {
                kind: authenticator.principal_kind.clone(),
                id: authenticator.principal_id.clone(),
            },
            authenticator_id: authenticator.id.clone(),
            challenge_digest: expected_challenge_digest.to_owned(),
            verified_at: now,
        })
    }
}
