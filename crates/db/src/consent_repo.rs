// S-110-T1c: append-only voice-consent repository per ADR-028
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::asset::AssetId;
use dubbridge_domain::consent::{ConsentRow, ConsentScope, ConsentStatus};

use crate::error::DbError;

#[derive(sqlx::FromRow)]
struct ConsentRowDb {
    id: Uuid,
    asset_id: Uuid,
    scope: String,
    status: String,
    evidence_ref: Option<String>,
    granted_by: Uuid,
    happened_at: OffsetDateTime,
}

fn parse_scope(s: &str) -> Result<ConsentScope, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "voice_consents.scope",
        value: s.to_owned(),
    })
}

fn parse_status(s: &str) -> Result<ConsentStatus, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "voice_consents.status",
        value: s.to_owned(),
    })
}

fn row_from_db(r: ConsentRowDb) -> Result<ConsentRow, DbError> {
    Ok(ConsentRow {
        id: r.id,
        asset_id: AssetId(r.asset_id),
        scope: parse_scope(&r.scope)?,
        status: parse_status(&r.status)?,
        evidence_ref: r.evidence_ref,
        granted_by: r.granted_by,
        happened_at: r.happened_at,
    })
}

/// Append a consent row (grant or revoke). INSERT only — no upsert (ADR-028).
pub async fn append_consent(pool: &PgPool, row: &ConsentRow) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO voice_consents (id, asset_id, scope, status, evidence_ref, granted_by, happened_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(row.id)
    .bind(row.asset_id.0)
    .bind(row.scope.to_string())
    .bind(row.status.to_string())
    .bind(&row.evidence_ref)
    .bind(row.granted_by)
    .bind(row.happened_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

/// Return the current consent status for an asset+scope: latest row by `happened_at DESC`.
/// Returns `None` if no consent row exists for the given asset and scope.
pub async fn latest_consent_status(
    pool: &PgPool,
    asset_id: AssetId,
    scope: &ConsentScope,
) -> Result<Option<ConsentStatus>, DbError> {
    let row = sqlx::query_as::<_, ConsentRowDb>(
        r#"
        SELECT id, asset_id, scope, status, evidence_ref, granted_by, happened_at
        FROM voice_consents
        WHERE asset_id = $1 AND scope = $2
        ORDER BY happened_at DESC
        LIMIT 1
        "#,
    )
    .bind(asset_id.0)
    .bind(scope.to_string())
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| parse_status(&r.status)).transpose()
}

/// Return all consent rows for an asset ordered `happened_at ASC` (full ledger view).
pub async fn list_consents_for_asset(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Vec<ConsentRow>, DbError> {
    let rows = sqlx::query_as::<_, ConsentRowDb>(
        r#"
        SELECT id, asset_id, scope, status, evidence_ref, granted_by, happened_at
        FROM voice_consents
        WHERE asset_id = $1
        ORDER BY happened_at ASC
        "#,
    )
    .bind(asset_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(row_from_db).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // parse_scope must succeed for every CHECK-constraint value and fail closed for unknown.
    #[test]
    fn parse_scope_known_variants_succeed() {
        assert!(matches!(
            parse_scope("voice_clone"),
            Ok(ConsentScope::VoiceClone)
        ));
        assert!(matches!(
            parse_scope("tts_synthesis"),
            Ok(ConsentScope::TtsSynthesis)
        ));
    }

    #[test]
    fn parse_scope_unknown_value_fails_closed() {
        let err = parse_scope("dubbing").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "voice_consents.scope",
                ..
            }
        ));
        assert!(err.to_string().contains("dubbing"));
    }

    // parse_status must succeed for every CHECK-constraint value and fail closed for unknown.
    #[test]
    fn parse_status_known_variants_succeed() {
        assert!(matches!(parse_status("grant"), Ok(ConsentStatus::Grant)));
        assert!(matches!(parse_status("revoke"), Ok(ConsentStatus::Revoke)));
    }

    #[test]
    fn parse_status_unknown_value_fails_closed() {
        let err = parse_status("pending").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "voice_consents.status",
                ..
            }
        ));
        assert!(err.to_string().contains("pending"));
    }

    // row_from_db must propagate parse errors fail-closed (unknown scope or status in DB).
    #[test]
    fn row_from_db_unknown_scope_fails_closed() {
        let db_row = ConsentRowDb {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            scope: "dubbing".to_string(),
            status: "grant".to_string(),
            evidence_ref: None,
            granted_by: Uuid::new_v4(),
            happened_at: OffsetDateTime::now_utc(),
        };
        let err = row_from_db(db_row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "voice_consents.scope",
                ..
            }
        ));
    }

    #[test]
    fn row_from_db_unknown_status_fails_closed() {
        let db_row = ConsentRowDb {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            scope: "voice_clone".to_string(),
            status: "pending".to_string(),
            evidence_ref: None,
            granted_by: Uuid::new_v4(),
            happened_at: OffsetDateTime::now_utc(),
        };
        let err = row_from_db(db_row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "voice_consents.status",
                ..
            }
        ));
    }

    // row_from_db round-trips a valid grant row with evidence_ref preserved.
    #[test]
    fn row_from_db_valid_grant_round_trips() {
        let asset_uuid = Uuid::new_v4();
        let db_row = ConsentRowDb {
            id: Uuid::new_v4(),
            asset_id: asset_uuid,
            scope: "voice_clone".to_string(),
            status: "grant".to_string(),
            evidence_ref: Some("ref-001".to_string()),
            granted_by: Uuid::new_v4(),
            happened_at: OffsetDateTime::now_utc(),
        };
        let row = row_from_db(db_row).unwrap();
        assert_eq!(row.asset_id.0, asset_uuid);
        assert_eq!(row.scope, ConsentScope::VoiceClone);
        assert_eq!(row.status, ConsentStatus::Grant);
        assert_eq!(row.evidence_ref.as_deref(), Some("ref-001"));
    }

    // row_from_db valid revoke row has no evidence_ref.
    #[test]
    fn row_from_db_valid_revoke_has_no_evidence_ref() {
        let db_row = ConsentRowDb {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            scope: "tts_synthesis".to_string(),
            status: "revoke".to_string(),
            evidence_ref: None,
            granted_by: Uuid::new_v4(),
            happened_at: OffsetDateTime::now_utc(),
        };
        let row = row_from_db(db_row).unwrap();
        assert_eq!(row.status, ConsentStatus::Revoke);
        assert!(row.evidence_ref.is_none());
    }
}
