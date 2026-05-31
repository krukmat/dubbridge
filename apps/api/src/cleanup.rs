// T1-T2: periodic cleanup of expired pending ingestion sessions.
//
// Cleanup is idempotent and retry-safe:
//   1. Delete blob from storage (not-found is a warn, not an error — blob may have
//      been cleaned in a prior partial run).
//   2. Delete DB row (missing row is a no-op — already cleaned).
//
// If the process dies between steps 1 and 2, the next run re-finds the row and
// attempts storage delete again (harmless warn). This ensures blobs are never
// permanently orphaned.
//
// T1-T4: Cleanup-vs-finalize race window (documented invariant).
//
// A session is eligible for cleanup only when expires_at < now(). finalize_ingestion
// enforces the same expiry check before writing any records (fail-closed per ADR-008).
// The residual race: finalize loads a record just before it expires, passes the expiry
// check, then cleanup deletes the pending row and blob while finalize is in-flight.
// In this window finalize continues writing asset/rights/artifact records — these are
// valid, committed rows backed by a blob that cleanup just deleted.
//
// Risk: the original blob is gone but the artifact record references its storage_key.
// Future reads of that artifact would find a missing blob — a content gap, not silent
// data corruption (the record exists and is queryable).
//
// Mitigation: the race window is bounded by the 1-hour cleanup interval. A session can
// only enter this window at the exact moment of expiry, not throughout its lifetime.
// Closing the window fully would require a distributed lock on the pending row during
// finalize — deferred to a future hardening slice.
// Tracked: docs/tasks/tuning-hardening.md T1-T4.

use dubbridge_db::pending_ingestion_repo::list_expired_pending_ingestions;
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

pub async fn cleanup_expired_ingestions(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
) {
    let expired = match list_expired_pending_ingestions(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            tracing::warn!(error = %error, "failed to list expired pending ingestions; skipping cleanup run");
            return;
        }
    };

    if expired.is_empty() {
        return;
    }

    tracing::info!(
        count = expired.len(),
        "cleaning up expired pending ingestions"
    );

    for record in expired {
        let token = record.ingest_token;
        let key = &record.storage_key;

        // Step 1: delete blob. Not-found means it was already cleaned; warn and continue.
        match storage.delete(key).await {
            Ok(()) => {}
            Err(error) => {
                tracing::warn!(
                    ingest_token = %token,
                    storage_key = %key,
                    error = %error,
                    "storage delete failed for expired session; will retry on next cleanup run"
                );
                // Do not delete the DB row — retry next cycle.
                continue;
            }
        }

        // Step 2: delete DB row. Missing row is a no-op.
        if let Err(error) =
            dubbridge_db::pending_ingestion_repo::delete_pending_ingestion(pool, token).await
        {
            tracing::warn!(
                ingest_token = %token,
                error = %error,
                "failed to delete expired pending ingestion row after blob deletion"
            );
        } else {
            tracing::info!(ingest_token = %token, "expired pending ingestion cleaned up");
        }
    }
}
