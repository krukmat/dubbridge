// MVP0-P2P P2.T4b: bounded PostgreSQL claim/lease/release operations.
//
// This module deliberately owns no remote dispatch behavior. It only provides
// durable single-owner work leasing over the authoritative publication outbox.

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::p2p_publication::{K1LineageId, P2pPublicationId};

use crate::error::DbError;

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

    if result.rows_affected() == 1 {
        return Ok(());
    }

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM p2p_publication_outbox WHERE id = $1)",
    )
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
