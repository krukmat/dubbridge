// MVP0-P2P P2.T5c: authoritative P2P_READY handoff contract.

use serde::{Deserialize, Serialize};

use crate::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId},
};

pub const P2P_READY_DESCRIPTOR_VERSION: &str = "p2p-ready-descriptor-v1";
pub const P2P_MANIFEST_VERSION: &str = "p2p-manifest-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct P2pReadyDescriptor {
    pub descriptor_version: String,
    pub asset_id: AssetId,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub manifest_version: String,
    pub manifest_digest_sha256: String,
    pub external_publication_id: String,
    pub ck_wrap_ref: String,
    pub kek_id: String,
    pub kek_version: i32,
    pub ready_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pReadyDescriptorInput {
    pub asset_id: AssetId,
    pub publication_id: P2pPublicationId,
    pub lineage_id: K1LineageId,
    pub manifest_digest_sha256: String,
    pub external_publication_id: String,
    pub kek_id: String,
    pub kek_version: i32,
    pub ready_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum P2pReadyDescriptorError {
    #[error("invalid manifest digest")]
    InvalidManifestDigest,
    #[error("missing external publication id")]
    MissingExternalPublicationId,
    #[error("missing KEK id")]
    MissingKekId,
    #[error("invalid KEK version")]
    InvalidKekVersion,
    #[error("missing ready timestamp")]
    MissingReadyAt,
}

impl TryFrom<P2pReadyDescriptorInput> for P2pReadyDescriptor {
    type Error = P2pReadyDescriptorError;

    fn try_from(input: P2pReadyDescriptorInput) -> Result<Self, Self::Error> {
        if !is_lower_hex_sha256(&input.manifest_digest_sha256) {
            return Err(P2pReadyDescriptorError::InvalidManifestDigest);
        }
        if input.external_publication_id.trim().is_empty() {
            return Err(P2pReadyDescriptorError::MissingExternalPublicationId);
        }
        if input.kek_id.trim().is_empty() {
            return Err(P2pReadyDescriptorError::MissingKekId);
        }
        if input.kek_version <= 0 {
            return Err(P2pReadyDescriptorError::InvalidKekVersion);
        }
        if input.ready_at.trim().is_empty() {
            return Err(P2pReadyDescriptorError::MissingReadyAt);
        }

        Ok(Self {
            descriptor_version: P2P_READY_DESCRIPTOR_VERSION.to_owned(),
            asset_id: input.asset_id,
            publication_id: input.publication_id,
            lineage_id: input.lineage_id,
            manifest_version: P2P_MANIFEST_VERSION.to_owned(),
            manifest_digest_sha256: input.manifest_digest_sha256,
            external_publication_id: input.external_publication_id,
            ck_wrap_ref: format!("p2p-k1-wrap/{}/{}", input.publication_id, input.lineage_id),
            kek_id: input.kek_id,
            kek_version: input.kek_version,
            ready_at: input.ready_at,
        })
    }
}

pub fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn input() -> P2pReadyDescriptorInput {
        P2pReadyDescriptorInput {
            asset_id: AssetId(Uuid::new_v4()),
            publication_id: P2pPublicationId::new(),
            lineage_id: K1LineageId::new(),
            manifest_digest_sha256:
                "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4".to_owned(),
            external_publication_id: "hyperdrive:stable-key".to_owned(),
            kek_id: "server-kek".to_owned(),
            kek_version: 1,
            ready_at: "2026-09-14T09:30:00.000000Z".to_owned(),
        }
    }

    #[test]
    fn descriptor_contains_only_opaque_key_reference() {
        let descriptor = P2pReadyDescriptor::try_from(input()).expect("valid descriptor");
        assert_eq!(descriptor.descriptor_version, P2P_READY_DESCRIPTOR_VERSION);
        assert_eq!(descriptor.manifest_version, P2P_MANIFEST_VERSION);
        assert!(descriptor.ck_wrap_ref.starts_with("p2p-k1-wrap/"));
    }

    #[test]
    fn invalid_digest_fails_closed() {
        let mut candidate = input();
        candidate.manifest_digest_sha256 = "Z".repeat(64);
        assert_eq!(
            P2pReadyDescriptor::try_from(candidate),
            Err(P2pReadyDescriptorError::InvalidManifestDigest)
        );
    }
}
