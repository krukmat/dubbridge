use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct P2pEnvelopeReleaseContext {
    pub authorization_id: Uuid,
    pub invitation_id: Uuid,
    pub asset_id: Uuid,
    pub publication_id: Uuid,
    pub lineage_id: Uuid,
    pub viewer_subject_id: Uuid,
    pub device_id: Uuid,
    pub device_key_id: String,
    pub device_public_key_spki: Vec<u8>,
    pub expires_at: OffsetDateTime,
    pub sealed_kek_id: String,
    pub sealed_kek_version: i32,
    pub sealed_nonce: Vec<u8>,
    pub sealed_wrapped_ck: Vec<u8>,
}

/// Resolve every predicate required before a K1 device envelope may be released.
/// The join intentionally fails closed if invitation, viewer, device, publication
/// lineage, readiness, sealed key material, or durable delivery evidence drift.
pub async fn get_envelope_release_context(
    pool: &PgPool,
    authorization_id: Uuid,
    viewer_subject_id: Uuid,
    now: OffsetDateTime,
) -> Result<P2pEnvelopeReleaseContext, DbError> {
    sqlx::query_as::<_, P2pEnvelopeReleaseContext>(
        r#"
        SELECT
            auth.id AS authorization_id,
            auth.invitation_id,
            auth.asset_id,
            auth.publication_id,
            auth.lineage_id,
            auth.viewer_subject_id,
            auth.device_id,
            device.key_id AS device_key_id,
            device.public_key_spki AS device_public_key_spki,
            auth.expires_at,
            publication.sealed_kek_id,
            publication.sealed_kek_version,
            publication.sealed_nonce,
            publication.sealed_wrapped_ck
          FROM p2p_audience_authorizations auth
          JOIN p2p_invitations invitation
            ON invitation.id = auth.invitation_id
           AND invitation.asset_id = auth.asset_id
           AND invitation.publication_id = auth.publication_id
           AND invitation.lineage_id = auth.lineage_id
           AND invitation.claimed_by_subject_id = auth.viewer_subject_id
           AND invitation.claimed_device_id = auth.device_id
          JOIN p2p_devices device
            ON device.id = auth.device_id
           AND device.subject_id = auth.viewer_subject_id
          JOIN p2p_publications publication
            ON publication.id = auth.publication_id
           AND publication.asset_id = auth.asset_id
           AND publication.lineage_id = auth.lineage_id
         WHERE auth.id = $1
           AND auth.viewer_subject_id = $2
           AND auth.revoked_at IS NULL
           AND auth.expires_at > $3
           AND invitation.revoked_at IS NULL
           AND invitation.expires_at > $3
           AND invitation.claimed_at IS NOT NULL
           AND device.revoked_at IS NULL
           AND publication.state = 'ready'
           AND publication.confirmed_lineage_id = publication.lineage_id
           AND publication.external_publication_id IS NOT NULL
           AND publication.external_confirmed_at IS NOT NULL
           AND publication.manifest_digest_sha256 IS NOT NULL
           AND publication.sealed_kek_id IS NOT NULL
           AND publication.sealed_kek_version IS NOT NULL
           AND publication.sealed_nonce IS NOT NULL
           AND publication.sealed_wrapped_ck IS NOT NULL
           AND EXISTS (
                SELECT 1
                  FROM p2p_publication_outbox outbox
                 WHERE outbox.publication_id = publication.id
                   AND outbox.lineage_id = publication.lineage_id
                   AND outbox.delivery_state = 'delivered'
                   AND outbox.delivered_at IS NOT NULL
           )
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
