// T1: S1 domain — rights ledger record and basis fields per ADR-008
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

/// Allowed ingestion source types per ADR-008. Excluded sources are not represented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    DirectUpload,
    AuthorizedS3,
    InternalFeed,
    LicensedSource,
    PublicDomainWithProof,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DirectUpload => "direct_upload",
            Self::AuthorizedS3 => "authorized_s3",
            Self::InternalFeed => "internal_feed",
            Self::LicensedSource => "licensed_source",
            Self::PublicDomainWithProof => "public_domain_with_proof",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseType {
    AllRightsReserved,
    CreativeCommons,
    PublicDomain,
    LicensedDistribution,
    InternalOnly,
}

impl std::fmt::Display for LicenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AllRightsReserved => "all_rights_reserved",
            Self::CreativeCommons => "creative_commons",
            Self::PublicDomain => "public_domain",
            Self::LicensedDistribution => "licensed_distribution",
            Self::InternalOnly => "internal_only",
        };
        write!(f, "{s}")
    }
}

/// Minimum rights basis required at ingestion time per ADR-008.
/// Fields deferred to later slices: allowed_territories, allowed_languages,
/// expiration_date, dubbing_permission, voice_cloning_permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsBasis {
    pub owner: String,
    pub license_type: LicenseType,
    pub source_type: SourceType,
    pub proof_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsRecord {
    pub id: Uuid,
    pub asset_id: AssetId,
    pub owner: String,
    pub license_type: LicenseType,
    pub source_type: SourceType,
    pub proof_reference: String,
    pub created_at: OffsetDateTime,
}

impl RightsRecord {
    pub fn new(asset_id: AssetId, basis: &RightsBasis) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            owner: basis.owner.clone(),
            license_type: basis.license_type.clone(),
            source_type: basis.source_type.clone(),
            proof_reference: basis.proof_reference.clone(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1-T3: Display impls are used in audit logs; verify all variants serialize correctly
    #[test]
    fn source_type_display_all_variants() {
        assert_eq!(SourceType::DirectUpload.to_string(), "direct_upload");
        assert_eq!(SourceType::AuthorizedS3.to_string(), "authorized_s3");
        assert_eq!(SourceType::InternalFeed.to_string(), "internal_feed");
        assert_eq!(SourceType::LicensedSource.to_string(), "licensed_source");
        assert_eq!(
            SourceType::PublicDomainWithProof.to_string(),
            "public_domain_with_proof"
        );
    }

    #[test]
    fn license_type_display_all_variants() {
        assert_eq!(
            LicenseType::AllRightsReserved.to_string(),
            "all_rights_reserved"
        );
        assert_eq!(LicenseType::CreativeCommons.to_string(), "creative_commons");
        assert_eq!(LicenseType::PublicDomain.to_string(), "public_domain");
        assert_eq!(
            LicenseType::LicensedDistribution.to_string(),
            "licensed_distribution"
        );
        assert_eq!(LicenseType::InternalOnly.to_string(), "internal_only");
    }

    #[test]
    fn rights_record_new_copies_basis_fields() {
        let asset_id = AssetId::new();
        let basis = RightsBasis {
            owner: "Acme Corp".to_string(),
            license_type: LicenseType::AllRightsReserved,
            source_type: SourceType::DirectUpload,
            proof_reference: "ref-001".to_string(),
        };
        let record = RightsRecord::new(asset_id, &basis);
        assert_eq!(record.owner, "Acme Corp");
        assert_eq!(record.proof_reference, "ref-001");
        assert_eq!(record.asset_id.0, asset_id.0);
    }
}
