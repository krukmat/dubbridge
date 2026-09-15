// MVP0-P2P P3: invitation, active-device and O3 audience authorization persistence.

use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId},
};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct P2pDeviceRecord {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub key_id: String,
    pub public_key_spki: Vec<u8>,
    pub created_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct P2pInvitationRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub publication_id: Uuid,
    pub lineage_id: Uuid,
    pub owner_subject_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub claimed_by_subject_id: Option<Uuid>,
    pub claimed_device_id: Option<Uuid>,
    pub claimed_at: Option<OffsetDateTime>,
    pub revoked_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct P2pAudienceAuthorizationRecord {
    pub id: Uuid,
    pub invitation_id: Uuid,
    pub asset_id: Uuid,
    pub publication_id: Uuid,
    pub lineage_id: Uuid,
    pub viewer_subject_id: Uuid,
    pub device_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct P2pClaimResult {
    pub invitation: P2pInvitationRecord,
    pub authorization: P2pAudienceAuthorizationRecord,
    pub device: P2pDeviceRecord,
}

#[derive(sqlx::FromRow)]
struct ReadyPublicationRow {
    publication_id: Uuid,
    lineage_id: Uuid,
}

pub async fn register_or_get_active_device(
    pool: &PgPool,
    subject_id: Uuid,
    key_id: &str,
    public_key_spki: &[u8],
) -> Result<P2pDeviceRecord, DbError> {
    if key_id.trim().is_empty() || public_key_spki.is_empty() {
        return Err(DbError::Conflict);
    }

    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let active = sqlx::query_as::<_, P2pDeviceRecord>(
        r#"
        SELECT id, subject_id, key_id, public_key_spki, created_at, revoked_at
          FROM p2p_devices
         WHERE subject_id = $1
           AND revoked_at IS NULL
         FOR UPDATE
        "#,
    )
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    if let Some(existing) = active {
        if existing.key_id == key_id && existing.public_key_spki == public_key_spki {
            tx.commit().await.map_err(DbError::QueryFailed)?;
            return Ok(existing);
        }
        return Err(DbError::Conflict);
    }

    let record = sqlx::query_as::<_, P2pDeviceRecord>(
        r#"
        INSERT INTO p2p_devices (id, subject_id, key_id, public_key_spki)
        VALUES ($1, $2, $3, $4)
        RETURNING id, subject_id, key_id, public_key_spki, created_at, revoked_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(subject_id)
    .bind(key_id)
    .bind(public_key_spki)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_conflict)?;

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(record)
}

pub async fn create_invitation(
    pool: &PgPool,
    owner_subject_id: Uuid,
    asset_id: AssetId,
    token_hash: &[u8; 32],
    expires_at: OffsetDateTime,
) -> Result<P2pInvitationRecord, DbError> {
    if expires_at <= OffsetDateTime::now_utc() {
        return Err(DbError::Conflict);
    }

    let ready = sqlx::query_as::<_, ReadyPublicationRow>(
        r#"
        SELECT p.id AS publication_id, p.lineage_id
          FROM p2p_publications p
          JOIN assets a ON a.id = p.asset_id
         WHERE p.asset_id = $1
           AND a.uploader_id = $2
           AND p.state = 'ready'
           AND p.confirmed_lineage_id = p.lineage_id
           AND p.external_publication_id IS NOT NULL
           AND p.external_confirmed_at IS NOT NULL
           AND p.manifest_digest_sha256 IS NOT NULL
           AND p.sealed_kek_id IS NOT NULL
           AND p.sealed_kek_version IS NOT NULL
           AND p.sealed_nonce IS NOT NULL
           AND p.sealed_wrapped_ck IS NOT NULL
           AND EXISTS (
                SELECT 1
                  FROM p2p_publication_outbox o
                 WHERE o.publication_id = p.id
                   AND o.lineage_id = p.lineage_id
                   AND o.delivery_state = 'delivered'
                   AND o.delivered_at IS NOT NULL
           )
        "#,
    )
    .bind(asset_id.0)
    .bind(owner_subject_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)?;

    sqlx::query_as::<_, P2pInvitationRecord>(
        r#"
        INSERT INTO p2p_invitations (
            id, asset_id, publication_id, lineage_id, owner_subject_id,
            token_hash, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, asset_id, publication_id, lineage_id, owner_subject_id,
                  expires_at, claimed_by_subject_id, claimed_device_id,
                  claimed_at, revoked_at, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(asset_id.0)
    .bind(ready.publication_id)
    .bind(ready.lineage_id)
    .bind(owner_subject_id)
    .bind(token_hash.as_slice())
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(map_unique_conflict)
}

pub async fn claim_invitation(
    pool: &PgPool,
    token_hash: &[u8; 32],
    viewer_subject_id: Uuid,
    device_id: Uuid,
    now: OffsetDateTime,
) -> Result<P2pClaimResult, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let mut invitation = lock_invitation(&mut tx, token_hash).await?;
    if invitation.revoked_at.is_some() || invitation.expires_at <= now {
        return Err(DbError::Conflict);
    }

    let device = active_device_for_viewer(&mut tx, device_id, viewer_subject_id).await?;
    ensure_publication_still_ready(&mut tx, &invitation).await?;

    match (invitation.claimed_by_subject_id, invitation.claimed_device_id) {
        (None, None) => {
            invitation = sqlx::query_as::<_, P2pInvitationRecord>(
                r#"
                UPDATE p2p_invitations
                   SET claimed_by_subject_id = $2,
                       claimed_device_id = $3,
                       claimed_at = $4,
                       updated_at = $4
                 WHERE id = $1
                RETURNING id, asset_id, publication_id, lineage_id, owner_subject_id,
                          expires_at, claimed_by_subject_id, claimed_device_id,
                          claimed_at, revoked_at, created_at, updated_at
                "#,
            )
            .bind(invitation.id)
            .bind(viewer_subject_id)
            .bind(device_id)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::QueryFailed)?;
        }
        (Some(existing_viewer), Some(existing_device))
            if existing_viewer == viewer_subject_id && existing_device == device_id => {}
        _ => return Err(DbError::Conflict),
    }

    let authorization = upsert_authorization(&mut tx, &invitation, viewer_subject_id, device_id)
        .await?;
    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(P2pClaimResult {
        invitation,
        authorization,
        device,
    })
}

pub async fn list_viewer_invitations(
    pool: &PgPool,
    viewer_subject_id: Uuid,
) -> Result<Vec<P2pInvitationRecord>, DbError> {
    sqlx::query_as::<_, P2pInvitationRecord>(
        r#"
        SELECT id, asset_id, publication_id, lineage_id, owner_subject_id,
               expires_at, claimed_by_subject_id, claimed_device_id,
               claimed_at, revoked_at, created_at, updated_at
          FROM p2p_invitations
         WHERE claimed_by_subject_id = $1
         ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(viewer_subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)
}

pub async fn get_active_authorization(
    pool: &PgPool,
    authorization_id: Uuid,
    viewer_subject_id: Uuid,
    now: OffsetDateTime,
) -> Result<P2pAudienceAuthorizationRecord, DbError> {
    sqlx::query_as::<_, P2pAudienceAuthorizationRecord>(
        r#"
        SELECT id, invitation_id, asset_id, publication_id, lineage_id,
               viewer_subject_id, device_id, expires_at, revoked_at,
               created_at, updated_at
          FROM p2p_audience_authorizations
         WHERE id = $1
           AND viewer_subject_id = $2
           AND revoked_at IS NULL
           AND expires_at > $3
        "#,
    )
    .bind(authorization_id)
    .bind(viewer_subject_id)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

async fn lock_invitation(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &[u8; 32],
) -> Result<P2pInvitationRecord, DbError> {
    sqlx::query_as::<_, P2pInvitationRecord>(
        r#"
        SELECT id, asset_id, publication_id, lineage_id, owner_subject_id,
               expires_at, claimed_by_subject_id, claimed_device_id,
               claimed_at, revoked_at, created_at, updated_at
          FROM p2p_invitations
         WHERE token_hash = $1
         FOR UPDATE
        "#,
    )
    .bind(token_hash.as_slice())
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

async fn active_device_for_viewer(
    tx: &mut Transaction<'_, Postgres>,
    device_id: Uuid,
    viewer_subject_id: Uuid,
) -> Result<P2pDeviceRecord, DbError> {
    sqlx::query_as::<_, P2pDeviceRecord>(
        r#"
        SELECT id, subject_id, key_id, public_key_spki, created_at, revoked_at
          FROM p2p_devices
         WHERE id = $1
           AND subject_id = $2
           AND revoked_at IS NULL
        "#,
    )
    .bind(device_id)
    .bind(viewer_subject_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

async fn ensure_publication_still_ready(
    tx: &mut Transaction<'_, Postgres>,
    invitation: &P2pInvitationRecord,
) -> Result<(), DbError> {
    let ready: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
              FROM p2p_publications p
             WHERE p.id = $1
               AND p.lineage_id = $2
               AND p.asset_id = $3
               AND p.state = 'ready'
               AND p.confirmed_lineage_id = p.lineage_id
               AND p.external_publication_id IS NOT NULL
               AND p.manifest_digest_sha256 IS NOT NULL
               AND EXISTS (
                    SELECT 1 FROM p2p_publication_outbox o
                     WHERE o.publication_id = p.id
                       AND o.lineage_id = p.lineage_id
                       AND o.delivery_state = 'delivered'
               )
        )
        "#,
    )
    .bind(invitation.publication_id)
    .bind(invitation.lineage_id)
    .bind(invitation.asset_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    if ready {
        Ok(())
    } else {
        Err(DbError::Conflict)
    }
}

async fn upsert_authorization(
    tx: &mut Transaction<'_, Postgres>,
    invitation: &P2pInvitationRecord,
    viewer_subject_id: Uuid,
    device_id: Uuid,
) -> Result<P2pAudienceAuthorizationRecord, DbError> {
    sqlx::query_as::<_, P2pAudienceAuthorizationRecord>(
        r#"
        INSERT INTO p2p_audience_authorizations (
            id, invitation_id, asset_id, publication_id, lineage_id,
            viewer_subject_id, device_id, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (invitation_id) DO UPDATE
           SET updated_at = p2p_audience_authorizations.updated_at
         WHERE p2p_audience_authorizations.viewer_subject_id = EXCLUDED.viewer_subject_id
           AND p2p_audience_authorizations.device_id = EXCLUDED.device_id
           AND p2p_audience_authorizations.revoked_at IS NULL
        RETURNING id, invitation_id, asset_id, publication_id, lineage_id,
                  viewer_subject_id, device_id, expires_at, revoked_at,
                  created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(invitation.id)
    .bind(invitation.asset_id)
    .bind(invitation.publication_id)
    .bind(invitation.lineage_id)
    .bind(viewer_subject_id)
    .bind(device_id)
    .bind(invitation.expires_at)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)
}

fn map_unique_conflict(error: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(database) = &error {
        if database.code().as_deref() == Some("23505") {
            return DbError::Conflict;
        }
    }
    DbError::QueryFailed(error)
}

impl P2pInvitationRecord {
    pub fn asset_id(&self) -> AssetId {
        AssetId(self.asset_id)
    }

    pub fn publication_id(&self) -> P2pPublicationId {
        P2pPublicationId(self.publication_id)
    }

    pub fn lineage_id(&self) -> K1LineageId {
        K1LineageId(self.lineage_id)
    }
}
