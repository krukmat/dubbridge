// T1: S1 domain — finalize ingestion command with fail-closed rights validation
use thiserror::Error;
use uuid::Uuid;

use crate::rights::RightsBasis;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IngestionError {
    #[error("rights basis is required to finalize ingestion")]
    MissingRightsBasis,
    #[error("rights owner field is required")]
    MissingRightsOwner,
    #[error("license type is required")]
    MissingLicenseType,
    #[error("source type is required")]
    MissingSourceType,
    #[error("proof reference is required")]
    MissingProofReference,
    #[error("uploader identity is required to finalize ingestion")]
    MissingUploaderContext,
}

#[derive(Debug)]
pub struct FinalizeIngestionCommand {
    pub ingest_token: Uuid,
    pub uploader_id: Option<Uuid>,
    pub rights_basis: Option<RightsBasis>,
    pub asset_title: String,
    pub file_key: String,
    pub file_size_bytes: i64,
    pub content_type: String,
}

impl FinalizeIngestionCommand {
    /// Validates all mandatory fields. Rights are checked before uploader context
    /// because rights are the primary legal precondition per ADR-008.
    pub fn validate(&self) -> Result<(), IngestionError> {
        match &self.rights_basis {
            None => return Err(IngestionError::MissingRightsBasis),
            Some(basis) => {
                if basis.owner.trim().is_empty() {
                    return Err(IngestionError::MissingRightsOwner);
                }
                if basis.proof_reference.trim().is_empty() {
                    return Err(IngestionError::MissingProofReference);
                }
            }
        }

        if self.uploader_id.is_none() {
            return Err(IngestionError::MissingUploaderContext);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rights::{LicenseType, RightsBasis, SourceType};

    fn valid_basis() -> RightsBasis {
        RightsBasis {
            owner: "Acme Studios".to_string(),
            license_type: LicenseType::LicensedDistribution,
            source_type: SourceType::DirectUpload,
            proof_reference: "contract-2024-001".to_string(),
        }
    }

    fn valid_command() -> FinalizeIngestionCommand {
        FinalizeIngestionCommand {
            ingest_token: Uuid::new_v4(),
            uploader_id: Some(Uuid::new_v4()),
            rights_basis: Some(valid_basis()),
            asset_title: "Test Video".to_string(),
            file_key: "assets/test.mp4".to_string(),
            file_size_bytes: 1024,
            content_type: "video/mp4".to_string(),
        }
    }

    #[test]
    fn validate_accepts_valid_command() {
        assert_eq!(valid_command().validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_missing_rights_basis() {
        let cmd = FinalizeIngestionCommand {
            rights_basis: None,
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(IngestionError::MissingRightsBasis));
    }

    #[test]
    fn validate_rejects_empty_owner() {
        let mut basis = valid_basis();
        basis.owner = "  ".to_string();
        let cmd = FinalizeIngestionCommand {
            rights_basis: Some(basis),
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(IngestionError::MissingRightsOwner));
    }

    #[test]
    fn validate_rejects_empty_proof_reference() {
        let mut basis = valid_basis();
        basis.proof_reference = String::new();
        let cmd = FinalizeIngestionCommand {
            rights_basis: Some(basis),
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(IngestionError::MissingProofReference));
    }

    #[test]
    fn validate_rejects_missing_uploader_context() {
        let cmd = FinalizeIngestionCommand {
            uploader_id: None,
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(IngestionError::MissingUploaderContext));
    }
}
