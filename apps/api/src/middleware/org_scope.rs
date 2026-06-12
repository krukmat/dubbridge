use std::collections::HashMap;

use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use dubbridge_auth::{AuthenticatedPrincipal, OrgMemberPrincipal};
use dubbridge_domain::workspace::{OrgId, OrgRole};
use sqlx::PgPool;
use uuid::Uuid;

/// Resolves the caller's org membership from the database and inserts an
/// [`OrgMemberPrincipal`] into request extensions for downstream guards and
/// handlers to consume.
///
/// Requires [`AuthenticatedPrincipal`] to already be present (inserted by
/// `authenticate_bearer`). Expects an `org_id` path parameter that is a valid
/// UUID. Returns 401 if the principal is missing; 403 (fail-closed) for any
/// other failure — malformed UUID, unknown org, or non-member.
pub async fn resolve_org_membership(
    State(pool): State<PgPool>,
    Path(params): Path<HashMap<String, String>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let principal = request
        .extensions()
        .get::<AuthenticatedPrincipal>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let org_id_str = params.get("org_id").ok_or(StatusCode::FORBIDDEN)?;
    let org_uuid = Uuid::parse_str(org_id_str).map_err(|_| StatusCode::FORBIDDEN)?;
    let org_id = OrgId(org_uuid);

    let member = dubbridge_db::workspace_repo::get_membership(&pool, org_id, principal.subject_id)
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?
        .ok_or(StatusCode::FORBIDDEN)?;

    request.extensions_mut().insert(OrgMemberPrincipal {
        principal,
        org_id,
        role: member.role,
    });

    Ok(next.run(request).await)
}

/// Guards a route by requiring the [`OrgMemberPrincipal`] already in extensions
/// to satisfy `min_role`. Must be stacked after [`resolve_org_membership`].
///
/// Returns 403 if the principal is absent or its role does not satisfy the
/// minimum — never reveals whether the user is not a member vs. has insufficient
/// role (fail-closed, no enumeration leak).
pub async fn require_org_member(
    State(min_role): State<OrgRole>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let member = request
        .extensions()
        .get::<OrgMemberPrincipal>()
        .ok_or(StatusCode::FORBIDDEN)?;

    if !member.role.satisfies(min_role) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use dubbridge_auth::{AuthenticatedPrincipal, OrgMemberPrincipal};
    use dubbridge_domain::workspace::{OrgId, OrgRole};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::{require_org_member, resolve_org_membership};

    fn fake_pool() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://nobody@localhost/fake")
            .expect("lazy pool")
    }

    fn make_principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(Uuid::new_v4(), std::iter::empty())
    }

    fn make_org_member(role: OrgRole) -> OrgMemberPrincipal {
        OrgMemberPrincipal {
            principal: make_principal(),
            org_id: OrgId::new(),
            role,
        }
    }

    async fn inject_org_member(
        axum::extract::State(member): axum::extract::State<OrgMemberPrincipal>,
        mut request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        request.extensions_mut().insert(member);
        next.run(request).await
    }

    async fn inject_principal(
        axum::extract::State(principal): axum::extract::State<AuthenticatedPrincipal>,
        mut request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        request.extensions_mut().insert(principal);
        next.run(request).await
    }

    // ── require_org_member ────────────────────────────────────────────────────

    #[tokio::test]
    async fn require_org_member_passes_sufficient_role() {
        let member = make_org_member(OrgRole::Admin);

        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(
                OrgRole::Editor,
                require_org_member,
            ))
            .route_layer(middleware::from_fn_with_state(member, inject_org_member));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 64).await.expect("body");
        assert_eq!(&body[..], b"ok");
    }

    #[tokio::test]
    async fn require_org_member_rejects_insufficient_role() {
        let member = make_org_member(OrgRole::Viewer);

        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(
                OrgRole::Editor,
                require_org_member,
            ))
            .route_layer(middleware::from_fn_with_state(member, inject_org_member));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_org_member_rejects_missing_principal() {
        // No OrgMemberPrincipal in extensions — represents a non-member whose
        // resolve step would have rejected the request in production.
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(
                OrgRole::Viewer,
                require_org_member,
            ));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    // ── resolve_org_membership ────────────────────────────────────────────────

    #[tokio::test]
    async fn resolve_rejects_unauthenticated_request() {
        // No AuthenticatedPrincipal → 401 before any DB call.
        let pool = fake_pool();

        let app = Router::new()
            .route("/orgs/{org_id}/test", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(pool, resolve_org_membership));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/orgs/550e8400-e29b-41d4-a716-446655440000/test")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn resolve_rejects_malformed_org_id() {
        // AuthenticatedPrincipal present but org_id is not a valid UUID → 403.
        let pool = fake_pool();
        let principal = make_principal();

        let app = Router::new()
            .route("/orgs/{org_id}/test", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(pool, resolve_org_membership))
            .route_layer(middleware::from_fn_with_state(principal, inject_principal));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/orgs/not-a-uuid/test")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
