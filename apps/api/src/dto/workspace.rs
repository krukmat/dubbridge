use dubbridge_domain::workspace::{OrgMember, Organization, Project, TargetLanguage};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::ingestion::AssetSummaryResponse;

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Organization> for OrganizationResponse {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id.0,
            name: org.name,
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub subject_id: Uuid,
    pub role: dubbridge_domain::workspace::OrgRole,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct OrgMemberResponse {
    pub org_id: Uuid,
    pub subject_id: Uuid,
    pub role: dubbridge_domain::workspace::OrgRole,
    pub joined_at: OffsetDateTime,
}

impl From<OrgMember> for OrgMemberResponse {
    fn from(member: OrgMember) -> Self {
        Self {
            org_id: member.org_id.0,
            subject_id: member.subject_id,
            role: member.role,
            joined_at: member.joined_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub asset_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.0,
            org_id: project.org_id.0,
            name: project.name,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkProjectAssetRequest {
    pub asset_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SetTargetLanguagesRequest {
    pub source_lang: String,
    pub target_languages: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct TargetLanguageResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: OffsetDateTime,
}

impl From<TargetLanguage> for TargetLanguageResponse {
    fn from(value: TargetLanguage) -> Self {
        Self {
            id: value.id,
            project_id: value.project_id.0,
            source_lang: value.source_lang,
            target_lang: value.target_lang,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProjectDetailResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub assets: Vec<AssetSummaryResponse>,
    pub target_languages: Vec<TargetLanguageResponse>,
}

impl ProjectDetailResponse {
    pub fn new(
        project: Project,
        assets: Vec<AssetSummaryResponse>,
        target_languages: Vec<TargetLanguageResponse>,
    ) -> Self {
        Self {
            id: project.id.0,
            org_id: project.org_id.0,
            name: project.name,
            created_at: project.created_at,
            updated_at: project.updated_at,
            assets,
            target_languages,
        }
    }
}

#[cfg(test)]
mod tests {
    use dubbridge_domain::{
        asset::Asset,
        workspace::{OrgId, OrgMember, OrgRole, Organization, Project, ProjectId, TargetLanguage},
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn organization_response_maps_domain_fields() {
        let org = Organization::new("Acme".to_string());
        let response = OrganizationResponse::from(org.clone());
        assert_eq!(response.id, org.id.0);
        assert_eq!(response.name, "Acme");
        assert_eq!(response.created_at, org.created_at);
        assert_eq!(response.updated_at, org.updated_at);
    }

    #[test]
    fn member_response_maps_role_and_subject() {
        let member = OrgMember::new(OrgId::new(), Uuid::new_v4(), OrgRole::Admin);
        let response = OrgMemberResponse::from(member.clone());
        assert_eq!(response.org_id, member.org_id.0);
        assert_eq!(response.subject_id, member.subject_id);
        assert_eq!(response.role, OrgRole::Admin);
    }

    #[test]
    fn project_response_maps_project_fields() {
        let project = Project::new(OrgId::new(), "Season 1".to_string());
        let response = ProjectResponse::from(project.clone());
        assert_eq!(response.id, project.id.0);
        assert_eq!(response.org_id, project.org_id.0);
        assert_eq!(response.name, "Season 1");
    }

    #[test]
    fn target_language_response_maps_language_fields() {
        let target_language =
            TargetLanguage::new(ProjectId::new(), "en".to_string(), "es-ES".to_string());
        let response = TargetLanguageResponse::from(target_language.clone());
        assert_eq!(response.project_id, target_language.project_id.0);
        assert_eq!(response.source_lang, "en");
        assert_eq!(response.target_lang, "es-ES");
    }

    #[test]
    fn project_detail_response_contains_asset_summaries() {
        let project = Project::new(OrgId::new(), "Project X".to_string());
        let asset = Asset::new_pending("Trailer".to_string(), Uuid::new_v4());
        let detail = ProjectDetailResponse::new(
            project.clone(),
            vec![AssetSummaryResponse::from(asset.clone())],
            vec![TargetLanguageResponse::from(TargetLanguage::new(
                project.id,
                "en".to_string(),
                "fr-FR".to_string(),
            ))],
        );

        assert_eq!(detail.id, project.id.0);
        assert_eq!(detail.assets.len(), 1);
        assert_eq!(detail.assets[0].id, asset.id.0);
        assert_eq!(detail.target_languages.len(), 1);
    }
}
