use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

#[derive(Debug, Clone)]
pub struct NewAuthenticator<'a> {
    pub id: &'a str,
    pub principal_kind: &'a str,
    pub principal_id: &'a str,
    pub kind: &'a str,
    pub public_key_json: &'a str,
    pub enrolled_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewApprovalRequest<'a> {
    pub id: Uuid,
    pub protocol_version: &'a str,
    pub action_schema: &'a str,
    pub action_type: &'a str,
    pub resource: &'a str,
    pub environment: Option<&'a str>,
    pub canonical_action: &'a str,
    pub canonical_intent: &'a str,
    pub action_digest: &'a str,
    pub intent_digest: &'a str,
    pub requester_kind: &'a str,
    pub requester_id: &'a str,
    pub audience_kind: &'a str,
    pub audience_id: &'a str,
    pub approver_kind: &'a str,
    pub approver_id: &'a str,
    pub policy_id: &'a str,
    pub policy_version: &'a str,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewChallenge<'a> {
    pub request_id: Uuid,
    pub nonce: &'a str,
    pub challenge_digest: &'a str,
    pub package_json: &'a str,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct VerifiedApproval<'a> {
    pub evidence_id: Uuid,
    pub request_id: Uuid,
    pub authenticator_id: &'a str,
    pub challenge_digest: &'a str,
    pub evidence_kind: &'a str,
    pub evidence_json: &'a str,
    pub receipt_id: Uuid,
    pub receipt_json: &'a str,
    pub verified_at: OffsetDateTime,
    pub receipt_expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewExecutionGrant<'a> {
    pub execution_id: Uuid,
    pub request_id: Uuid,
    pub action_digest: &'a str,
    pub audience_kind: &'a str,
    pub audience_id: &'a str,
    pub grant_json: &'a str,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApprovalRequestRecord {
    pub id: Uuid,
    pub protocol_version: String,
    pub canonical_action: String,
    pub canonical_intent: String,
    pub action_digest: String,
    pub intent_digest: String,
    pub requester_kind: String,
    pub requester_id: String,
    pub audience_kind: String,
    pub audience_id: String,
    pub approver_kind: String,
    pub approver_id: String,
    pub policy_id: String,
    pub policy_version: String,
    pub state: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub execution_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeResult {
    Consumed { grant_json: String },
    Idempotent { grant_json: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ConsumeError {
    #[error(transparent)]
    Database(#[from] DbError),
    #[error("approval is not approved")]
    NotApproved,
    #[error("approval expired")]
    Expired,
    #[error("action digest does not match approved action")]
    ActionMismatch,
    #[error("executor audience does not match approval")]
    AudienceMismatch,
    #[error("approval was already consumed by another execution")]
    AlreadyConsumed,
}

pub async fn insert_authenticator(pool: &PgPool, value: &NewAuthenticator<'_>) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO haa_authenticators
            (id, principal_kind, principal_id, kind, public_key_json, active, enrolled_at)
        VALUES ($1, $2, $3, $4, $5, TRUE, $6)
        "#,
    )
    .bind(value.id)
    .bind(value.principal_kind)
    .bind(value.principal_id)
    .bind(value.kind)
    .bind(value.public_key_json)
    .bind(value.enrolled_at)
    .execute(pool)
    .await
    .map_err(map_query_error)?;
    Ok(())
}

pub async fn revoke_authenticator(
    pool: &PgPool,
    authenticator_id: &str,
    revoked_at: OffsetDateTime,
) -> Result<(), DbError> {
    let result = sqlx::query(
        "UPDATE haa_authenticators SET active = FALSE, revoked_at = $2 WHERE id = $1 AND active",
    )
    .bind(authenticator_id)
    .bind(revoked_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn insert_request(pool: &PgPool, value: &NewApprovalRequest<'_>) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO haa_approval_requests (
            id, protocol_version, action_schema, action_type, resource, environment,
            canonical_action, canonical_intent, action_digest, intent_digest,
            requester_kind, requester_id, audience_kind, audience_id,
            approver_kind, approver_id, policy_id, policy_version, state,
            created_at, updated_at, expires_at
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,
            'pending',$19,$19,$20
        )
        "#,
    )
    .bind(value.id)
    .bind(value.protocol_version)
    .bind(value.action_schema)
    .bind(value.action_type)
    .bind(value.resource)
    .bind(value.environment)
    .bind(value.canonical_action)
    .bind(value.canonical_intent)
    .bind(value.action_digest)
    .bind(value.intent_digest)
    .bind(value.requester_kind)
    .bind(value.requester_id)
    .bind(value.audience_kind)
    .bind(value.audience_id)
    .bind(value.approver_kind)
    .bind(value.approver_id)
    .bind(value.policy_id)
    .bind(value.policy_version)
    .bind(value.created_at)
    .bind(value.expires_at)
    .execute(pool)
    .await
    .map_err(map_query_error)?;
    Ok(())
}

pub async fn get_request(pool: &PgPool, id: Uuid) -> Result<ApprovalRequestRecord, DbError> {
    sqlx::query_as::<_, ApprovalRequestRecord>(
        r#"
        SELECT id, protocol_version, canonical_action, canonical_intent,
               action_digest, intent_digest, requester_kind, requester_id,
               audience_kind, audience_id, approver_kind, approver_id,
               policy_id, policy_version, state, created_at, updated_at,
               expires_at, execution_id
        FROM haa_approval_requests
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

pub async fn insert_challenge(pool: &PgPool, value: &NewChallenge<'_>) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO haa_challenges
            (request_id, nonce, challenge_digest, package_json, issued_at, expires_at)
        VALUES ($1,$2,$3,$4,$5,$6)
        "#,
    )
    .bind(value.request_id)
    .bind(value.nonce)
    .bind(value.challenge_digest)
    .bind(value.package_json)
    .bind(value.issued_at)
    .bind(value.expires_at)
    .execute(pool)
    .await
    .map_err(map_query_error)?;
    Ok(())
}

pub async fn approve(pool: &PgPool, value: &VerifiedApproval<'_>) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    lock_pending_request(&mut tx, value.request_id).await?;

    let challenge_updated = sqlx::query(
        r#"
        UPDATE haa_challenges
        SET satisfied_at = $3
        WHERE request_id = $1 AND challenge_digest = $2
          AND satisfied_at IS NULL AND expires_at > $3
        "#,
    )
    .bind(value.request_id)
    .bind(value.challenge_digest)
    .bind(value.verified_at)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;
    if challenge_updated.rows_affected() != 1 {
        return Err(DbError::Conflict);
    }

    sqlx::query(
        r#"
        INSERT INTO haa_approval_evidence
            (id, request_id, authenticator_id, challenge_digest, kind, evidence_json, verified_at)
        SELECT $1,$2,$3,$4,$5,$6,$7
        WHERE EXISTS (
            SELECT 1 FROM haa_authenticators
            WHERE id = $3 AND active
        )
        "#,
    )
    .bind(value.evidence_id)
    .bind(value.request_id)
    .bind(value.authenticator_id)
    .bind(value.challenge_digest)
    .bind(value.evidence_kind)
    .bind(value.evidence_json)
    .bind(value.verified_at)
    .execute(&mut *tx)
    .await
    .map_err(map_query_error)?;

    sqlx::query(
        r#"
        INSERT INTO haa_approval_receipts (id, request_id, receipt_json, issued_at, expires_at)
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(value.receipt_id)
    .bind(value.request_id)
    .bind(value.receipt_json)
    .bind(value.verified_at)
    .bind(value.receipt_expires_at)
    .execute(&mut *tx)
    .await
    .map_err(map_query_error)?;

    sqlx::query(
        "UPDATE haa_approval_requests SET state = 'approved', updated_at = $2 WHERE id = $1",
    )
    .bind(value.request_id)
    .bind(value.verified_at)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn reject(
    pool: &PgPool,
    request_id: Uuid,
    rejected_at: OffsetDateTime,
) -> Result<(), DbError> {
    let result = sqlx::query(
        r#"
        UPDATE haa_approval_requests
        SET state = 'rejected', updated_at = $2
        WHERE id = $1 AND state IN ('requested','pending')
        "#,
    )
    .bind(request_id)
    .bind(rejected_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    if result.rows_affected() != 1 {
        return Err(DbError::Conflict);
    }
    Ok(())
}

pub async fn authorize_and_consume(
    pool: &PgPool,
    grant: &NewExecutionGrant<'_>,
    now: OffsetDateTime,
) -> Result<ConsumeResult, ConsumeError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DbError::QueryFailed)
        .map_err(ConsumeError::Database)?;

    #[derive(sqlx::FromRow)]
    struct LockedRequest {
        state: String,
        action_digest: String,
        audience_kind: String,
        audience_id: String,
        expires_at: OffsetDateTime,
        execution_id: Option<Uuid>,
    }

    let row = sqlx::query_as::<_, LockedRequest>(
        r#"
        SELECT state, action_digest, audience_kind, audience_id, expires_at, execution_id
        FROM haa_approval_requests
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(grant.request_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)
    .map_err(ConsumeError::Database)?
    .ok_or(ConsumeError::Database(DbError::NotFound))?;

    if row.state == "consumed" {
        if row.execution_id == Some(grant.execution_id) {
            let grant_json: String = sqlx::query_scalar(
                "SELECT grant_json FROM haa_execution_grants WHERE execution_id = $1",
            )
            .bind(grant.execution_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::QueryFailed)
            .map_err(ConsumeError::Database)?;
            tx.commit()
                .await
                .map_err(DbError::QueryFailed)
                .map_err(ConsumeError::Database)?;
            return Ok(ConsumeResult::Idempotent { grant_json });
        }
        return Err(ConsumeError::AlreadyConsumed);
    }
    if row.state != "approved" {
        return Err(ConsumeError::NotApproved);
    }
    if row.expires_at <= now || grant.expires_at <= now {
        sqlx::query(
            "UPDATE haa_approval_requests SET state = 'expired', updated_at = $2 WHERE id = $1",
        )
        .bind(grant.request_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(DbError::QueryFailed)
        .map_err(ConsumeError::Database)?;
        tx.commit()
            .await
            .map_err(DbError::QueryFailed)
            .map_err(ConsumeError::Database)?;
        return Err(ConsumeError::Expired);
    }
    if row.action_digest != grant.action_digest {
        return Err(ConsumeError::ActionMismatch);
    }
    if row.audience_kind != grant.audience_kind || row.audience_id != grant.audience_id {
        return Err(ConsumeError::AudienceMismatch);
    }

    sqlx::query(
        r#"
        INSERT INTO haa_execution_grants
            (execution_id, request_id, action_digest, audience_kind, audience_id,
             grant_json, issued_at, expires_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(grant.execution_id)
    .bind(grant.request_id)
    .bind(grant.action_digest)
    .bind(grant.audience_kind)
    .bind(grant.audience_id)
    .bind(grant.grant_json)
    .bind(grant.issued_at)
    .bind(grant.expires_at)
    .execute(&mut *tx)
    .await
    .map_err(map_query_error)
    .map_err(ConsumeError::Database)?;

    sqlx::query(
        r#"
        UPDATE haa_approval_requests
        SET state = 'consumed', consumed_at = $2, execution_id = $3, updated_at = $2
        WHERE id = $1
        "#,
    )
    .bind(grant.request_id)
    .bind(now)
    .bind(grant.execution_id)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)
    .map_err(ConsumeError::Database)?;

    tx.commit()
        .await
        .map_err(DbError::QueryFailed)
        .map_err(ConsumeError::Database)?;
    Ok(ConsumeResult::Consumed {
        grant_json: grant.grant_json.to_owned(),
    })
}

async fn lock_pending_request(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
) -> Result<(), DbError> {
    let state: Option<String> = sqlx::query_scalar(
        "SELECT state FROM haa_approval_requests WHERE id = $1 FOR UPDATE",
    )
    .bind(request_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    match state.as_deref() {
        Some("pending") | Some("requested") => Ok(()),
        Some(_) => Err(DbError::Conflict),
        None => Err(DbError::NotFound),
    }
}

fn map_query_error(error: sqlx::Error) -> DbError {
    match &error {
        sqlx::Error::Database(database)
            if database.is_unique_violation() || database.is_foreign_key_violation() =>
        {
            DbError::Conflict
        }
        _ => DbError::QueryFailed(error),
    }
}
