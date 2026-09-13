use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredAuthenticator {
    pub id: String,
    pub principal_kind: String,
    pub principal_id: String,
    pub kind: String,
    pub public_key_json: String,
    pub active: bool,
    pub enrolled_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredChallenge {
    pub request_id: Uuid,
    pub nonce: String,
    pub challenge_digest: String,
    pub package_json: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub satisfied_at: Option<OffsetDateTime>,
}

pub async fn get_authenticator(
    pool: &PgPool,
    authenticator_id: &str,
) -> Result<StoredAuthenticator, DbError> {
    sqlx::query_as::<_, StoredAuthenticator>(
        r#"
        SELECT id, principal_kind, principal_id, kind, public_key_json,
               active, enrolled_at, revoked_at
        FROM haa_authenticators
        WHERE id = $1
        "#,
    )
    .bind(authenticator_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

pub async fn get_challenge(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<StoredChallenge, DbError> {
    sqlx::query_as::<_, StoredChallenge>(
        r#"
        SELECT request_id, nonce, challenge_digest, package_json,
               issued_at, expires_at, satisfied_at
        FROM haa_challenges
        WHERE request_id = $1
        "#,
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}
