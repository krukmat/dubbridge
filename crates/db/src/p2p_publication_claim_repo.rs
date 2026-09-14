// MVP0-P2P P2.T4b: bounded PostgreSQL claim/lease/release operations.
//
// This module deliberately owns no remote dispatch behavior. It only provides
// durable single-owner work leasing over the authoritative publication outbox.

use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    p2p_publication::{K1LineageId, P2pPublicationId},
};

use crate::{audit_repo::insert_audit_event_tx, error::DbError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pPublicationClaim {
    pub outbox_id: Uuid,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub claim_token: Uuid,
    pub attempt_count: i32,
    pub claimed_at: OffsetDateTime,
    pub lease_expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy)]
pub struct ReadyFinalization<'a> {
    pub outbox_id: Uuid,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub claim_token: Uuid,
    pub external_publication_id: &'a str,
    pub confirmed_at: OffsetDateTime,
    pub delivered_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct ClaimRow {
    outbox_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    claim_token: Uuid,
    attempt_count: i32,
    claimed_at: OffsetDateTime,
    lease_expires_at: OffsetDateTime,
}

fn claim_from_row(row: ClaimRow) -> P2pPublicationClaim {
    P2pPublicationClaim {
        outbox_id: row.outbox_id,
        publication_id: P2pPublicationId(row.publication_id),
        lineage_id: K1LineageId(row.lineage_id),
        claim_token: row.claim_token,
        attempt_count: row.attempt_count,
        claimed_at: row.claimed_at,
        lease_expires_at: row.lease_expires_at,
    }
}

/// Atomically claim the oldest currently-dispatchable publication obligation.
///
/// A live claim is never stolen. An expired claim is eligible for recovery and
/// is re-claimed with a new token while incrementing the durable attempt count.
/// `FOR UPDATE SKIP LOCKED` keeps competing workers from serializing behind the
/// same row while PostgreSQL remains the authority.
pub async fn claim_next_publication_work(
    pool: &PgPool,
    claim_token: Uuid,
    lease_expires_at: OffsetDateTime,
) -> Result<Option<P2pPublicationClaim>, DbError> {
    if claim_token.is_nil() || lease_expires_at <= OffsetDateTime::now_utc() {
        return Err(DbError::Conflict);
    }

    let row = sqlx::query_as::<_, ClaimRow>(
        r#"
        WITH candidate AS (
            SELECT o.id
              FROM p2p_publication_outbox o
              JOIN p2p_publications p ON p.id = o.publication_id
             WHERE p.state IN ('publish_pending', 'publishing', 'reconciling')
               AND o.delivered_at IS NULL
               AND (
                    (o.delivery_state = 'pending' AND o.available_at <= now())
                    OR
                    (o.delivery_state = 'claimed' AND o.lease_expires_at <= now())
               )
             ORDER BY o.available_at ASC, o.created_at ASC
             FOR UPDATE OF o SKIP LOCKED
             LIMIT 1
        )
        UPDATE p2p_publication_outbox o
           SET delivery_state = 'claimed',
               attempt_count = o.attempt_count + 1,
               claimed_at = now(),
               claim_token = $1,
               lease_expires_at = $2,
               updated_at = now()
          FROM candidate
         WHERE o.id = candidate.id
           AND $2 > now()
        RETURNING o.id AS outbox_id,
                  o.publication_id,
                  o.lineage_id,
                  o.claim_token,
                  o.attempt_count,
                  o.claimed_at,
                  o.lease_expires_at
        "#,
    )
    .bind(claim_token)
    .bind(lease_expires_at)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(claim_from_row))
}

/// Release a claim owned by `claim_token` back to the durable pending set.
///
/// Releasing with a stale or foreign token fails closed. `retry_at` controls
/// when the outbox obligation becomes claimable again; `last_error` is bounded
/// to operational diagnostics and must not contain secret material.
pub async fn release_publication_claim(
    pool: &PgPool,
    outbox_id: Uuid,
    claim_token: Uuid,
    retry_at: OffsetDateTime,
    last_error: Option<&str>,
) -> Result<(), DbError> {
    if claim_token.is_nil() || last_error.is_some_and(|value| value.trim().is_empty()) {
        return Err(DbError::Conflict);
    }

    let result = sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'pending',
               available_at = $3,
               claim_token = NULL,
               lease_expires_at = NULL,
               last_error = $4,
               updated_at = now()
         WHERE id = $1
           AND delivery_state = 'claimed'
           AND claim_token = $2
        "#,
    )
    .bind(outbox_id)
    .bind(claim_token)
    .bind(retry_at)
    .bind(last_error)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    resolve_owned_mutation(pool, outbox_id, result.rows_affected()).await
}

/// Mark a successfully reconciled obligation delivered after durable Ready.
///
/// Completion remains claim-token guarded so a stale worker cannot acknowledge
/// work reclaimed by another owner. Remote success alone never calls this path:
/// callers must first persist same-lineage confirmation and the Ready transition.
pub async fn complete_publication_claim(
    pool: &PgPool,
    outbox_id: Uuid,
    claim_token: Uuid,
    delivered_at: OffsetDateTime,
) -> Result<(), DbError> {
    if claim_token.is_nil() {
        return Err(DbError::Conflict);
    }

    let result = sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'delivered',
               delivered_at = $3,
               claim_token = NULL,
               lease_expires_at = NULL,
               last_error = NULL,
               updated_at = now()
         WHERE id = $1
           AND delivery_state = 'claimed'
           AND claim_token = $2
        "#,
    )
    .bind(outbox_id)
    .bind(claim_token)
    .bind(delivered_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    resolve_owned_mutation(pool, outbox_id, result.rows_affected()).await
}

/// Persist same-lineage external confirmation, transition the publication to
/// `ready`, acknowledge the owned outbox claim, and record both required P2
/// audit events in one PostgreSQL transaction.
pub async fn finalize_publication_ready(
    pool: &PgPool,
    finalization: ReadyFinalization<'_>,
) -> Result<(), DbError> {
    if finalization.claim_token.is_nil() || finalization.external_publication_id.trim().is_empty() {
        return Err(DbError::Conflict);
    }

    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    ensure_claim_owned(
        &mut tx,
        finalization.outbox_id,
        finalization.publication_id,
        finalization.lineage_id,
        finalization.claim_token,
    )
    .await?;

    let asset_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE p2p_publications
           SET external_publication_id = $3,
               confirmed_lineage_id = $2,
               external_confirmed_at = COALESCE(external_confirmed_at, $4),
               state = 'ready',
               failure_detail = NULL,
               updated_at = now()
         WHERE id = $1
           AND lineage_id = $2
           AND state IN ('publishing', 'reconciling')
           AND (external_publication_id IS NULL OR external_publication_id = $3)
           AND (confirmed_lineage_id IS NULL OR confirmed_lineage_id = $2)
        RETURNING asset_id
        "#,
    )
    .bind(finalization.publication_id.0)
    .bind(finalization.lineage_id.0)
    .bind(finalization.external_publication_id)
    .bind(finalization.confirmed_at)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)?;

    let outbox_update = sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'delivered',
               delivered_at = $3,
               claim_token = NULL,
               lease_expires_at = NULL,
               last_error = NULL,
               updated_at = now()
         WHERE id = $1
           AND delivery_state = 'claimed'
           AND claim_token = $2
        "#,
    )
    .bind(finalization.outbox_id)
    .bind(finalization.claim_token)
    .bind(finalization.delivered_at)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;
    if outbox_update.rows_affected() != 1 {
        return Err(DbError::Conflict);
    }

    let confirmed = AuditEvent::new_p2p_event(
        AssetId(asset_id),
        AuditEventKind::P2pPublicationConfirmed,
        finalization.publication_id.0,
        finalization.lineage_id.0,
        None,
    );
    insert_audit_event_tx(&mut tx, &confirmed).await?;
    let ready = AuditEvent::new_p2p_event(
        AssetId(asset_id),
        AuditEventKind::P2pPublicationReady,
        finalization.publication_id.0,
        finalization.lineage_id.0,
        None,
    );
    insert_audit_event_tx(&mut tx, &ready).await?;

    tx.commit().await.map_err(DbError::QueryFailed)
}

/// Atomically persist terminal publication failure, release the owned claim,
/// and persist the correlated terminal audit event.
pub async fn fail_publication_claim(
    pool: &PgPool,
    outbox_id: Uuid,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    claim_token: Uuid,
    failed_at: OffsetDateTime,
    reason: &str,
) -> Result<(), DbError> {
    if claim_token.is_nil() || reason.trim().is_empty() {
        return Err(DbError::Conflict);
    }

    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    ensure_claim_owned(&mut tx, outbox_id, publication_id, lineage_id, claim_token).await?;

    let asset_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE p2p_publications
           SET state = 'failed',
               failure_detail = $3,
               updated_at = now()
         WHERE id = $1
           AND lineage_id = $2
           AND state IN ('publish_pending', 'publishing', 'reconciling')
        RETURNING asset_id
        "#,
    )
    .bind(publication_id.0)
    .bind(lineage_id.0)
    .bind(reason)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)?;

    let outbox_update = sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'pending',
               available_at = $3,
               claim_token = NULL,
               lease_expires_at = NULL,
               last_error = $4,
               updated_at = now()
         WHERE id = $1
           AND delivery_state = 'claimed'
           AND claim_token = $2
        "#,
    )
    .bind(outbox_id)
    .bind(claim_token)
    .bind(failed_at)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;
    if outbox_update.rows_affected() != 1 {
        return Err(DbError::Conflict);
    }

    let failed = AuditEvent::new_p2p_event(
        AssetId(asset_id),
        AuditEventKind::P2pPublicationFailed,
        publication_id.0,
        lineage_id.0,
        Some(reason.to_owned()),
    );
    insert_audit_event_tx(&mut tx, &failed).await?;

    tx.commit().await.map_err(DbError::QueryFailed)
}

async fn ensure_claim_owned(
    tx: &mut Transaction<'_, Postgres>,
    outbox_id: Uuid,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    claim_token: Uuid,
) -> Result<(), DbError> {
    let owned = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT publication_id, lineage_id
          FROM p2p_publication_outbox
         WHERE id = $1
           AND delivery_state = 'claimed'
           AND claim_token = $2
         FOR UPDATE
        "#,
    )
    .bind(outbox_id)
    .bind(claim_token)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    match owned {
        Some((stored_publication_id, stored_lineage_id))
            if stored_publication_id == publication_id.0 && stored_lineage_id == lineage_id.0 =>
        {
            Ok(())
        }
        Some(_) => Err(DbError::Conflict),
        None => {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM p2p_publication_outbox WHERE id = $1)",
            )
            .bind(outbox_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::QueryFailed)?;
            if exists {
                Err(DbError::Conflict)
            } else {
                Err(DbError::NotFound)
            }
        }
    }
}

async fn resolve_owned_mutation(
    pool: &PgPool,
    outbox_id: Uuid,
    rows_affected: u64,
) -> Result<(), DbError> {
    if rows_affected == 1 {
        return Ok(());
    }

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM p2p_publication_outbox WHERE id = $1)")
            .bind(outbox_id)
            .fetch_one(pool)
            .await
            .map_err(DbError::QueryFailed)?;

    if exists {
        Err(DbError::Conflict)
    } else {
        Err(DbError::NotFound)
    }
}
