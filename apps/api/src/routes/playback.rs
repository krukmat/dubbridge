use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::SharedTokenVerifier;
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    playback_service::{get_playback_manifest, get_playback_segment, issue_playback_grant},
    state::AppState,
};

const PLAYBACK_REQUIRED_SCOPE: &str = "workspaces:write";

pub fn router(state: Arc<AppState>, _verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    Router::new()
        .merge(
            Router::new()
                .route("/assets/{id}/playback-grants", post(issue_playback_grant))
                .route_layer(middleware::from_fn_with_state(
                    state,
                    authorize_playback_grant_request,
                )),
        )
        .route(
            "/assets/{id}/playback/{grant_id}/manifest",
            get(get_playback_manifest),
        )
        .route(
            "/assets/{id}/playback/segments/{filename}",
            get(get_playback_segment),
        )
}

async fn authorize_playback_grant_request(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let asset_id = playback_asset_id_from_request(&request);
    let token = match bearer_token(request.headers()) {
        Some(token) => token,
        None => {
            return auth_boundary_denial_response(
                &state,
                asset_id,
                None,
                StatusCode::UNAUTHORIZED,
                "auth_unauthorized",
            )
            .await;
        }
    };

    let principal = match state.verifier.verify_access_token(token) {
        Ok(principal) => principal,
        Err(_) => {
            return auth_boundary_denial_response(
                &state,
                asset_id,
                None,
                StatusCode::UNAUTHORIZED,
                "auth_unauthorized",
            )
            .await;
        }
    };

    if !principal.has_scope(PLAYBACK_REQUIRED_SCOPE) {
        return auth_boundary_denial_response(
            &state,
            asset_id,
            Some(principal.subject_id),
            StatusCode::FORBIDDEN,
            "missing_scope",
        )
        .await;
    }

    request.extensions_mut().insert(principal);
    next.run(request).await
}

async fn auth_boundary_denial_response(
    state: &AppState,
    asset_id: Option<AssetId>,
    actor_subject_id: Option<Uuid>,
    status: StatusCode,
    reason: &'static str,
) -> Response {
    if let Some(asset_id) = asset_id {
        let event = AuditEvent::new_playback_event(
            asset_id,
            AuditEventKind::PlaybackGrantRefused,
            Some(
                json!({
                    "asset_id": asset_id.0,
                    "actor_subject_id": actor_subject_id,
                    "org_id": serde_json::Value::Null,
                    "project_id": serde_json::Value::Null,
                    "reason": reason,
                })
                .to_string(),
            ),
        );

        if let Err(error) = emit_governance_audit(&state.pool, &event).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": error.to_string(),
                })),
            )
                .into_response();
        }
    }

    status.into_response()
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    let header_value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let mut parts = header_value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;

    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(token)
}

fn playback_asset_id_from_request(request: &Request) -> Option<AssetId> {
    let mut segments = request.uri().path().split('/');
    let _ = segments.next()?;
    let collection = segments.next()?;
    let asset_id = segments.next()?;
    let action = segments.next()?;

    if collection != "assets" || action != "playback-grants" {
        return None;
    }

    Uuid::parse_str(asset_id).ok().map(AssetId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
    };
    use dubbridge_auth::{AuthenticatedPrincipal, TokenVerificationError, TokenVerifier};
    use dubbridge_db::{artifact_repo, playback_repo, preparation_repo};
    use dubbridge_domain::{
        artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
        asset::{Asset, AssetId},
        workspace::{OrgId, OrgMember, OrgRole, Organization, Project},
    };
    use dubbridge_storage::LocalFsAdapter;
    use sqlx::PgPool;
    use std::{collections::HashMap, env};
    use tempfile::TempDir;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{build_app, state::AppState};

    #[derive(Clone, Default)]
    struct StubTokenVerifier {
        responses: HashMap<String, Result<AuthenticatedPrincipal, TokenVerificationError>>,
    }

    impl StubTokenVerifier {
        fn with_token(
            mut self,
            token: &str,
            result: Result<AuthenticatedPrincipal, TokenVerificationError>,
        ) -> Self {
            self.responses.insert(token.to_string(), result);
            self
        }
    }

    impl TokenVerifier for StubTokenVerifier {
        fn verify_access_token(
            &self,
            token: &str,
        ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
            self.responses
                .get(token)
                .cloned()
                .unwrap_or(Err(TokenVerificationError::MalformedToken))
        }
    }

    struct TestContext {
        pool: PgPool,
        app: axum::Router,
        reviewer_id: Uuid,
        viewer_id: Uuid,
        outsider_id: Uuid,
        write_token: String,
        viewer_write_token: String,
        outsider_write_token: String,
        read_only_token: String,
    }

    struct PlaybackFixture {
        org: Organization,
        project: Project,
        asset: Asset,
    }

    impl TestContext {
        async fn new() -> Option<Self> {
            let database_url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
            let pool = PgPool::connect(&database_url)
                .await
                .expect("connect database");
            migrate_and_reset(&pool).await;

            let storage_dir = TempDir::new().expect("temp dir");
            let storage_path = storage_dir.path().to_path_buf();

            let reviewer_id =
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440100").expect("uuid");
            let viewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440101").expect("uuid");
            let outsider_id =
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440102").expect("uuid");

            let write_token = "playback-reviewer-write-token".to_string();
            let viewer_write_token = "playback-viewer-write-token".to_string();
            let outsider_write_token = "playback-outsider-write-token".to_string();
            let read_only_token = "playback-read-only-token".to_string();

            let verifier: SharedTokenVerifier = Arc::new(
                StubTokenVerifier::default()
                    .with_token(
                        &write_token,
                        Ok(AuthenticatedPrincipal::new(
                            reviewer_id,
                            ["workspaces:read", "workspaces:write"]
                                .into_iter()
                                .map(str::to_string),
                        )),
                    )
                    .with_token(
                        &viewer_write_token,
                        Ok(AuthenticatedPrincipal::new(
                            viewer_id,
                            ["workspaces:read", "workspaces:write"]
                                .into_iter()
                                .map(str::to_string),
                        )),
                    )
                    .with_token(
                        &outsider_write_token,
                        Ok(AuthenticatedPrincipal::new(
                            outsider_id,
                            ["workspaces:read", "workspaces:write"]
                                .into_iter()
                                .map(str::to_string),
                        )),
                    )
                    .with_token(
                        &read_only_token,
                        Ok(AuthenticatedPrincipal::new(
                            reviewer_id,
                            ["workspaces:read"].into_iter().map(str::to_string),
                        )),
                    ),
            );

            let state = Arc::new(AppState::new(
                pool.clone(),
                Box::new(LocalFsAdapter::new(&storage_path)),
                verifier.clone(),
                dubbridge_config::AppConfig::from_env(),
            ));

            Some(Self {
                pool,
                app: build_app(state, verifier),
                reviewer_id,
                viewer_id,
                outsider_id,
                write_token,
                viewer_write_token,
                outsider_write_token,
                read_only_token,
            })
        }
    }

    fn test_app() -> axum::Router {
        let storage_dir = TempDir::new().expect("temp dir");
        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_token(
                    "workspaces-write-token",
                    Ok(AuthenticatedPrincipal::new(
                        Uuid::new_v4(),
                        ["workspaces:write"].into_iter().map(str::to_string),
                    )),
                )
                .with_token(
                    "workspaces-read-token",
                    Ok(AuthenticatedPrincipal::new(
                        Uuid::new_v4(),
                        ["workspaces:read"].into_iter().map(str::to_string),
                    )),
                ),
        );
        let pool = PgPool::connect_lazy("postgres://nobody@localhost/fake").expect("lazy pool");
        let state = Arc::new(AppState::new(
            pool,
            Box::new(LocalFsAdapter::new(storage_dir.path())),
            verifier.clone(),
            dubbridge_config::AppConfig::from_env(),
        ));

        build_app(state, verifier)
    }

    #[tokio::test]
    async fn playback_grant_route_requires_authentication() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            None,
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn playback_grant_route_requires_workspace_write_scope() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.read_only_token),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn playback_asset_id_from_request_extracts_uuid_for_playback_route() {
        let asset_id = Uuid::new_v4();
        let request = Request::builder()
            .uri(format!("/assets/{asset_id}/playback-grants"))
            .body(Body::empty())
            .expect("request");

        assert_eq!(
            playback_asset_id_from_request(&request),
            Some(AssetId(asset_id))
        );
    }

    #[tokio::test]
    async fn playback_grant_route_rejects_malformed_asset_id() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/assets/not-a-uuid/playback-grants")
                    .header(header::AUTHORIZATION, "Bearer workspaces-write-token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn build_app_keeps_health_route_after_playback_merge() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health/live")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn playback_grant_route_issues_grant_for_authorized_reviewer() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        insert_membership(
            &ctx.pool,
            fixture.org.id,
            ctx.reviewer_id,
            OrgRole::Reviewer,
        )
        .await;
        mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.write_token),
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;
        let grant_id = Uuid::parse_str(body["grant_id"].as_str().expect("grant id")).expect("uuid");

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(count_playback_grants(&ctx.pool).await, 1);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            0
        );
        let detail = latest_audit_detail(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
            .await
            .expect("playback grant issued detail");
        assert!(detail.contains(&grant_id.to_string()));
        assert!(detail.contains(&ctx.reviewer_id.to_string()));

        let grant = playback_repo::get_active_grant(
            &ctx.pool,
            dubbridge_domain::playback::PlaybackGrantId(grant_id),
        )
        .await
        .expect("load grant")
        .expect("grant exists");
        assert_eq!(grant.asset_id, fixture.asset.id);
        assert_eq!(grant.principal.principal_id, ctx.reviewer_id);
        assert_eq!(grant.principal.org_id, fixture.org.id);
        assert_eq!(grant.principal.project_id, fixture.project.id);
    }

    #[tokio::test]
    async fn playback_grant_route_rejects_non_member_before_grant_creation() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        insert_membership(
            &ctx.pool,
            fixture.org.id,
            ctx.reviewer_id,
            OrgRole::Reviewer,
        )
        .await;
        mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.outsider_write_token),
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "asset not found");
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            0
        );
        let detail = latest_audit_detail(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
            .await
            .expect("playback grant refused detail");
        assert!(detail.contains("asset_not_found"));
        assert!(detail.contains(&ctx.outsider_id.to_string()));
        assert_ne!(ctx.outsider_id, ctx.reviewer_id);
    }

    #[tokio::test]
    async fn playback_grant_route_rejects_member_with_insufficient_role() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        insert_membership(&ctx.pool, fixture.org.id, ctx.viewer_id, OrgRole::Viewer).await;
        mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.viewer_write_token),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            0
        );
    }

    #[tokio::test]
    async fn playback_grant_route_rejects_asset_that_is_not_ready() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        insert_membership(
            &ctx.pool,
            fixture.org.id,
            ctx.reviewer_id,
            OrgRole::Reviewer,
        )
        .await;
        preparation_repo::upsert_preparation_status(
            &ctx.pool,
            fixture.asset.id,
            PreparationStatus::InProgress,
            None,
        )
        .await
        .expect("mark in progress");

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.write_token),
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"], "asset not ready for playback");
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            0
        );
        let detail = latest_audit_detail(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
            .await
            .expect("playback grant refused detail");
        assert!(detail.contains("asset_not_ready"));
        assert_ne!(ctx.read_only_token, ctx.write_token);
    }

    #[tokio::test]
    async fn playback_grant_route_fails_closed_when_refusal_audit_persistence_fails() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        insert_membership(
            &ctx.pool,
            fixture.org.id,
            ctx.reviewer_id,
            OrgRole::Reviewer,
        )
        .await;
        preparation_repo::upsert_preparation_status(
            &ctx.pool,
            fixture.asset.id,
            PreparationStatus::InProgress,
            None,
        )
        .await
        .expect("mark in progress");
        drop_audit_events_table(&ctx.pool).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.write_token),
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            body["error"]
                .as_str()
                .expect("error message")
                .contains("audit persistence failed")
        );
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    }

    #[tokio::test]
    async fn playback_grant_route_writes_refusal_audit_for_missing_bearer_token() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            None,
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            0
        );
        let detail = latest_audit_detail(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
            .await
            .expect("playback grant refused detail");
        assert!(detail.contains("auth_unauthorized"));
    }

    #[tokio::test]
    async fn playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            Some(&ctx.read_only_token),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
                .await,
            1
        );
        assert_eq!(
            count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued")
                .await,
            0
        );
        let detail = latest_audit_detail(&ctx.pool, fixture.asset.id.0, "playback_grant_refused")
            .await
            .expect("playback grant refused detail");
        assert!(detail.contains("missing_scope"));
        assert!(detail.contains(&ctx.reviewer_id.to_string()));
    }

    #[tokio::test]
    async fn playback_grant_route_fails_closed_when_auth_boundary_audit_persistence_fails() {
        let Some(ctx) = TestContext::new().await else {
            eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
            return;
        };

        let fixture = insert_playback_fixture(&ctx.pool).await;
        drop_audit_events_table(&ctx.pool).await;

        let response = send_request(
            &ctx.app,
            Method::POST,
            &format!("/assets/{}/playback-grants", fixture.asset.id.0),
            None,
        )
        .await;
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            body["error"]
                .as_str()
                .expect("error message")
                .contains("audit persistence failed")
        );
        assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    }

    async fn migrate_and_reset(pool: &PgPool) {
        sqlx::migrate!("../../infra/migrations")
            .run(pool)
            .await
            .expect("migrations");

        sqlx::query(
            "TRUNCATE TABLE playback_grants, publications, review_decisions, review_tasks, asset_preparation_status, target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets RESTART IDENTITY CASCADE",
        )
        .execute(pool)
        .await
        .expect("truncate tables");
    }

    async fn insert_playback_fixture(pool: &PgPool) -> PlaybackFixture {
        let org = Organization::new("Playback Org".to_string());
        sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
            .bind(org.id.0)
            .bind(&org.name)
            .execute(pool)
            .await
            .expect("insert org");

        let project = Project::new(org.id, "Playback Project".to_string());
        sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
            .bind(project.id.0)
            .bind(project.org_id.0)
            .bind(&project.name)
            .execute(pool)
            .await
            .expect("insert project");

        let mut asset = Asset::new_pending("playback-asset".to_string(), Uuid::new_v4());
        asset.status = dubbridge_domain::asset::IngestionStatus::Finalized;
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset.id.0)
            .bind(&asset.title)
            .bind(asset.uploader_id)
            .bind(asset.status.to_string())
            .execute(pool)
            .await
            .expect("insert asset");

        sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
            .bind(project.id.0)
            .bind(asset.id.0)
            .execute(pool)
            .await
            .expect("insert project asset");

        PlaybackFixture {
            org,
            project,
            asset,
        }
    }

    async fn insert_membership(pool: &PgPool, org_id: OrgId, subject_id: Uuid, role: OrgRole) {
        let member = OrgMember::new(org_id, subject_id, role);
        sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
            .bind(member.org_id.0)
            .bind(member.subject_id)
            .bind(member.role.to_string())
            .execute(pool)
            .await
            .expect("insert membership");
    }

    async fn mark_asset_ready_with_manifest(
        pool: &PgPool,
        asset_id: dubbridge_domain::asset::AssetId,
    ) {
        preparation_repo::upsert_preparation_status(pool, asset_id, PreparationStatus::Ready, None)
            .await
            .expect("upsert ready");

        let source = ArtifactRecord::new_original(
            asset_id,
            Uuid::new_v4(),
            format!("ingest/{asset_id}/source.mp4"),
            "video/mp4".into(),
            1_000,
            "deadbeef".into(),
        );
        artifact_repo::insert_artifact_record(pool, &source)
            .await
            .expect("insert source artifact");

        let manifest = DerivedArtifact::new(
            asset_id,
            source.id,
            ArtifactKind::HlsManifest,
            format!("prepared/{asset_id}/index.m3u8"),
            "application/vnd.apple.mpegurl".into(),
            512,
            "manifestchk".into(),
        );
        preparation_repo::insert_derived_artifact(pool, &manifest)
            .await
            .expect("insert manifest");
    }

    async fn send_request(
        app: &axum::Router,
        method: Method,
        uri: &str,
        bearer_token: Option<&str>,
    ) -> axum::response::Response {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = bearer_token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }

        app.clone()
            .oneshot(builder.body(Body::empty()).expect("request"))
            .await
            .expect("response")
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        serde_json::from_slice(
            &to_bytes(response.into_body(), 64 * 1024)
                .await
                .expect("body"),
        )
        .expect("json body")
    }

    async fn count_playback_grants(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM playback_grants")
            .fetch_one(pool)
            .await
            .expect("count playback grants")
    }

    async fn count_audit_events_by_kind(pool: &PgPool, asset_id: Uuid, event_kind: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2",
        )
        .bind(asset_id)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("count audit events")
    }

    async fn latest_audit_detail(
        pool: &PgPool,
        asset_id: Uuid,
        event_kind: &str,
    ) -> Option<String> {
        sqlx::query_scalar(
            "SELECT detail FROM audit_events WHERE asset_id = $1 AND event_kind = $2 ORDER BY happened_at DESC, id DESC LIMIT 1",
        )
        .bind(asset_id)
        .bind(event_kind)
        .fetch_optional(pool)
        .await
        .expect("latest audit detail")
    }

    async fn drop_audit_events_table(pool: &PgPool) {
        sqlx::query("DROP TABLE audit_events")
            .execute(pool)
            .await
            .expect("drop audit_events");
    }
}
