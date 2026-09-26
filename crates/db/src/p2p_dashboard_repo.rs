// MVP0-P2P P6 prerequisite: authoritative owner/viewer dashboard read models.

use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::DbError,
    p2p_audience_repo::{P2pAudienceAuthorizationRecord, P2pInvitationRecord},
};

#[derive(Debug, Clone)]
pub struct P2pOwnerContentRecord {
    pub asset_id: AssetId,
    pub asset_title: String,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub state: PublicationState,
}

#[derive(Debug, Clone)]
pub struct P2pViewerInboxRecord {
    pub invitation: P2pInvitationRecord,
    pub authorization: P2pAudienceAuthorizationRecord,
}

#[derive(sqlx::FromRow)]
struct OwnerContentRow {
    asset_id: Uuid,
    asset_title: String,
    publication_id: Uuid,
    lineage_id: Uuid,
    state: String,
}

#[derive(sqlx::FromRow)]
struct ViewerInboxRow {
    invitation_id: Uuid,
    invitation_asset_id: Uuid,
    invitation_publication_id: Uuid,
    invitation_lineage_id: Uuid,
    owner_subject_id: Uuid,
    invitation_expires_at: OffsetDateTime,
    claimed_by_subject_id: Option<Uuid>,
    claimed_device_id: Option<Uuid>,
    claimed_at: Option<OffsetDateTime>,
    invitation_revoked_at: Option<OffsetDateTime>,
    invitation_created_at: OffsetDateTime,
    invitation_updated_at: OffsetDateTime,
    authorization_id: Uuid,
    authorization_viewer_subject_id: Uuid,
    authorization_device_id: Uuid,
    authorization_expires_at: OffsetDateTime,
    authorization_revoked_at: Option<OffsetDateTime>,
    authorization_created_at: OffsetDateTime,
    authorization_updated_at: OffsetDateTime,
}

/// List only P2P publications whose backing asset belongs to this owner.
/// Asset ingestion status is deliberately not used as P2P publication state.
pub async fn list_owner_p2p_content(
    pool: &PgPool,
    owner_subject_id: Uuid,
) -> Result<Vec<P2pOwnerContentRecord>, DbError> {
    let rows = sqlx::query_as::<_, OwnerContentRow>(
        r#"
        SELECT p.asset_id,
               a.title AS asset_title,
               p.id AS publication_id,
               p.lineage_id,
               p.state
          FROM p2p_publications p
          JOIN assets a ON a.id = p.asset_id
         WHERE a.uploader_id = $1
         ORDER BY p.updated_at DESC, p.id DESC
        "#,
    )
    .bind(owner_subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter()
        .map(|row| {
            let state = row.state.parse().map_err(|_| DbError::UnknownStoredValue {
                field: "p2p_publications.state",
                value: row.state.clone(),
            })?;
            Ok(P2pOwnerContentRecord {
                asset_id: AssetId(row.asset_id),
                asset_title: row.asset_title,
                publication_id: P2pPublicationId(row.publication_id),
                lineage_id: K1LineageId(row.lineage_id),
                state,
            })
        })
        .collect()
}

/// Return claimed invitations together with their durable authorization for the
/// authenticated viewer. Expired/revoked rows remain visible for UI history;
/// playback must still revalidate through the active-authorization endpoint.
pub async fn list_viewer_p2p_inbox(
    pool: &PgPool,
    viewer_subject_id: Uuid,
) -> Result<Vec<P2pViewerInboxRecord>, DbError> {
    let rows = sqlx::query_as::<_, ViewerInboxRow>(
        r#"
        SELECT i.id AS invitation_id,
               i.asset_id AS invitation_asset_id,
               i.publication_id AS invitation_publication_id,
               i.lineage_id AS invitation_lineage_id,
               i.owner_subject_id,
               i.expires_at AS invitation_expires_at,
               i.claimed_by_subject_id,
               i.claimed_device_id,
               i.claimed_at,
               i.revoked_at AS invitation_revoked_at,
               i.created_at AS invitation_created_at,
               i.updated_at AS invitation_updated_at,
               a.id AS authorization_id,
               a.viewer_subject_id AS authorization_viewer_subject_id,
               a.device_id AS authorization_device_id,
               a.expires_at AS authorization_expires_at,
               a.revoked_at AS authorization_revoked_at,
               a.created_at AS authorization_created_at,
               a.updated_at AS authorization_updated_at
          FROM p2p_invitations i
          JOIN p2p_audience_authorizations a
            ON a.invitation_id = i.id
           AND a.viewer_subject_id = $1
         WHERE i.claimed_by_subject_id = $1
         ORDER BY i.created_at DESC, i.id DESC
        "#,
    )
    .bind(viewer_subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(rows.into_iter().map(viewer_inbox_from_row).collect())
}

fn viewer_inbox_from_row(row: ViewerInboxRow) -> P2pViewerInboxRecord {
    let invitation = P2pInvitationRecord {
        id: row.invitation_id,
        asset_id: row.invitation_asset_id,
        publication_id: row.invitation_publication_id,
        lineage_id: row.invitation_lineage_id,
        owner_subject_id: row.owner_subject_id,
        expires_at: row.invitation_expires_at,
        claimed_by_subject_id: row.claimed_by_subject_id,
        claimed_device_id: row.claimed_device_id,
        claimed_at: row.claimed_at,
        revoked_at: row.invitation_revoked_at,
        created_at: row.invitation_created_at,
        updated_at: row.invitation_updated_at,
    };
    let authorization = P2pAudienceAuthorizationRecord {
        id: row.authorization_id,
        invitation_id: row.invitation_id,
        asset_id: row.invitation_asset_id,
        publication_id: row.invitation_publication_id,
        lineage_id: row.invitation_lineage_id,
        viewer_subject_id: row.authorization_viewer_subject_id,
        device_id: row.authorization_device_id,
        expires_at: row.authorization_expires_at,
        revoked_at: row.authorization_revoked_at,
        created_at: row.authorization_created_at,
        updated_at: row.authorization_updated_at,
    };
    P2pViewerInboxRecord {
        invitation,
        authorization,
    }
}
