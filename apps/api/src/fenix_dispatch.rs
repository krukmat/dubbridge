//! B4.W3.3: an explicitly opt-in, NONPRODUCTION-ONLY application-side
//! composition of trusted DubBridge scope, durable work intent and the
//! one-shot Fenix S2S client. No API route, server or worker instantiates it.
//! Nothing in this module issues service tokens or applies the candidate DDL.

use dubbridge_connectors::fenix_s2s::{
    DubBridgeActor, FenixClient, FenixClientError, FenixCredentialProvider, FenixOperationRequest,
    FenixTransport, OperationParts, TenantSelection,
};
use dubbridge_db::error::DbError;
use dubbridge_db::fenix_dispatch_intent_repo::{
    FenixDispatchIntent, FenixDispatchOutcome, FenixIntentDraft, claim_fenix_dispatch_once,
    finalize_fenix_dispatch_attempt, prepare_fenix_dispatch_intent, read_fenix_dispatch_intent,
};
use dubbridge_domain::fenix_operation_identity::derive_fenix_operation_identity;
use dubbridge_domain::workspace::OrgRole;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::fenix_scope::PgFenixScopeAuthorizer;

/// A source-only guard, not an environment-variable-controlled bypass.
/// A real environment/token/route/capability enablement is W4 and has not
/// been authorized. Production application startup never constructs this.
#[derive(Debug, Clone, Default)]
pub struct FenixDispatchOptIn {
    pub enabled: bool,
    pub environment: String,
}

impl FenixDispatchOptIn {
    fn permits_test_only(&self) -> bool {
        self.enabled && self.environment == "integration-test"
    }
}

#[derive(Debug, Error)]
pub enum FenixDispatchError {
    #[error("Fenix dispatch is not enabled")]
    Disabled,
    #[error("Fenix dispatch is not authorized")]
    Denied,
    #[error("Fenix work intent is not eligible for a first send")]
    AlreadyClaimed,
    #[error("outgoing payload no longer matches the persisted work intent")]
    PayloadChanged,
    #[error("Fenix request could not be prepared")]
    InvalidRequest(#[from] FenixClientError),
    #[error("Fenix intent storage is unavailable")]
    Persistence(#[from] DbError),
}

/// The only allowed API-to-Fenix composition is constructed with a real
/// PostgreSQL org/project scope authorizer, never a permissive policy stub.
/// Construction alone does not register an application HTTP route.
pub struct FenixDispatchAdapter<T, C> {
    pool: PgPool,
    scope: PgFenixScopeAuthorizer,
    client: FenixClient<T, C>,
    opt_in: FenixDispatchOptIn,
}

impl<T: FenixTransport, C: FenixCredentialProvider> FenixDispatchAdapter<T, C> {
    pub fn new_nonproduction(
        pool: PgPool,
        minimum_org_role: OrgRole,
        client: FenixClient<T, C>,
        opt_in: FenixDispatchOptIn,
    ) -> Self {
        let scope = PgFenixScopeAuthorizer::new(pool.clone(), minimum_org_role);
        Self {
            pool,
            scope,
            client,
            opt_in,
        }
    }

    fn guard(&self, actor: &DubBridgeActor, target: &TenantSelection) -> Result<(), FenixDispatchError> {
        if !self.opt_in.permits_test_only() {
            return Err(FenixDispatchError::Disabled);
        }
        if actor.user_id.is_nil()
            || actor.workspace_id.is_nil()
            || target.org_id.is_nil()
            || actor.workspace_id != target.org_id
            || target.project_id.is_some_and(|id| id.is_nil())
        {
            return Err(FenixDispatchError::Denied);
        }
        Ok(())
    }

    /// W3.3.a: authorize and create exactly ONE immutable DB intent before
    /// any network request. The work ID is new only for this initial insert;
    /// all later operations MUST read the persisted ID. Persist the digest of
    /// the EXACT bytes FenixClient::submit_once will send (not a digest of
    /// arbitrary caller JSON or a claimed Fenix execution result).
    pub async fn prepare(
        &self,
        actor: &DubBridgeActor,
        target: &TenantSelection,
        capability_key: String,
        schema_version: String,
        input: Value,
    ) -> Result<Uuid, FenixDispatchError> {
        self.guard(actor, target)?;
        let intent_id = Uuid::new_v4();
        let (operation_id, trace_id) =
            derive_fenix_operation_identity(intent_id, &capability_key);
        let parts = OperationParts::from_persisted_work_item(
            intent_id, capability_key.clone(), schema_version.clone(), input,
        )?;
        let request = FenixOperationRequest::build(
            &self.scope, actor, target, parts,
        ).await?;
        if request.operation_id() != operation_id.to_string()
            || request.trace_id() != trace_id.to_string()
        {
            return Err(FenixDispatchError::PayloadChanged);
        }
        let digest = exact_wire_digest(&request)?;
        let draft = FenixIntentDraft {
            id: intent_id,
            requested_by_subject_id: actor.user_id,
            workspace_id: actor.workspace_id,
            org_id: target.org_id,
            project_id: target.project_id,
            capability_key,
            input_schema_version: schema_version,
            request_sha256: digest,
            operation_id,
            trace_id,
        };
        prepare_fenix_dispatch_intent(&self.pool, &draft).await?;
        Ok(intent_id)
    }

    /// W3.3.b: one-shot send, only after current membership/project permission
    /// is rechecked and the EXACT outbound byte digest matches the immutable
    /// intent. Atomic 'prepared -> sending' is committed BEFORE the HTTP POST.
    /// A crash after that point leaves 'sending' for manual W5 reconciliation.
    pub async fn submit_prepared_once(
        &self,
        actor: &DubBridgeActor,
        target: &TenantSelection,
        intent_id: Uuid,
        input: Value,
    ) -> Result<FenixDispatchIntent, FenixDispatchError> {
        self.guard(actor, target)?;
        let stored = read_fenix_dispatch_intent(
            &self.pool, intent_id, actor.user_id, target.org_id,
        ).await?;
        if stored.project_id != target.project_id || stored.workspace_id != actor.workspace_id {
            return Err(FenixDispatchError::Denied);
        }
        if stored.delivery_state != "prepared" {
            return Err(FenixDispatchError::AlreadyClaimed);
        }

        let parts = OperationParts::from_persisted_work_item(
            stored.id,
            stored.capability_key.clone(),
            stored.input_schema_version.clone(),
            input,
        )?;
        let request = FenixOperationRequest::build(
            &self.scope, actor, target, parts,
        ).await?;
        if request.operation_id() != stored.operation_id.to_string()
            || request.trace_id() != stored.trace_id.to_string()
            || !digest_matches(&stored.request_sha256, &exact_wire_digest(&request)?)
        {
            return Err(FenixDispatchError::PayloadChanged);
        }
        let send_token = Uuid::new_v4();
        // CAS and a fresh membership check; no network I/O if claim fails.
        claim_fenix_dispatch_once(
            &self.pool, intent_id, actor.user_id, target.org_id, send_token,
        )
        .await
        .map_err(|error| match error {
            DbError::Conflict => FenixDispatchError::AlreadyClaimed,
            other => FenixDispatchError::Persistence(other),
        })?;

        let outcome = match self.client.submit_once(&request).await {
            Ok(ack) => FenixDispatchOutcome::Admitted {
                replayed: ack.replayed,
            },
            Err(error) => classify_one_shot_outcome(error),
        };
        // If persistence fails, NEVER retry HTTP: sending remains unresolved.
        finalize_fenix_dispatch_attempt(
            &self.pool, intent_id, send_token, outcome,
        ).await.map_err(FenixDispatchError::Persistence)
    }
}

fn exact_wire_digest(request: &FenixOperationRequest) -> Result<[u8; 32], FenixClientError> {
    let bytes = request.outbound_json_bytes()?;
    Ok(Sha256::digest(bytes).into())
}

// Constant-time compare to avoid making a persisted digest a validity oracle.
fn digest_matches(persisted: &[u8], expected: &[u8; 32]) -> bool {
    if persisted.len() != expected.len() {
        return false;
    }
    let mismatch = persisted.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b));
    mismatch == 0
}

fn classify_one_shot_outcome(error: FenixClientError) -> FenixDispatchOutcome {
    match error {
        FenixClientError::AmbiguousOutcome => FenixDispatchOutcome::IndeterminateTimeout,
        FenixClientError::InvalidResponse => FenixDispatchOutcome::IndeterminateInvalidResponse,
        FenixClientError::Configuration => FenixDispatchOutcome::IndeterminateProviderUnavailable,
        FenixClientError::Denied => FenixDispatchOutcome::DeniedUnauthorized,
        FenixClientError::InvalidRequest => FenixDispatchOutcome::DeniedInvalidRequest,
        FenixClientError::Conflict => FenixDispatchOutcome::DeniedConflict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_in_defaults_off_and_never_permits_production() {
        assert!(!FenixDispatchOptIn::default().permits_test_only());
        assert!(!FenixDispatchOptIn {
            enabled: true,
            environment: "production".into(),
        }.permits_test_only());
        assert!(!FenixDispatchOptIn {
            enabled: false,
            environment: "integration-test".into(),
        }.permits_test_only());
        assert!(FenixDispatchOptIn {
            enabled: true,
            environment: "integration-test".into(),
        }.permits_test_only());
    }

    #[test]
    fn exact_digest_compare_rejects_mutation_and_wrong_size() {
        let hash = [7u8; 32];
        assert!(digest_matches(&hash, &hash));
        assert!(!digest_matches(&hash[..31], &hash));
        let mut altered = hash;
        altered[0] = 8;
        assert!(!digest_matches(&altered, &hash));
    }

    #[test]
    fn errors_never_trigger_automatic_resend() {
        assert!(matches!(
            classify_one_shot_outcome(FenixClientError::AmbiguousOutcome),
            FenixDispatchOutcome::IndeterminateTimeout
        ));
        assert!(matches!(
            classify_one_shot_outcome(FenixClientError::InvalidResponse),
            FenixDispatchOutcome::IndeterminateInvalidResponse
        ));
        assert!(matches!(
            classify_one_shot_outcome(FenixClientError::Conflict),
            FenixDispatchOutcome::DeniedConflict
        ));
    }
}
