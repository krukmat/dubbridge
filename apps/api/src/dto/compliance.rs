use dubbridge_domain::{
    audit::{AuditEvent, AuditEventKind},
    consent::{ConsentRow, ConsentScope, ConsentStatus},
    rights::{LicenseType, RightsRecord, SourceType},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuditEventResponse {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub event_kind: AuditEventKind,
    pub ingest_token: Option<Uuid>,
    pub recording_session_id: Option<Uuid>,
    pub platform_ingest_session_id: Option<Uuid>,
    pub detail: Option<String>,
    pub happened_at: OffsetDateTime,
}

impl From<AuditEvent> for AuditEventResponse {
    fn from(value: AuditEvent) -> Self {
        Self {
            id: value.id,
            asset_id: value.asset_id.map(|asset_id| asset_id.0),
            event_kind: value.event_kind,
            ingest_token: value.ingest_token,
            recording_session_id: value.recording_session_id,
            platform_ingest_session_id: value.platform_ingest_session_id,
            detail: value.detail,
            happened_at: value.happened_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuditTimelineResponse {
    pub asset_id: Uuid,
    pub events: Vec<AuditEventResponse>,
}

#[derive(Debug, Serialize)]
pub struct RightsRecordResponse {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub owner: String,
    pub license_type: LicenseType,
    pub source_type: SourceType,
    pub proof_reference: String,
    pub created_at: OffsetDateTime,
}

impl From<RightsRecord> for RightsRecordResponse {
    fn from(value: RightsRecord) -> Self {
        Self {
            id: value.id,
            asset_id: value.asset_id.0,
            owner: value.owner,
            license_type: value.license_type,
            source_type: value.source_type,
            proof_reference: value.proof_reference,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RightsLedgerResponse {
    pub asset_id: Uuid,
    pub entries: Vec<RightsRecordResponse>,
}

#[derive(Debug, Serialize)]
pub struct ConsentRowResponse {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub scope: ConsentScope,
    pub status: ConsentStatus,
    pub evidence_ref: Option<String>,
    pub granted_by: Uuid,
    pub happened_at: OffsetDateTime,
}

impl From<ConsentRow> for ConsentRowResponse {
    fn from(value: ConsentRow) -> Self {
        Self {
            id: value.id,
            asset_id: value.asset_id.0,
            scope: value.scope,
            status: value.status,
            evidence_ref: value.evidence_ref,
            granted_by: value.granted_by,
            happened_at: value.happened_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConsentLedgerResponse {
    pub asset_id: Uuid,
    pub current_status: Option<ConsentStatus>,
    pub rows: Vec<ConsentRowResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ConsentMutationRequest {
    pub asset_id: Uuid,
    pub scope: ConsentScope,
    pub status: ConsentStatus,
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConsentMutationResponse {
    pub asset_id: Uuid,
    pub scope: ConsentScope,
    pub current_status: ConsentStatus,
    pub happened_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::{
        asset::AssetId,
        audit::{AuditEvent, AuditEventKind},
    };

    #[test]
    fn audit_event_response_maps_optional_fields() {
        let asset_id = AssetId::new();
        let event = AuditEvent::new_consent(
            asset_id,
            AuditEventKind::ConsentGranted,
            Some("scope=voice_clone".to_string()),
        );

        let response = AuditEventResponse::from(event);
        assert_eq!(response.asset_id, Some(asset_id.0));
        assert_eq!(response.event_kind, AuditEventKind::ConsentGranted);
        assert_eq!(response.detail.as_deref(), Some("scope=voice_clone"));
    }

    #[test]
    fn rights_record_response_maps_domain_fields() {
        let asset_id = AssetId::new();
        let record = RightsRecord::new(
            asset_id,
            &dubbridge_domain::rights::RightsBasis {
                owner: "Acme".to_string(),
                license_type: LicenseType::CreativeCommons,
                source_type: SourceType::LicensedSource,
                proof_reference: "proof-001".to_string(),
            },
        );

        let response = RightsRecordResponse::from(record);
        assert_eq!(response.asset_id, asset_id.0);
        assert_eq!(response.owner, "Acme");
        assert_eq!(response.license_type, LicenseType::CreativeCommons);
    }

    #[test]
    fn consent_row_response_maps_status_and_evidence() {
        let row = dubbridge_domain::consent::new_grant(
            AssetId::new(),
            ConsentScope::VoiceClone,
            "proof-xyz",
            Uuid::new_v4(),
        )
        .expect("grant row");

        let response = ConsentRowResponse::from(row);
        assert_eq!(response.scope, ConsentScope::VoiceClone);
        assert_eq!(response.status, ConsentStatus::Grant);
        assert_eq!(response.evidence_ref.as_deref(), Some("proof-xyz"));
    }
}
