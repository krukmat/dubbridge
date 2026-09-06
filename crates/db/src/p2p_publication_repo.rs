// MVP0-P2P P2.T1c: atomic publication + outbox persistence (ADR-044/O4).
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};

use crate::error::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pPublicationRecord {
    pub id: P2pPublicationId,
    pub asset_id: AssetId,
    pub lineage_id: K1LineageId,
    pub state: PublicationState,
    pub external_publication_id: Option<String>,
    pub confirmed_lineage_id: Option<K1LineageId>,
    pub external_confirmed_at: Option<OffsetDateTime>,
    pub failure_detail: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pOutboxRecord {
    pub id: Uuid,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub delivery_state: String,
    pub attempt_count: i32,
    pub available_at: OffsetDateTime,
    pub claimed_at: Option<OffsetDateTime>,
    pub delivered_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsurePublicationResult {
    pub publication: P2pPublicationRecord,
    pub outbox: P2pOutboxRecord,
    pub created: bool,
}

#[derive(sqlx::FromRow)]
struct PublicationRow {
    id: Uuid,
    asset_id: Uuid,
    lineage_id: Uuid,
    state: String,
    external_publication_id: Option<String>,
    confirmed_lineage_id: Option<Uuid>,
    external_confirmed_at: Option<OffsetDateTime>,
    failure_detail: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    delivery_state: String,
    attempt_count: i32,
    available_at: OffsetDateTime,
    claimed_at: Option<OffsetDateTime>,
    delivered_at: Option<OffsetDateTime>,
    last_error: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn publication_from_row(row: PublicationRow) -> Result<P2pPublicationRecord, DbError> {
    let state = row
        .state
        .parse()
        .map_err(|_| DbError::UnknownStoredValue {
            field: "p2p_publications.state",
            value: row.state.clone(),
        })?;

    Ok(P2pPublicationRecord {
        id: P2pPublicationId(row.id),
        asset_id: AssetId(row.asset_id),
        lineage_id: K1LineageId(row.lineage_id),
        state,
        external_publication_id: row.external_publication_id,
        confirmed_lineage_id: row.confirmed_lineage_id.map(K1LineageId),
        external_confirmed_at: row.external_confirmed_at,
        failure_detail: row.failure_detail,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn outbox_from_row(row: OutboxRow) -> P2pOutboxRecord {
    P2pOutboxRecord {
        id: row.id,
        publication_id: P2pPublicationId(row.publication_id),
        lineage_id: K1LineageId(row.lineage_id),
        delivery_state: row.delivery_state,
        attempt_count: row.attempt_count,
        available_at: row.available_at,
        claimed_at: row.claimed_at,
        delivered_at: row.delivered_at,
        last_error: row.last_error,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Atomically create (or idempotently ensure) one logical publication and its
/// durable outbox obligation. A pre-existing publication for the asset is valid
/// only when it carries the exact requested lineage; a different lineage is an
/// explicit conflict, never an implicit replacement.
pub async fn ensure_publication_with_outbox(
    pool: &PgPool,
    asset_id: AssetId,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    outbox_id: Uuid,
) -> Result<EnsurePublicationResult, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;

    let inserted = sqlx::query_as::<_, PublicationRow>(
        r#"
        INSERT INTO p2p_publications (id, asset_id, lineage_id, state)
        VALUES ($1, $2, $3, 'building')
        ON CONFLICT (asset_id) DO NOTHING
        RETURNING id, asset_id, lineage_id, state,
                  external_publication_id, confirmed_lineage_id, external_confirmed_at,
                  failure_detail, created_at, updated_at
        "#,
    )
    .bind(publication_id.0)
    .bind(asset_id.0)
    .bind(lineage_id.0)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    let created = inserted.is_some();
    let publication_row = match inserted {
        Some(row) => row,
        None => sqlx::query_as::<_, PublicationRow>(
            r#"
            SELECT id, asset_id, lineage_id, state,
                   external_publication_id, confirmed_lineage_id, external_confirmed_at,
                   failure_detail, created_at, updated_at
              FROM p2p_publications
             WHERE asset_id = $1
             FOR UPDATE
            "#,
        )
        .bind(asset_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(DbError::QueryFailed)?,
    };

    if publication_row.lineage_id != lineage_id.0 {
        return Err(DbError::Conflict);
    }

    sqlx::query(
        r#"
        INSERT INTO p2p_publication_outbox (id, publication_id, lineage_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (publication_id) DO NOTHING
        "#,
    )
    .bind(outbox_id)
    .bind(publication_row.id)
    .bind(publication_row.lineage_id)
    .execute(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    let outbox_row = sqlx::query_as::<_, OutboxRow>(
        r#"
        SELECT id, publication_id, lineage_id, delivery_state, attempt_count,
               available_at, claimed_at, delivered_at, last_error, created_at, updated_at
          FROM p2p_publication_outbox
         WHERE publication_id = $1
        "#,
    )
    .bind(publication_row.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    if outbox_row.lineage_id != publication_row.lineage_id {
        return Err(DbError::Conflict);
    }

    tx.commit().await.map_err(DbError::QueryFailed)?;

    Ok(EnsurePublicationResult {
        publication: publication_from_row(publication_row)?,
        outbox: outbox_from_row(outbox_row),
        created,
    })
}
