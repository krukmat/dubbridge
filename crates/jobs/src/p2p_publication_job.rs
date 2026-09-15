// MVP0-P2P P2.T4d: PostgreSQL-authoritative publication dispatcher.
//
// Claims and retries are durable in PostgreSQL. The Availability Node provides
// external evidence only; a successful HTTP exchange never establishes Ready
// until same-lineage evidence is persisted and the guarded lifecycle transition
// succeeds.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use dubbridge_connectors::p2p_availability::{
    AvailabilityPublicationClient, AvailabilityPublicationError, AvailabilityPublicationEvidence,
    AvailabilityPublicationRequest,
};
use dubbridge_db::error::DbError;
use dubbridge_db::p2p_audit_transition_repo::enter_publication_reconciliation;
use dubbridge_db::p2p_publication_claim_repo::{
    P2pPublicationClaim, ReadyFinalization, claim_next_publication_work,
    fail_publication_claim as persist_failed_claim, finalize_publication_ready,
    release_publication_claim,
};
use dubbridge_db::p2p_publication_repo::{
    P2pPublicationRecord, get_publication, transition_publication_state,
};
use dubbridge_db::p2p_ready_repo::persist_confirmed_manifest_digest;
use dubbridge_domain::p2p_publication::{P2pPublicationId, PublicationState};
use dubbridge_domain::p2p_recovery::{
    DispatchAttempt, DispatchOutcome, RecoveryAction, decide_recovery_action,
};
use dubbridge_p2p::{
    manifest::{Manifest, canonical_json, manifest_sha256},
    package_writer::canonical_package_ref,
};
use sqlx::PgPool;
use thiserror::Error;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchTick {
    Idle,
    Ready(P2pPublicationId),
    Retrying(P2pPublicationId),
    Failed(P2pPublicationId),
}

#[derive(Debug, Error)]
pub enum P2pDispatchError {
    #[error("P2P publication database operation failed")]
    Database(#[from] DbError),
    #[error("P2P publication package is missing or invalid")]
    InvalidPackage,
    #[error("P2P publication configuration is invalid")]
    InvalidConfiguration,
}

#[async_trait]
pub trait AvailabilityPublisher: Send + Sync {
    async fn publish(
        &self,
        request: &AvailabilityPublicationRequest,
    ) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError>;
}

#[async_trait]
impl AvailabilityPublisher for AvailabilityPublicationClient {
    async fn publish(
        &self,
        request: &AvailabilityPublicationRequest,
    ) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError> {
        AvailabilityPublicationClient::publish(self, request).await
    }
}

pub struct P2pPublicationDispatcher {
    pool: PgPool,
    publisher: Arc<dyn AvailabilityPublisher>,
    ciphertext_root: PathBuf,
    lease_duration: Duration,
    retry_delay: Duration,
    max_attempts: u32,
}

impl P2pPublicationDispatcher {
    pub fn new(
        pool: PgPool,
        publisher: Arc<dyn AvailabilityPublisher>,
        ciphertext_root: PathBuf,
        lease_duration: Duration,
        retry_delay: Duration,
        max_attempts: u32,
    ) -> Result<Self, P2pDispatchError> {
        if ciphertext_root.as_os_str().is_empty()
            || lease_duration <= Duration::ZERO
            || retry_delay < Duration::ZERO
            || max_attempts == 0
        {
            return Err(P2pDispatchError::InvalidConfiguration);
        }
        Ok(Self {
            pool,
            publisher,
            ciphertext_root,
            lease_duration,
            retry_delay,
            max_attempts,
        })
    }

    pub async fn dispatch_once(&self) -> Result<DispatchTick, P2pDispatchError> {
        let now = OffsetDateTime::now_utc();
        let claim =
            claim_next_publication_work(&self.pool, Uuid::new_v4(), now + self.lease_duration)
                .await?;
        let Some(claim) = claim else {
            return Ok(DispatchTick::Idle);
        };

        self.dispatch_claim(claim).await
    }

    async fn dispatch_claim(
        &self,
        claim: P2pPublicationClaim,
    ) -> Result<DispatchTick, P2pDispatchError> {
        let publication = get_publication(&self.pool, claim.publication_id)
            .await?
            .ok_or(DbError::NotFound)?;
        if publication.lineage_id != claim.lineage_id {
            return Err(P2pDispatchError::Database(DbError::Conflict));
        }

        let publication = self.move_to_publishing(publication).await?;
        let request = match load_request(&self.ciphertext_root, &publication).await {
            Ok(request) => request,
            Err(_) => return self.fail_claim(claim, "package_invalid").await,
        };

        match self.publisher.publish(&request).await {
            Ok(evidence) => self.confirm_ready(claim, evidence).await,
            Err(error) if is_retryable_remote_error(error) => {
                self.retry_or_fail(claim, error).await
            }
            Err(error) => self.fail_claim(claim, remote_error_code(error)).await,
        }
    }

    async fn move_to_publishing(
        &self,
        publication: P2pPublicationRecord,
    ) -> Result<P2pPublicationRecord, P2pDispatchError> {
        match publication.state {
            PublicationState::PublishPending | PublicationState::Reconciling => {
                Ok(transition_publication_state(
                    &self.pool,
                    publication.id,
                    PublicationState::Publishing,
                    None,
                )
                .await?)
            }
            PublicationState::Publishing => Ok(publication),
            _ => Err(P2pDispatchError::Database(DbError::Conflict)),
        }
    }

    async fn confirm_ready(
        &self,
        claim: P2pPublicationClaim,
        evidence: AvailabilityPublicationEvidence,
    ) -> Result<DispatchTick, P2pDispatchError> {
        let confirmed_at = match OffsetDateTime::parse(&evidence.confirmed_at, &Rfc3339) {
            Ok(value) => value,
            Err(_) => return self.fail_claim(claim, "publication_response_invalid").await,
        };

        persist_confirmed_manifest_digest(
            &self.pool,
            claim.publication_id,
            claim.lineage_id,
            &evidence.manifest_digest_sha256,
        )
        .await?;

        finalize_publication_ready(
            &self.pool,
            ReadyFinalization {
                outbox_id: claim.outbox_id,
                publication_id: claim.publication_id,
                lineage_id: claim.lineage_id,
                claim_token: claim.claim_token,
                external_publication_id: &evidence.external_publication_id,
                confirmed_at,
                delivered_at: OffsetDateTime::now_utc(),
            },
        )
        .await?;
        Ok(DispatchTick::Ready(claim.publication_id))
    }

    async fn retry_or_fail(
        &self,
        claim: P2pPublicationClaim,
        error: AvailabilityPublicationError,
    ) -> Result<DispatchTick, P2pDispatchError> {
        let outcome = match error {
            AvailabilityPublicationError::AmbiguousOutcome => DispatchOutcome::Unknown,
            _ => DispatchOutcome::Failure,
        };
        let reason = remote_error_code(error);
        enter_publication_reconciliation(
            &self.pool,
            claim.publication_id,
            claim.lineage_id,
            reason,
        )
        .await?;

        let action = decide_recovery_action(
            PublicationState::Reconciling,
            claim.lineage_id,
            DispatchAttempt {
                lease_held: false,
                attempts: u32::try_from(claim.attempt_count).unwrap_or(u32::MAX),
                max_attempts: self.max_attempts,
                last_outcome: Some(outcome),
                confirmed_lineage: None,
            },
        );

        if action == RecoveryAction::MarkFailed {
            return self.fail_claim(claim, reason).await;
        }

        release_publication_claim(
            &self.pool,
            claim.outbox_id,
            claim.claim_token,
            OffsetDateTime::now_utc() + self.retry_delay,
            Some(reason),
        )
        .await?;
        Ok(DispatchTick::Retrying(claim.publication_id))
    }

    async fn fail_claim(
        &self,
        claim: P2pPublicationClaim,
        reason: &'static str,
    ) -> Result<DispatchTick, P2pDispatchError> {
        persist_failed_claim(
            &self.pool,
            claim.outbox_id,
            claim.publication_id,
            claim.lineage_id,
            claim.claim_token,
            OffsetDateTime::now_utc(),
            reason,
        )
        .await?;
        Ok(DispatchTick::Failed(claim.publication_id))
    }
}

async fn load_request(
    ciphertext_root: &Path,
    publication: &P2pPublicationRecord,
) -> Result<AvailabilityPublicationRequest, P2pDispatchError> {
    let package_ref = canonical_package_ref(
        &publication.id.to_string(),
        &publication.lineage_id.to_string(),
    );
    let manifest_path = ciphertext_root.join(&package_ref).join(MANIFEST_FILE_NAME);
    let bytes = tokio::fs::read(manifest_path)
        .await
        .map_err(|_| P2pDispatchError::InvalidPackage)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| P2pDispatchError::InvalidPackage)?;
    validate_manifest(&manifest, publication)?;

    let canonical = canonical_json(&manifest);
    if canonical.as_bytes() != bytes.as_slice() {
        return Err(P2pDispatchError::InvalidPackage);
    }
    let digest = manifest_sha256(&canonical);
    AvailabilityPublicationRequest::new(publication.id, publication.lineage_id, digest, package_ref)
        .map_err(|_| P2pDispatchError::InvalidPackage)
}

fn validate_manifest(
    manifest: &Manifest,
    publication: &P2pPublicationRecord,
) -> Result<(), P2pDispatchError> {
    if manifest.manifest_version != "p2p-manifest-v1"
        || manifest.publication_id != publication.id.to_string()
        || manifest.lineage_id != publication.lineage_id.to_string()
        || manifest.asset_id != publication.asset_id.0.to_string()
    {
        return Err(P2pDispatchError::InvalidPackage);
    }
    Ok(())
}

fn is_retryable_remote_error(error: AvailabilityPublicationError) -> bool {
    matches!(
        error,
        AvailabilityPublicationError::Unavailable | AvailabilityPublicationError::AmbiguousOutcome
    )
}

fn remote_error_code(error: AvailabilityPublicationError) -> &'static str {
    match error {
        AvailabilityPublicationError::InvalidConfiguration => "client_configuration_invalid",
        AvailabilityPublicationError::InvalidRequest => "request_invalid",
        AvailabilityPublicationError::Unauthorized => "service_identity_rejected",
        AvailabilityPublicationError::PublicationConflict => "publication_conflict",
        AvailabilityPublicationError::PackageInvalid => "package_invalid",
        AvailabilityPublicationError::Unavailable => "publication_unavailable",
        AvailabilityPublicationError::AmbiguousOutcome => "publication_outcome_unknown",
        AvailabilityPublicationError::InvalidResponse => "publication_response_invalid",
    }
}
