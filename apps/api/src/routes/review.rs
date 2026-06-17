use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use dubbridge_auth::{OrgMemberPrincipal, SharedTokenVerifier, authenticate_bearer, require_scope};
use dubbridge_db::{error::DbError, review_repo};
use dubbridge_domain::{
    review::{PublicationStatus, ReviewTaskId, ReviewVerdict},
    workspace::{OrgId, OrgRole, ProjectId},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::review::{
        ReviewDecisionRequest, ReviewDecisionResponse, ReviewPublicationResponse,
        ReviewQueueResponse, ReviewTaskResponse,
    },
    middleware::org_scope::{require_org_member, resolve_org_membership},
    review_gate::{self, ReviewGateError},
    state::AppState,
};

pub fn router(pool: PgPool, verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    let read_routes = Router::new()
        .route(
            "/orgs/{org_id}/projects/{project_id}/review-tasks",
            get(list_review_queue),
        )
        .route_layer(middleware::from_fn_with_state(
            OrgRole::Reviewer,
            require_org_member,
        ))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            resolve_org_membership,
        ))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:read"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier.clone(),
            authenticate_bearer,
        ));

    let write_routes = Router::new()
        .route(
            "/orgs/{org_id}/projects/{project_id}/review-tasks/{review_task_id}/decision",
            post(record_decision),
        )
        .route(
            "/orgs/{org_id}/projects/{project_id}/review-tasks/{review_task_id}/publish",
            post(publish_review_task_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            OrgRole::Reviewer,
            require_org_member,
        ))
        .route_layer(middleware::from_fn_with_state(pool, resolve_org_membership))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from("workspaces:write"),
            require_scope,
        ))
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ));

    Router::new().merge(read_routes).merge(write_routes)
}

#[derive(Debug, Deserialize)]
struct ReviewTaskPath {
    org_id: Uuid,
    project_id: Uuid,
    review_task_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct ProjectPath {
    org_id: Uuid,
    project_id: Uuid,
}

async fn list_review_queue(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ProjectPath>,
) -> Result<Json<ReviewQueueResponse>, ApiError> {
    ensure_member_matches_project_path(&member, path.org_id)?;

    let project_id = ProjectId(path.project_id);
    let org_id = OrgId(path.org_id);
    ensure_project_in_org(&state.pool, org_id, project_id).await?;

    let tasks = review_repo::list_review_tasks_for_scope(&state.pool, org_id, project_id, None)
        .await
        .map_err(ApiError::from_db)?;

    Ok(Json(ReviewQueueResponse {
        org_id: path.org_id,
        project_id: path.project_id,
        tasks: tasks.into_iter().map(ReviewTaskResponse::from).collect(),
    }))
}

async fn record_decision(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ReviewTaskPath>,
    Json(request): Json<ReviewDecisionRequest>,
) -> Result<(StatusCode, Json<ReviewDecisionResponse>), ApiError> {
    ensure_member_matches_project_path(&member, path.org_id)?;
    let task_id = ensure_scoped_review_task(
        &state.pool,
        OrgId(path.org_id),
        ProjectId(path.project_id),
        ReviewTaskId(path.review_task_id),
    )
    .await?;

    let state_after = match request.verdict {
        ReviewVerdict::Approved => {
            review_gate::approve_review_task(
                &state.pool,
                task_id,
                member.principal.subject_id,
                request.comment,
            )
            .await
        }
        ReviewVerdict::Rejected => {
            review_gate::reject_review_task(
                &state.pool,
                task_id,
                member.principal.subject_id,
                request.comment,
            )
            .await
        }
    }
    .map_err(ApiError::from_review_gate)?;

    Ok((
        StatusCode::OK,
        Json(ReviewDecisionResponse {
            review_task_id: task_id.0,
            state: state_after,
        }),
    ))
}

async fn publish_review_task_handler(
    State(state): State<Arc<AppState>>,
    Extension(member): Extension<OrgMemberPrincipal>,
    Path(path): Path<ReviewTaskPath>,
) -> Result<(StatusCode, Json<ReviewPublicationResponse>), ApiError> {
    ensure_member_matches_project_path(&member, path.org_id)?;
    let task_id = ensure_scoped_review_task(
        &state.pool,
        OrgId(path.org_id),
        ProjectId(path.project_id),
        ReviewTaskId(path.review_task_id),
    )
    .await?;

    let row = review_gate::publish_review_task(&state.pool, task_id, member.principal.subject_id)
        .await
        .map_err(ApiError::from_review_gate)?;

    Ok((
        StatusCode::CREATED,
        Json(ReviewPublicationResponse {
            review_task_id: row.review_task_id.0,
            status: PublicationStatus::Published,
            published_by: row.published_by,
            published_at: row.published_at,
        }),
    ))
}

fn ensure_member_matches_project_path(
    member: &OrgMemberPrincipal,
    path_org_id: Uuid,
) -> Result<(), ApiError> {
    if member.org_id.0 == path_org_id {
        Ok(())
    } else {
        Err(ApiError::forbidden("project not found"))
    }
}

async fn ensure_project_in_org(
    pool: &PgPool,
    org_id: OrgId,
    project_id: ProjectId,
) -> Result<(), ApiError> {
    let projects = dubbridge_db::workspace_repo::list_projects_for_org(pool, org_id)
        .await
        .map_err(ApiError::from_db)?;
    if projects.into_iter().any(|project| project.id == project_id) {
        Ok(())
    } else {
        Err(ApiError::forbidden("project not found"))
    }
}

async fn ensure_scoped_review_task(
    pool: &PgPool,
    org_id: OrgId,
    project_id: ProjectId,
    review_task_id: ReviewTaskId,
) -> Result<ReviewTaskId, ApiError> {
    let task = review_repo::get_review_task(pool, review_task_id)
        .await
        .map_err(ApiError::from_db)?
        .ok_or_else(|| ApiError::forbidden("review task not found"))?;

    if task.org_id == org_id && task.project_id == project_id {
        Ok(task.id)
    } else {
        Err(ApiError::forbidden("review task not found"))
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn from_db(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::forbidden("resource not found"),
            DbError::ConnectionFailed(source) | DbError::QueryFailed(source) => {
                Self::internal(format!("database operation failed: {source}"))
            }
            DbError::UnknownStoredValue { field, value } => {
                Self::internal(format!("corrupt stored value in {field}: {value}"))
            }
        }
    }

    fn from_review_gate(error: ReviewGateError) -> Self {
        match error {
            ReviewGateError::ReviewTaskNotFound { .. } => Self::forbidden("review task not found"),
            ReviewGateError::ReviewNotApproved { .. } => {
                Self::conflict("review approval is required before publication")
            }
            ReviewGateError::AlreadyPublished { .. } => {
                Self::conflict("review task is already published")
            }
            ReviewGateError::Db(message) => {
                Self::internal(format!("review gate database operation failed: {message}"))
            }
            ReviewGateError::AuditFailed(message) => {
                Self::internal(format!("review gate audit failed: {message}"))
            }
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
