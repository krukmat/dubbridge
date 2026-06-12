use std::sync::Arc;

use async_trait::async_trait;
use dubbridge_domain::{
    asset::{Asset, AssetId},
    audit::AuditEvent,
    workspace::{OrgId, OrgMember, Organization, Project, ProjectId, TargetLanguage},
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WorkspaceServiceError {
    #[error("database error: {0}")]
    Db(#[from] dubbridge_db::error::DbError),
}

#[async_trait]
pub trait WorkspaceService: Send + Sync {
    async fn create_org_with_owner(
        &self,
        organization: Organization,
        owner_membership: OrgMember,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError>;

    async fn list_orgs_for_subject(
        &self,
        subject_id: Uuid,
    ) -> Result<Vec<Organization>, WorkspaceServiceError>;

    async fn add_member_with_audit(
        &self,
        member: OrgMember,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError>;

    async fn list_org_members(
        &self,
        org_id: OrgId,
    ) -> Result<Vec<OrgMember>, WorkspaceServiceError>;

    async fn create_project_with_assets_and_audit(
        &self,
        project: Project,
        asset_ids: Vec<AssetId>,
        caller_subject_id: Uuid,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError>;

    async fn list_projects_for_org(
        &self,
        org_id: OrgId,
    ) -> Result<Vec<Project>, WorkspaceServiceError>;

    async fn get_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Project>, WorkspaceServiceError>;

    async fn list_assets_for_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Asset>, WorkspaceServiceError>;

    async fn link_asset_to_project(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        caller_subject_id: Uuid,
    ) -> Result<(), WorkspaceServiceError>;

    async fn list_target_languages(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError>;

    async fn replace_target_languages(
        &self,
        project_id: ProjectId,
        source_lang: String,
        target_languages: Vec<String>,
    ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError>;
}

pub type SharedWorkspaceService = Arc<dyn WorkspaceService>;

pub fn pg_workspace_service(pool: PgPool) -> SharedWorkspaceService {
    Arc::new(PgWorkspaceService { pool })
}

struct PgWorkspaceService {
    pool: PgPool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_service_error_display_db_variant() {
        let err = WorkspaceServiceError::Db(dubbridge_db::error::DbError::NotFound);
        assert!(err.to_string().contains("database error"));
    }
}

#[async_trait]
impl WorkspaceService for PgWorkspaceService {
    async fn create_org_with_owner(
        &self,
        organization: Organization,
        owner_membership: OrgMember,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;

        dubbridge_db::workspace_repo::insert_org_tx(&mut tx, &organization).await?;
        dubbridge_db::workspace_repo::add_org_member_tx(&mut tx, &owner_membership).await?;
        dubbridge_db::audit_repo::insert_audit_event_tx(&mut tx, &audit_event).await?;
        tx.commit().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        Ok(())
    }

    async fn list_orgs_for_subject(
        &self,
        subject_id: Uuid,
    ) -> Result<Vec<Organization>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::list_orgs_for_subject(&self.pool, subject_id).await?)
    }

    async fn add_member_with_audit(
        &self,
        member: OrgMember,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        dubbridge_db::workspace_repo::add_org_member_tx(&mut tx, &member).await?;
        dubbridge_db::audit_repo::insert_audit_event_tx(&mut tx, &audit_event).await?;
        tx.commit().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        Ok(())
    }

    async fn list_org_members(
        &self,
        org_id: OrgId,
    ) -> Result<Vec<OrgMember>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::list_org_members(&self.pool, org_id).await?)
    }

    async fn create_project_with_assets_and_audit(
        &self,
        project: Project,
        asset_ids: Vec<AssetId>,
        caller_subject_id: Uuid,
        audit_event: AuditEvent,
    ) -> Result<(), WorkspaceServiceError> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        dubbridge_db::workspace_repo::insert_project_tx(&mut tx, &project).await?;
        for asset_id in asset_ids {
            dubbridge_db::workspace_repo::link_asset_to_project_tx(
                &mut tx,
                project.id,
                asset_id,
                caller_subject_id,
            )
            .await?;
        }
        dubbridge_db::audit_repo::insert_audit_event_tx(&mut tx, &audit_event).await?;
        tx.commit().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        Ok(())
    }

    async fn list_projects_for_org(
        &self,
        org_id: OrgId,
    ) -> Result<Vec<Project>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::list_projects_for_org(&self.pool, org_id).await?)
    }

    async fn get_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Project>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::get_project(&self.pool, project_id).await?)
    }

    async fn list_assets_for_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Asset>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::list_assets_for_project(&self.pool, project_id).await?)
    }

    async fn link_asset_to_project(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        caller_subject_id: Uuid,
    ) -> Result<(), WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::link_asset_to_project(
            &self.pool,
            project_id,
            asset_id,
            caller_subject_id,
        )
        .await?)
    }

    async fn list_target_languages(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError> {
        Ok(dubbridge_db::workspace_repo::list_target_languages(&self.pool, project_id).await?)
    }

    async fn replace_target_languages(
        &self,
        project_id: ProjectId,
        source_lang: String,
        target_languages: Vec<String>,
    ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        dubbridge_db::workspace_repo::delete_target_languages_for_project_tx(&mut tx, project_id)
            .await?;

        let mut created = Vec::with_capacity(target_languages.len());
        for target_lang in target_languages {
            let target_language = TargetLanguage::new(project_id, source_lang.clone(), target_lang);
            dubbridge_db::workspace_repo::upsert_target_language_tx(&mut tx, &target_language)
                .await?;
            created.push(target_language);
        }

        tx.commit().await.map_err(|error| {
            WorkspaceServiceError::Db(dubbridge_db::error::DbError::QueryFailed(error))
        })?;
        Ok(created)
    }
}
