// MVP0-P2P P2.T6b: durable package-seal evidence for the exact P2 lineage.
// The migration-owned audit trigger records `p2p_lineage_sealed` in this same
// transaction when evidence is attached for the first time.

use dubbridge_domain::p2p_publication::{K1LineageId, P2pPublicationId};
use sqlx::PgPool;

use crate::error::DbError;

pub async fn persist_sealed_package_evidence(
    pool: &PgPool,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    manifest_digest_sha256: &str,
    package_ref: &str,
) -> Result<(), DbError> {
    if !is_lower_hex_sha256(manifest_digest_sha256) || package_ref.trim().is_empty() {
        return Err(DbError::Conflict);
    }

    let result = sqlx::query(
        r#"
        UPDATE p2p_publications
           SET manifest_digest_sha256 = $3,
               package_ref = $4,
               -- K1 is persisted provisionally before package construction so a
               -- crash cannot rotate the lineage key. Re-assert the complete
               -- sealed tuple in this seal-completion write so the frozen ADR-018
               -- `p2p_lineage_sealed` boundary commits wrapped-key identity and
               -- package evidence together with the audit trigger.
               sealed_kek_id = sealed_kek_id,
               sealed_kek_version = sealed_kek_version,
               sealed_nonce = sealed_nonce,
               sealed_wrapped_ck = sealed_wrapped_ck,
               sealed_at = sealed_at,
               updated_at = now()
         WHERE id = $1
           AND lineage_id = $2
           AND state = 'building'
           AND sealed_kek_id IS NOT NULL
           AND sealed_kek_version IS NOT NULL
           AND sealed_nonce IS NOT NULL
           AND sealed_wrapped_ck IS NOT NULL
           AND sealed_at IS NOT NULL
           AND (manifest_digest_sha256 IS NULL OR manifest_digest_sha256 = $3)
           AND (package_ref IS NULL OR package_ref = $4)
        "#,
    )
    .bind(publication_id.0)
    .bind(lineage_id.0)
    .bind(manifest_digest_sha256)
    .bind(package_ref)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    if result.rows_affected() == 1 {
        return Ok(());
    }

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM p2p_publications WHERE id = $1)")
            .bind(publication_id.0)
            .fetch_one(pool)
            .await
            .map_err(DbError::QueryFailed)?;

    if exists {
        Err(DbError::Conflict)
    } else {
        Err(DbError::NotFound)
    }
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::is_lower_hex_sha256;

    #[test]
    fn digest_validation_is_strict_lower_hex_sha256() {
        assert!(is_lower_hex_sha256(&"a5".repeat(32)));
        assert!(!is_lower_hex_sha256(&"A5".repeat(32)));
        assert!(!is_lower_hex_sha256("abc"));
    }
}
