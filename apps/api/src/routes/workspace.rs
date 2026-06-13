use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
};
use dubbridge_auth::{
    AuthenticatedPrincipal, OrgMemberPrincipal, SharedTokenVerifier, authenticate_bearer,
    require_scope,
};
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    workspace::{OrgId, OrgMember, OrgRole, Organization, Project, ProjectId},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{
        ingestion::AssetSummaryResponse,
        workspace::{
            AddMemberRequest, CreateOrgRequest, CreateProjectRequest, LinkProjectAssetRequest,
            OrgMemberResponse, OrganizationResponse, ProjectDetailResponse, ProjectResponse,
            SetTargetLanguagesRequest, TargetLanguageResponse,
        },
    },
    middleware::org_scope::{require_org_member, resolve_org_membership},
    state::AppState,
    workspace_service::WorkspaceServiceError,
};

pub fn router(pool: PgPool, verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    let global_write_routes = Router::new()
        .route("/orgs", post(create_org))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:write"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let global_read_routes = Router::new()
        .route("/orgs", get(list_orgs))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let org_write_routes = Router::new()
        .route("/orgs/{org_id}/members", post(add_member))
        .route("/orgs/{org_id}/projects", post(create_project))
        .route(
            "/orgs/{org_id}/projects/{project_id}/assets",
            post(link_project_asset),
        )
        .route(
            "/orgs/{org_id}/projects/{project_id}/target-languages",
            put(set_target_languages),
        )
        .route_layer(middleware::from_fn_with_state(
            OrgRole::Admin,
            require_org_member,
        ))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            resolve_org_membership,
        ))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:write"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let org_read_routes = Router::new()
        .route("/orgs/{org_id}/members", get(list_members))
        .route("/orgs/{org_id}/projects", get(list_projects))
        .route(
            "/orgs/{org_id}/projects/{project_id}",
            get(get_project_detail),
        )
        .route(
            "/orgs/{org_id}/projects/{project_id}/target-languages",
            get(get_target_languages),
        )
        .route_layer(middleware::from_fn_with_state(
            OrgRole::Viewer,
            require_org_member,
        ))
        .route_layer(middleware::from_fn_with_state(pool, resolve_org_membership))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ));

    Router::new()
        .merge(global_write_routes)
        .merge(global_read_routes)
        .merge(org_write_routes)
        .merge(org_read_routes)
}

#[derive(Debug, Deserialize)]
struct ProjectPath {
    org_id: Uuid,
    project_id: Uuid,
}

async fn create_org(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<OrganizationResponse>), ApiError> {
    let name = normalized_non_empty(&request.name, "organization name")?;
    let organization = Organization::new(name);
    let owner_membership = OrgMember::new(organization.id, principal.subject_id, OrgRole::Owner);
    let audit_event = AuditEvent::new_workspace_event(
        AuditEventKind::OrgCreated,
        Some(
            json!({
                "actor_subject_id": principal.subject_id,
                "org_id": organization.id.0,
                "org_name": organization.name,
            })
            .to_string(),
        ),
    );

    state
        .workspace_service
        .create_org_with_owner(organization.clone(), owner_membership, audit_event)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok((
        StatusCode::CREATED,
        Json(OrganizationResponse::new(organization, OrgRole::Owner)),
    ))
}

async fn list_orgs(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Result<Json<Vec<OrganizationResponse>>, ApiError> {
    let orgs = state
        .workspace_service
        .list_orgs_for_subject(principal.subject_id)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(Json(
        orgs.into_iter()
            .map(|(org, role)| OrganizationResponse::new(org, role))
            .collect(),
    ))
}

async fn add_member(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Json(request): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<OrgMemberResponse>), ApiError> {
    let new_member = OrgMember::new(member.org_id, request.subject_id, request.role);
    let audit_event = AuditEvent::new_workspace_event(
        AuditEventKind::OrgMemberAdded,
        Some(
            json!({
                "actor_subject_id": member.principal.subject_id,
                "org_id": member.org_id.0,
                "subject_id": new_member.subject_id,
                "role": new_member.role,
            })
            .to_string(),
        ),
    );

    state
        .workspace_service
        .add_member_with_audit(new_member.clone(), audit_event)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok((
        StatusCode::CREATED,
        Json(OrgMemberResponse::from(new_member)),
    ))
}

async fn list_members(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
) -> Result<Json<Vec<OrgMemberResponse>>, ApiError> {
    let members = state
        .workspace_service
        .list_org_members(member.org_id)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(Json(
        members.into_iter().map(OrgMemberResponse::from).collect(),
    ))
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectDetailResponse>), ApiError> {
    let name = normalized_non_empty(&request.name, "project name")?;
    let project = Project::new(member.org_id, name);

    let audit_event = AuditEvent::new_workspace_event(
        AuditEventKind::ProjectCreated,
        Some(
            json!({
                "actor_subject_id": member.principal.subject_id,
                "org_id": member.org_id.0,
                "project_id": project.id.0,
                "project_name": project.name,
            })
            .to_string(),
        ),
    );
    state
        .workspace_service
        .create_project_with_assets_and_audit(
            project.clone(),
            request.asset_ids.into_iter().map(AssetId).collect(),
            member.principal.subject_id,
            audit_event,
        )
        .await
        .map_err(ApiError::from_workspace_service)
        .map_err(map_project_asset_error)?;

    let detail = build_project_detail_response(state.as_ref(), project.id, member.org_id).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn list_projects(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    let projects = state
        .workspace_service
        .list_projects_for_org(member.org_id)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(Json(
        projects.into_iter().map(ProjectResponse::from).collect(),
    ))
}

async fn get_project_detail(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ProjectPath>,
) -> Result<Json<ProjectDetailResponse>, ApiError> {
    if path.org_id != member.org_id.0 {
        return Err(ApiError::forbidden("project not found"));
    }

    Ok(Json(
        build_project_detail_response(state.as_ref(), ProjectId(path.project_id), member.org_id)
            .await?,
    ))
}

async fn link_project_asset(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ProjectPath>,
    Json(request): Json<LinkProjectAssetRequest>,
) -> Result<Json<ProjectDetailResponse>, ApiError> {
    if path.org_id != member.org_id.0 {
        return Err(ApiError::forbidden("project not found"));
    }

    let project_id = ProjectId(path.project_id);
    let _project = load_project_in_org(state.as_ref(), project_id, member.org_id).await?;

    state
        .workspace_service
        .link_asset_to_project(
            project_id,
            AssetId(request.asset_id),
            member.principal.subject_id,
        )
        .await
        .map_err(ApiError::from_workspace_service)
        .map_err(map_project_asset_error)?;

    Ok(Json(
        build_project_detail_response(state.as_ref(), project_id, member.org_id).await?,
    ))
}

async fn set_target_languages(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ProjectPath>,
    Json(request): Json<SetTargetLanguagesRequest>,
) -> Result<Json<Vec<TargetLanguageResponse>>, ApiError> {
    if path.org_id != member.org_id.0 {
        return Err(ApiError::forbidden("project not found"));
    }

    let project_id = ProjectId(path.project_id);
    let _project = load_project_in_org(state.as_ref(), project_id, member.org_id).await?;
    let source_lang = normalized_non_empty(&request.source_lang, "source language")?;
    let target_languages = normalized_target_languages(request.target_languages)?;

    let stored = state
        .workspace_service
        .replace_target_languages(project_id, source_lang, target_languages)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(Json(
        stored
            .into_iter()
            .map(TargetLanguageResponse::from)
            .collect(),
    ))
}

async fn get_target_languages(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ProjectPath>,
) -> Result<Json<Vec<TargetLanguageResponse>>, ApiError> {
    if path.org_id != member.org_id.0 {
        return Err(ApiError::forbidden("project not found"));
    }

    let project =
        load_project_in_org(state.as_ref(), ProjectId(path.project_id), member.org_id).await?;
    let target_languages = state
        .workspace_service
        .list_target_languages(project.id)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(Json(
        target_languages
            .into_iter()
            .map(TargetLanguageResponse::from)
            .collect(),
    ))
}

async fn build_project_detail_response(
    state: &AppState,
    project_id: ProjectId,
    org_id: OrgId,
) -> Result<ProjectDetailResponse, ApiError> {
    let project = load_project_in_org(state, project_id, org_id).await?;
    let assets = state
        .workspace_service
        .list_assets_for_project(project.id)
        .await
        .map_err(ApiError::from_workspace_service)?;
    let target_languages = state
        .workspace_service
        .list_target_languages(project.id)
        .await
        .map_err(ApiError::from_workspace_service)?;

    Ok(ProjectDetailResponse::new(
        project,
        assets.into_iter().map(AssetSummaryResponse::from).collect(),
        target_languages
            .into_iter()
            .map(TargetLanguageResponse::from)
            .collect(),
    ))
}

async fn load_project_in_org(
    state: &AppState,
    project_id: ProjectId,
    org_id: OrgId,
) -> Result<Project, ApiError> {
    let project = state
        .workspace_service
        .get_project(project_id)
        .await
        .map_err(ApiError::from_workspace_service)?
        .ok_or_else(|| ApiError::not_found("project not found"))?;

    if project.org_id != org_id {
        return Err(ApiError::forbidden("project not found"));
    }

    Ok(project)
}

fn normalized_non_empty(value: &str, field: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request(format!("{field} is required")));
    }
    Ok(trimmed.to_string())
}

fn normalized_target_languages(values: Vec<String>) -> Result<Vec<String>, ApiError> {
    let normalized: Vec<String> = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

    if normalized.is_empty() {
        return Err(ApiError::bad_request(
            "at least one target language is required",
        ));
    }

    Ok(normalized)
}

fn map_project_asset_error(error: ApiError) -> ApiError {
    if error.status == StatusCode::NOT_FOUND {
        return ApiError::forbidden("asset not found");
    }
    if error.message == "record not found" {
        return ApiError::forbidden("asset not found");
    }
    error
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn from_db(error: dubbridge_db::error::DbError) -> Self {
        match error {
            dubbridge_db::error::DbError::NotFound => Self::not_found("record not found"),
            dubbridge_db::error::DbError::ConnectionFailed(source)
            | dubbridge_db::error::DbError::QueryFailed(source) => {
                Self::internal(format!("database operation failed: {source}"))
            }
            dubbridge_db::error::DbError::UnknownStoredValue { field, value } => {
                Self::internal(format!("corrupt stored value in {field}: {value}"))
            }
        }
    }

    fn from_workspace_service(error: WorkspaceServiceError) -> Self {
        match error {
            WorkspaceServiceError::Db(error) => Self::from_db(error),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "error": self.message,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use dubbridge_auth::{
        AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
    };
    use dubbridge_domain::{
        asset::{Asset, IngestionStatus},
        audit::AuditEvent,
        workspace::{OrgMember, OrgRole, Organization, Project, TargetLanguage},
    };
    use dubbridge_storage::LocalFsAdapter;
    use tempfile::TempDir;

    use crate::{
        state::AppState,
        workspace_service::{SharedWorkspaceService, WorkspaceService, WorkspaceServiceError},
    };

    use super::*;

    #[derive(Default)]
    struct FakeWorkspaceData {
        orgs: HashMap<Uuid, Organization>,
        members: Vec<OrgMember>,
        projects: HashMap<Uuid, Project>,
        project_assets: HashMap<Uuid, Vec<Asset>>,
        target_languages: HashMap<Uuid, Vec<TargetLanguage>>,
        assets_by_id: HashMap<Uuid, Asset>,
        audits: Vec<AuditEvent>,
        fail_create_org: bool,
        fail_add_member: bool,
        fail_create_project: bool,
    }

    #[derive(Default)]
    struct FakeWorkspaceService {
        data: Mutex<FakeWorkspaceData>,
    }

    #[async_trait]
    impl WorkspaceService for FakeWorkspaceService {
        async fn create_org_with_owner(
            &self,
            organization: Organization,
            owner_membership: OrgMember,
            audit_event: AuditEvent,
        ) -> Result<(), WorkspaceServiceError> {
            let mut data = self.data.lock().expect("lock");
            if data.fail_create_org {
                return Err(WorkspaceServiceError::Db(
                    dubbridge_db::error::DbError::QueryFailed(sqlx::Error::RowNotFound),
                ));
            }
            data.orgs.insert(organization.id.0, organization);
            data.members.push(owner_membership);
            data.audits.push(audit_event);
            Ok(())
        }

        async fn list_orgs_for_subject(
            &self,
            subject_id: Uuid,
        ) -> Result<Vec<(Organization, OrgRole)>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            let mut orgs = Vec::new();
            for member in &data.members {
                if member.subject_id == subject_id
                    && let Some(org) = data.orgs.get(&member.org_id.0)
                {
                    orgs.push((org.clone(), member.role));
                }
            }
            Ok(orgs)
        }

        async fn add_member_with_audit(
            &self,
            member: OrgMember,
            audit_event: AuditEvent,
        ) -> Result<(), WorkspaceServiceError> {
            let mut data = self.data.lock().expect("lock");
            if data.fail_add_member {
                return Err(WorkspaceServiceError::Db(
                    dubbridge_db::error::DbError::QueryFailed(sqlx::Error::RowNotFound),
                ));
            }
            data.members.push(member);
            data.audits.push(audit_event);
            Ok(())
        }

        async fn list_org_members(
            &self,
            org_id: OrgId,
        ) -> Result<Vec<OrgMember>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            Ok(data
                .members
                .iter()
                .filter(|member| member.org_id == org_id)
                .cloned()
                .collect())
        }

        async fn create_project_with_assets_and_audit(
            &self,
            project: Project,
            asset_ids: Vec<AssetId>,
            caller_subject_id: Uuid,
            audit_event: AuditEvent,
        ) -> Result<(), WorkspaceServiceError> {
            let mut data = self.data.lock().expect("lock");
            if data.fail_create_project {
                return Err(WorkspaceServiceError::Db(
                    dubbridge_db::error::DbError::QueryFailed(sqlx::Error::RowNotFound),
                ));
            }
            let mut project_assets = Vec::new();
            for asset_id in asset_ids {
                let asset = data.assets_by_id.get(&asset_id.0).cloned().ok_or(
                    WorkspaceServiceError::Db(dubbridge_db::error::DbError::NotFound),
                )?;
                if asset.uploader_id != caller_subject_id {
                    return Err(WorkspaceServiceError::Db(
                        dubbridge_db::error::DbError::NotFound,
                    ));
                }
                project_assets.push(asset);
            }
            data.projects.insert(project.id.0, project.clone());
            data.project_assets.insert(project.id.0, project_assets);
            data.audits.push(audit_event);
            Ok(())
        }

        async fn list_projects_for_org(
            &self,
            org_id: OrgId,
        ) -> Result<Vec<Project>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            Ok(data
                .projects
                .values()
                .filter(|project| project.org_id == org_id)
                .cloned()
                .collect())
        }

        async fn get_project(
            &self,
            project_id: ProjectId,
        ) -> Result<Option<Project>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            Ok(data.projects.get(&project_id.0).cloned())
        }

        async fn list_assets_for_project(
            &self,
            project_id: ProjectId,
        ) -> Result<Vec<Asset>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            Ok(data
                .project_assets
                .get(&project_id.0)
                .cloned()
                .unwrap_or_default())
        }

        async fn link_asset_to_project(
            &self,
            project_id: ProjectId,
            asset_id: AssetId,
            caller_subject_id: Uuid,
        ) -> Result<(), WorkspaceServiceError> {
            let mut data = self.data.lock().expect("lock");
            let asset =
                data.assets_by_id
                    .get(&asset_id.0)
                    .cloned()
                    .ok_or(WorkspaceServiceError::Db(
                        dubbridge_db::error::DbError::NotFound,
                    ))?;
            if asset.uploader_id != caller_subject_id {
                return Err(WorkspaceServiceError::Db(
                    dubbridge_db::error::DbError::NotFound,
                ));
            }
            data.project_assets
                .entry(project_id.0)
                .or_default()
                .push(asset);
            Ok(())
        }

        async fn list_target_languages(
            &self,
            project_id: ProjectId,
        ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError> {
            let data = self.data.lock().expect("lock");
            Ok(data
                .target_languages
                .get(&project_id.0)
                .cloned()
                .unwrap_or_default())
        }

        async fn replace_target_languages(
            &self,
            project_id: ProjectId,
            source_lang: String,
            target_languages: Vec<String>,
        ) -> Result<Vec<TargetLanguage>, WorkspaceServiceError> {
            let mut data = self.data.lock().expect("lock");
            let stored: Vec<TargetLanguage> = target_languages
                .into_iter()
                .map(|target_lang| {
                    TargetLanguage::new(project_id, source_lang.clone(), target_lang)
                })
                .collect();
            data.target_languages.insert(project_id.0, stored.clone());
            Ok(stored)
        }
    }

    #[derive(Default)]
    struct StubVerifier;

    impl TokenVerifier for StubVerifier {
        fn verify_access_token(
            &self,
            _token: &str,
        ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
            Err(TokenVerificationError::MalformedToken)
        }
    }

    fn fake_state(service: SharedWorkspaceService) -> Arc<AppState> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://nobody@localhost/fake")
            .expect("lazy pool");
        let verifier: SharedTokenVerifier = Arc::new(StubVerifier);
        let temp_dir = TempDir::new().expect("temp dir");
        let config = dubbridge_config::AppConfig::from_env();
        Arc::new(AppState::with_workspace_service(
            pool,
            Box::new(LocalFsAdapter::new(temp_dir.path())),
            verifier,
            config,
            service,
        ))
    }

    fn principal(subject_id: Uuid, scopes: &[&str]) -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(subject_id, scopes.iter().map(|scope| scope.to_string()))
    }

    fn member_principal(subject_id: Uuid, org_id: OrgId, role: OrgRole) -> OrgMemberPrincipal {
        OrgMemberPrincipal {
            principal: principal(subject_id, &["workspaces:read", "workspaces:write"]),
            org_id,
            role,
        }
    }

    fn seed_org(
        service: &Arc<FakeWorkspaceService>,
        org: &Organization,
        subject_id: Uuid,
        role: OrgRole,
    ) {
        let mut data = service.data.lock().expect("lock");
        data.orgs.insert(org.id.0, org.clone());
        data.members.push(OrgMember::new(org.id, subject_id, role));
    }

    fn seed_project(service: &Arc<FakeWorkspaceService>, project: &Project) {
        let mut data = service.data.lock().expect("lock");
        data.projects.insert(project.id.0, project.clone());
    }

    fn seed_asset(service: &Arc<FakeWorkspaceService>, asset: &Asset) {
        let mut data = service.data.lock().expect("lock");
        data.assets_by_id.insert(asset.id.0, asset.clone());
    }

    #[tokio::test]
    async fn router_mounts_workspace_routes() {
        let verifier: SharedTokenVerifier = Arc::new(StubVerifier);
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://nobody@localhost/fake")
            .expect("lazy pool");
        let _app = router(pool, verifier);
    }

    #[tokio::test]
    async fn create_org_handler_persists_owner_and_audit() {
        let service = Arc::new(FakeWorkspaceService::default());
        let state = fake_state(service.clone());
        let subject_id = Uuid::new_v4();

        let (status, Json(response)) = create_org(
            State(state),
            Extension(principal(subject_id, &["workspaces:write"])),
            Json(CreateOrgRequest {
                name: "Acme".to_string(),
            }),
        )
        .await
        .expect("create org");

        assert_eq!(status, StatusCode::CREATED);
        let data = service.data.lock().expect("lock");
        assert_eq!(data.orgs.len(), 1);
        assert_eq!(data.members.len(), 1);
        assert_eq!(data.members[0].role, OrgRole::Owner);
        assert_eq!(data.audits.len(), 1);
        assert_eq!(response.name, "Acme");
        assert_eq!(response.viewer_role, OrgRole::Owner);
    }

    #[tokio::test]
    async fn create_org_handler_surfaces_fail_closed_audit_error() {
        let service = Arc::new(FakeWorkspaceService::default());
        service.data.lock().expect("lock").fail_create_org = true;
        let state = fake_state(service);
        let err = create_org(
            State(state),
            Extension(principal(Uuid::new_v4(), &["workspaces:write"])),
            Json(CreateOrgRequest {
                name: "Acme".to_string(),
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn list_orgs_handler_returns_subject_orgs() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let subject_id = Uuid::new_v4();
        seed_org(&service, &org, subject_id, OrgRole::Owner);
        let state = fake_state(service);

        let Json(orgs) = list_orgs(
            State(state),
            Extension(principal(subject_id, &["workspaces:read"])),
        )
        .await
        .expect("list orgs");
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].id, org.id.0);
        assert_eq!(orgs[0].viewer_role, OrgRole::Owner);
    }

    #[tokio::test]
    async fn add_member_handler_persists_membership_and_audit() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        let state = fake_state(service.clone());
        let target_id = Uuid::new_v4();

        let (status, Json(response)) = add_member(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Json(AddMemberRequest {
                subject_id: target_id,
                role: OrgRole::Reviewer,
            }),
        )
        .await
        .expect("add member");

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.subject_id, target_id);
        let data = service.data.lock().expect("lock");
        assert_eq!(data.audits.len(), 1);
        assert_eq!(data.members.len(), 2);
    }

    #[tokio::test]
    async fn list_members_handler_returns_org_members() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let viewer_id = Uuid::new_v4();
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        service
            .data
            .lock()
            .expect("lock")
            .members
            .push(OrgMember::new(org.id, viewer_id, OrgRole::Viewer));
        let state = fake_state(service);

        let Json(members) = list_members(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
        )
        .await
        .expect("list members");
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn create_project_handler_returns_detail_with_assets() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let asset = Asset {
            id: AssetId::new(),
            title: "Trailer".to_string(),
            uploader_id: actor_id,
            status: IngestionStatus::Finalized,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_asset(&service, &asset);
        let state = fake_state(service.clone());

        let (status, Json(detail)) = create_project(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Json(CreateProjectRequest {
                name: "Launch".to_string(),
                asset_ids: vec![asset.id.0],
            }),
        )
        .await
        .expect("create project");

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(detail.assets.len(), 1);
        let data = service.data.lock().expect("lock");
        assert_eq!(data.projects.len(), 1);
        assert_eq!(data.audits.len(), 1);
    }

    #[tokio::test]
    async fn create_project_handler_rejects_foreign_asset() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let asset = Asset {
            id: AssetId::new(),
            title: "Trailer".to_string(),
            uploader_id: Uuid::new_v4(),
            status: IngestionStatus::Finalized,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_asset(&service, &asset);
        let state = fake_state(service);

        let err = create_project(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Json(CreateProjectRequest {
                name: "Launch".to_string(),
                asset_ids: vec![asset.id.0],
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn list_projects_handler_returns_org_projects() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let project = Project::new(org.id, "Launch".to_string());
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_project(&service, &project);
        let state = fake_state(service);

        let Json(projects) = list_projects(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
        )
        .await
        .expect("list projects");
        assert_eq!(projects.len(), 1);
    }

    #[tokio::test]
    async fn get_project_detail_handler_rejects_cross_org_path() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let other_org = Organization::new("Other".to_string());
        let actor_id = Uuid::new_v4();
        let project = Project::new(other_org.id, "Foreign".to_string());
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_project(&service, &project);
        let state = fake_state(service);

        let err = get_project_detail(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: org.id.0,
                project_id: project.id.0,
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn get_project_detail_handler_rejects_path_org_mismatch() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        let state = fake_state(service);

        let err = get_project_detail(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: Uuid::new_v4(),
                project_id: Uuid::new_v4(),
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn link_project_asset_handler_updates_detail() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let project = Project::new(org.id, "Launch".to_string());
        let asset = Asset {
            id: AssetId::new(),
            title: "Trailer".to_string(),
            uploader_id: actor_id,
            status: IngestionStatus::Finalized,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_project(&service, &project);
        seed_asset(&service, &asset);
        let state = fake_state(service);

        let Json(detail) = link_project_asset(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: org.id.0,
                project_id: project.id.0,
            }),
            Json(LinkProjectAssetRequest {
                asset_id: asset.id.0,
            }),
        )
        .await
        .expect("link asset");
        assert_eq!(detail.assets.len(), 1);
    }

    #[tokio::test]
    async fn link_project_asset_handler_rejects_path_org_mismatch() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let state = fake_state(service);

        let err = link_project_asset(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: Uuid::new_v4(),
                project_id: Uuid::new_v4(),
            }),
            Json(LinkProjectAssetRequest {
                asset_id: Uuid::new_v4(),
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn set_and_get_target_languages_handlers_round_trip() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let project = Project::new(org.id, "Launch".to_string());
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_project(&service, &project);
        let state = fake_state(service.clone());

        let Json(stored) = set_target_languages(
            State(state.clone()),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: org.id.0,
                project_id: project.id.0,
            }),
            Json(SetTargetLanguagesRequest {
                source_lang: "en".to_string(),
                target_languages: vec!["es-ES".to_string(), "fr-FR".to_string()],
            }),
        )
        .await
        .expect("set target languages");
        assert_eq!(stored.len(), 2);

        let Json(read_back) = get_target_languages(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: org.id.0,
                project_id: project.id.0,
            }),
        )
        .await
        .expect("get target languages");
        assert_eq!(read_back.len(), 2);
    }

    #[tokio::test]
    async fn target_language_handlers_reject_path_org_mismatch() {
        let service = Arc::new(FakeWorkspaceService::default());
        let org = Organization::new("Acme".to_string());
        let actor_id = Uuid::new_v4();
        let project = Project::new(org.id, "Launch".to_string());
        seed_org(&service, &org, actor_id, OrgRole::Owner);
        seed_project(&service, &project);
        let state = fake_state(service);

        let set_err = set_target_languages(
            State(state.clone()),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: Uuid::new_v4(),
                project_id: project.id.0,
            }),
            Json(SetTargetLanguagesRequest {
                source_lang: "en".to_string(),
                target_languages: vec!["es-ES".to_string()],
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(set_err.status, StatusCode::FORBIDDEN);

        let get_err = get_target_languages(
            State(state),
            Extension(member_principal(actor_id, org.id, OrgRole::Owner)),
            Path(ProjectPath {
                org_id: Uuid::new_v4(),
                project_id: project.id.0,
            }),
        )
        .await
        .expect_err("error");
        assert_eq!(get_err.status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn api_error_helpers_cover_remaining_mappings() {
        let not_found = map_project_asset_error(ApiError::not_found("record not found"));
        assert_eq!(not_found.status, StatusCode::FORBIDDEN);

        let unknown = ApiError::from_db(dubbridge_db::error::DbError::UnknownStoredValue {
            field: "x",
            value: "bad".to_string(),
        });
        assert_eq!(unknown.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(unknown.message.contains("corrupt stored value"));

        let response = ApiError::bad_request("oops").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn normalization_helpers_fail_closed() {
        assert!(normalized_non_empty("   ", "project name").is_err());
        assert!(normalized_target_languages(vec!["   ".to_string()]).is_err());
        assert_eq!(
            normalized_target_languages(vec![" es-ES ".to_string()]).expect("normalize"),
            vec!["es-ES".to_string()]
        );
    }
}
