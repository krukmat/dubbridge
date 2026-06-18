// T1-T2: periodic cleanup of expired pending ingestion sessions.
//
// H1-T1: cleanup now uses claim_expired_for_cleanup, which atomically deletes
// expired rows using a CTE with FOR UPDATE SKIP LOCKED. Any row currently locked
// by an in-flight finalize transaction is skipped and picked up on the next
// cleanup cycle. This closes the race window documented in the T1-T4 invariant:
// finalize holds a row lock (SELECT FOR UPDATE) for its entire transaction, so
// cleanup will never delete a blob that finalize is about to reference.
//
// Storage-blob deletion happens after the DB row is already gone. If a blob
// delete fails, the row is gone but the blob is orphaned. This is acceptable:
// the artifact record was never written (the row was still pending), so no
// asset points to the orphaned blob. ADR-006 assigns broader orphan
// reconciliation to S2.

use std::collections::BTreeSet;

use dubbridge_db::pending_ingestion_repo::claim_expired_for_cleanup;
use dubbridge_storage::{INGESTS_PREFIX, StorageAdapter};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestReconciliationPlan {
    pub retained: Vec<String>,
    pub orphan_candidates: Vec<String>,
    pub skipped: Vec<SkippedReconciliationKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestReconciliationRun {
    pub plan: IngestReconciliationPlan,
    pub deleted: Vec<String>,
    pub already_absent: Vec<String>,
    pub failed: Vec<FailedReconciliationDelete>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedReconciliationDelete {
    pub key: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedReconciliationKey {
    pub key: String,
    pub reason: ReconciliationSkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationSkipReason {
    InvalidPrefix,
    InvalidIngestToken,
    MissingObjectName,
    UnexpectedObjectPath,
}

pub async fn cleanup_expired_ingestions(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
) {
    let claimed = match claim_expired_for_cleanup(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            tracing::warn!(error = %error, "failed to claim expired pending ingestions; skipping cleanup run");
            return;
        }
    };

    if claimed.is_empty() {
        return;
    }

    tracing::info!(
        count = claimed.len(),
        "cleaning up claimed expired ingestions"
    );

    for (token, key) in claimed {
        match storage.delete(&key).await {
            Ok(()) => {
                tracing::info!(ingest_token = %token, "expired pending ingestion blob deleted");
            }
            Err(error) => {
                // DB row already gone (claimed atomically). Blob is now orphaned.
                // ADR-006 assigns cross-store orphan reconciliation to S2.
                tracing::warn!(
                    ingest_token = %token,
                    storage_key = %key,
                    error = %error,
                    "storage delete failed for claimed expired session; blob orphaned (reconcile via S2)"
                );
            }
        }
    }
}

pub async fn plan_ingest_reconciliation(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
) -> anyhow::Result<IngestReconciliationPlan> {
    let candidates = storage.list_keys(INGESTS_PREFIX).await?;
    let referenced = load_referenced_storage_keys(pool).await?;
    Ok(plan_from_candidate_keys(candidates, referenced))
}

pub async fn run_ingest_reconciliation(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
) -> anyhow::Result<IngestReconciliationRun> {
    let plan = plan_ingest_reconciliation(pool, storage).await?;
    let mut deleted = Vec::new();
    let mut already_absent = Vec::new();
    let mut failed = Vec::new();

    for key in &plan.orphan_candidates {
        match storage.delete(key).await {
            Ok(()) => {
                tracing::info!(
                    storage_key = %key,
                    "deleted orphaned ingest object during reconciliation"
                );
                deleted.push(key.clone());
            }
            Err(dubbridge_storage::StorageError::NotFound { .. }) => {
                tracing::info!(
                    storage_key = %key,
                    "orphaned ingest object already absent during reconciliation"
                );
                already_absent.push(key.clone());
            }
            Err(error) => {
                let error = error.to_string();
                tracing::warn!(
                    storage_key = %key,
                    error = %error,
                    "failed to delete orphaned ingest object during reconciliation; will retry on a later run"
                );
                failed.push(FailedReconciliationDelete {
                    key: key.clone(),
                    error,
                });
            }
        }
    }

    tracing::info!(
        retained = plan.retained.len(),
        skipped = plan.skipped.len(),
        orphan_candidates = plan.orphan_candidates.len(),
        deleted = deleted.len(),
        already_absent = already_absent.len(),
        failed = failed.len(),
        "completed ingest object reconciliation run"
    );

    Ok(IngestReconciliationRun {
        plan,
        deleted,
        already_absent,
        failed,
    })
}

async fn load_referenced_storage_keys(pool: &PgPool) -> Result<BTreeSet<String>, sqlx::Error> {
    let keys: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT storage_key FROM pending_ingestions
        UNION
        SELECT storage_key FROM artifact_records
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(keys.into_iter().collect())
}

fn plan_from_candidate_keys(
    candidates: Vec<String>,
    referenced: BTreeSet<String>,
) -> IngestReconciliationPlan {
    let mut retained = Vec::new();
    let mut orphan_candidates = Vec::new();
    let mut skipped = Vec::new();

    for key in candidates.into_iter().collect::<BTreeSet<_>>() {
        if let Err(reason) = validate_ingest_candidate_key(&key) {
            tracing::warn!(
                storage_key = %key,
                reason = ?reason,
                "skipping malformed storage reconciliation candidate"
            );
            skipped.push(SkippedReconciliationKey { key, reason });
            continue;
        }

        if referenced.contains(&key) {
            retained.push(key);
        } else {
            orphan_candidates.push(key);
        }
    }

    IngestReconciliationPlan {
        retained,
        orphan_candidates,
        skipped,
    }
}

fn validate_ingest_candidate_key(key: &str) -> Result<(), ReconciliationSkipReason> {
    let rest = key
        .strip_prefix(INGESTS_PREFIX)
        .ok_or(ReconciliationSkipReason::InvalidPrefix)?;
    let (token, object_name) = rest
        .split_once('/')
        .ok_or(ReconciliationSkipReason::MissingObjectName)?;

    Uuid::parse_str(token).map_err(|_| ReconciliationSkipReason::InvalidIngestToken)?;

    if object_name.is_empty() {
        return Err(ReconciliationSkipReason::MissingObjectName);
    }

    if object_name.contains('/') {
        return Err(ReconciliationSkipReason::UnexpectedObjectPath);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_from_candidate_keys_deduplicates_and_skips_malformed_keys() {
        let token = Uuid::nil();
        let referenced_key = format!("ingests/{token}/referenced.mp4");
        let orphan_key = format!("ingests/{token}/orphan.mp4");
        let duplicate_orphan_key = orphan_key.clone();
        let invalid_token_key = "ingests/not-a-token/file.mp4".to_string();
        let nested_key = format!("ingests/{token}/nested/file.mp4");
        let wrong_prefix_key = format!("assets/{token}/file.mp4");

        let plan = plan_from_candidate_keys(
            vec![
                duplicate_orphan_key,
                referenced_key.clone(),
                invalid_token_key.clone(),
                orphan_key.clone(),
                nested_key.clone(),
                wrong_prefix_key.clone(),
            ],
            BTreeSet::from([referenced_key.clone()]),
        );

        assert_eq!(plan.retained, vec![referenced_key]);
        assert_eq!(plan.orphan_candidates, vec![orphan_key]);
        assert_eq!(
            plan.skipped,
            vec![
                SkippedReconciliationKey {
                    key: wrong_prefix_key,
                    reason: ReconciliationSkipReason::InvalidPrefix,
                },
                SkippedReconciliationKey {
                    key: nested_key,
                    reason: ReconciliationSkipReason::UnexpectedObjectPath,
                },
                SkippedReconciliationKey {
                    key: invalid_token_key,
                    reason: ReconciliationSkipReason::InvalidIngestToken,
                },
            ]
        );
    }
}
