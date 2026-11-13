//! B4.W3.2: dormant, capability-neutral Fenix dispatch-intent repository.
//! The table is a separately approved candidate (docs/proposals/0040*), NOT
//! included in the executable migrations. No API/worker calls this module.
//! Recording an admission ack is not recording successful model execution.

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

/// Created only after trusted DubBridge org/project authorization. The caller
/// MUST derive operation_id and trace_id from this very `id` and capability
/// using the existing Fenix connector factory, and must bind the SHA256 to
/// the final canonical request. This repository never generates a new UUID.
pub struct FenixIntentDraft {
    pub id: Uuid,
    pub requested_by_subject_id: Uuid,
    pub workspace_id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub capability_key: String,
    pub input_schema_version: String,
    pub request_sha256: [u8; 32],
    pub operation_id: Uuid,
    pub trace_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct FenixDispatchIntent {
    pub id: Uuid,
    pub requested_by_subject_id: Uuid,
    pub workspace_id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub capability_key: String,
    pub input_schema_version: String,
    pub request_sha256: Vec<u8>,
    pub operation_id: Uuid,
    pub trace_id: Uuid,
    pub delivery_state: String,
    pub send_token: Option<Uuid>,
    pub remote_replayed: Option<bool>,
    pub last_error_code: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Copy)]
pub enum FenixDispatchOutcome {
    /// This means only accepted by Fenix's durable admission, NOT a run.
    Admitted { replayed: bool },
    IndeterminateTimeout,
    IndeterminateInvalidResponse,
    IndeterminateProviderUnavailable,
    DeniedInvalidRequest,
    DeniedUnauthorized,
    DeniedConflict,
}

impl FenixDispatchOutcome {
    fn fields(self) -> (&'static str, Option<bool>, Option<&'static str>) {
        match self {
            Self::Admitted { replayed } => ("submitted", Some(replayed), None),
            Self::IndeterminateTimeout => ("indeterminate", None, Some("AMBIGUOUS_OUTCOME")),
            Self::IndeterminateInvalidResponse => ("indeterminate", None, Some("INVALID_RESPONSE")),
            Self::IndeterminateProviderUnavailable => {
                ("indeterminate", None, Some("PROVIDER_UNAVAILABLE"))
            }
            Self::DeniedInvalidRequest => ("rejected", None, Some("INVALID_REQUEST")),
            Self::DeniedUnauthorized => ("rejected", None, Some("REQUEST_DENIED")),
            Self::DeniedConflict => ("rejected", None, Some("OPERATION_CONFLICT")),
        }
    }
}

fn valid_intent_draft(d: &FenixIntentDraft) -> bool {
    !d.id.is_nil()
        && !d.requested_by_subject_id.is_nil()
        && !d.org_id.is_nil()
        && d.workspace_id == d.org_id
        && d.project_id.is_none_or(|id| !id.is_nil())
        && !d.operation_id.is_nil()
        && !d.trace_id.is_nil()
        && d.operation_id != d.trace_id
        && d.request_sha256.iter().any(|value| *value != 0)
        && d.capability_key.len() <= 96
        && d.capability_key.contains('.')
        && !d.input_schema_version.is_empty()
        && d.input_schema_version.len() <= 3
}

/// Insert only for an existing local member and a project belonging to the
/// chosen org. A nil row means NO authorization or unavailable scope; there
/// is no fallback that inserts a fake persisted work-item.
pub async fn prepare_fenix_dispatch_intent(
    pool: &PgPool,
    draft: &FenixIntentDraft,
) -> Result<FenixDispatchIntent, DbError> {
    if !valid_intent_draft(draft) {
        return Err(DbError::Conflict);
    }
    sqlx::query_as::<_, FenixDispatchIntent>(
        r#"
        INSERT INTO fenix_dispatch_intents (
            id, requested_by_subject_id, workspace_id, org_id, project_id,
            capability_key, input_schema_version, request_sha256,
            operation_id, trace_id
        )
        SELECT $1,$2,$3,$4,$5,$6,$7,$8,$9,$10
          FROM org_members m
         WHERE m.subject_id = $2 AND m.org_id = $4
           AND ($5::uuid IS NULL OR EXISTS (
               SELECT 1 FROM projects p WHERE p.id = $5 AND p.org_id = $4
           ))
        RETURNING *
        "#,
    )
    .bind(draft.id)
    .bind(draft.requested_by_subject_id)
    .bind(draft.workspace_id)
    .bind(draft.org_id)
    .bind(draft.project_id)
    .bind(&draft.capability_key)
    .bind(&draft.input_schema_version)
    .bind(draft.request_sha256.as_slice())
    .bind(draft.operation_id)
    .bind(draft.trace_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)
}

/// The ONLY automatic send claim. CAS prepared -> sending; never redispatch
/// an already sending, submitted, rejected or indeterminate work item.
/// A current org membership is rechecked at the claim boundary.
pub async fn claim_fenix_dispatch_once(
    pool: &PgPool,
    id: Uuid,
    actor_subject_id: Uuid,
    org_id: Uuid,
    send_token: Uuid,
) -> Result<FenixDispatchIntent, DbError> {
    if id.is_nil() || actor_subject_id.is_nil() || org_id.is_nil() || send_token.is_nil() {
        return Err(DbError::Conflict);
    }
    sqlx::query_as::<_, FenixDispatchIntent>(
        r#"
        UPDATE fenix_dispatch_intents i
           SET delivery_state = 'sending', send_token = $4, updated_at = now()
         WHERE i.id = $1
           AND i.requested_by_subject_id = $2
           AND i.org_id = $3 AND i.workspace_id = $3
           AND i.delivery_state = 'prepared'
           AND EXISTS (
               SELECT 1 FROM org_members m
                WHERE m.subject_id = $2 AND m.org_id = $3
           )
           AND (i.project_id IS NULL OR EXISTS (
               SELECT 1 FROM projects p
                WHERE p.id = i.project_id AND p.org_id = $3
           ))
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(actor_subject_id)
    .bind(org_id)
    .bind(send_token)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)
}

/// Persist a bounded classification after one send, with matching claim token.
/// This API cannot move indeterminate back to pending or automatically retry.
pub async fn finalize_fenix_dispatch_attempt(
    pool: &PgPool,
    id: Uuid,
    send_token: Uuid,
    outcome: FenixDispatchOutcome,
) -> Result<FenixDispatchIntent, DbError> {
    if id.is_nil() || send_token.is_nil() {
        return Err(DbError::Conflict);
    }
    let (state, replayed, error_code) = outcome.fields();
    sqlx::query_as::<_, FenixDispatchIntent>(
        r#"
        UPDATE fenix_dispatch_intents
           SET delivery_state = $3, remote_replayed = $4,
               last_error_code = $5, updated_at = now()
         WHERE id = $1 AND send_token = $2 AND delivery_state = 'sending'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(send_token)
    .bind(state)
    .bind(replayed)
    .bind(error_code)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)
}

/// Read only an intent still owned by the same active org member and scope.
/// A missing row, revoked membership or cross-org request all fail identically.
pub async fn read_fenix_dispatch_intent(
    pool: &PgPool,
    id: Uuid,
    actor_subject_id: Uuid,
    org_id: Uuid,
) -> Result<FenixDispatchIntent, DbError> {
    sqlx::query_as::<_, FenixDispatchIntent>(
        r#"
        SELECT i.*
          FROM fenix_dispatch_intents i
          JOIN org_members m ON m.org_id = i.org_id
             AND m.subject_id = i.requested_by_subject_id
         WHERE i.id = $1 AND i.requested_by_subject_id = $2
           AND i.org_id = $3 AND i.workspace_id = $3
           AND (i.project_id IS NULL OR EXISTS (
               SELECT 1 FROM projects p
                WHERE p.id = i.project_id AND p.org_id = $3
           ))
        "#,
    )
    .bind(id)
    .bind(actor_subject_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_rejects_nil_ids_wrong_scope_and_empty_fingerprint() {
        let mut draft = FenixIntentDraft {
            id: Uuid::new_v4(),
            requested_by_subject_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            project_id: None,
            capability_key: "integration.diagnostics".into(),
            input_schema_version: "1".into(),
            request_sha256: [1; 32],
            operation_id: Uuid::new_v4(),
            trace_id: Uuid::new_v4(),
        };
        assert!(!valid_intent_draft(&draft));
        draft.workspace_id = draft.org_id;
        assert!(valid_intent_draft(&draft));
        draft.request_sha256 = [0; 32];
        assert!(!valid_intent_draft(&draft));
        draft.request_sha256 = [1; 32];
        draft.operation_id = Uuid::nil();
        assert!(!valid_intent_draft(&draft));
    }

    #[test]
    fn remote_ack_means_submitted_not_completed() {
        assert_eq!(FenixDispatchOutcome::Admitted { replayed: false }.fields().0, "submitted");
        assert_eq!(FenixDispatchOutcome::IndeterminateTimeout.fields().0, "indeterminate");
        assert_eq!(FenixDispatchOutcome::DeniedConflict.fields().0, "rejected");
    }
}
