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

use dubbridge_db::pending_ingestion_repo::claim_expired_for_cleanup;
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

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
