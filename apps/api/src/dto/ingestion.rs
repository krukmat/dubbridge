use dubbridge_domain::{
    asset::{Asset, IngestionStatus},
    rights::{LicenseType, RightsBasis, SourceType},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct IngestSessionResponse {
    pub ingest_token: Uuid,
    pub title: String,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRightsRequest {
    pub owner: String,
    pub license_type: LicenseType,
    pub source_type: SourceType,
    pub proof_reference: String,
}

impl From<SubmitRightsRequest> for RightsBasis {
    fn from(value: SubmitRightsRequest) -> Self {
        Self {
            owner: value.owner,
            license_type: value.license_type,
            source_type: value.source_type,
            proof_reference: value.proof_reference,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RightsSubmissionResponse {
    pub ingest_token: Uuid,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AssetSummaryResponse {
    pub id: Uuid,
    pub title: String,
    pub uploader_id: Uuid,
    pub status: IngestionStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Asset> for AssetSummaryResponse {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id.0,
            title: asset.title,
            uploader_id: asset.uploader_id,
            status: asset.status,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        }
    }
}
