use dubbridge_domain::workspace::{OrgId, OrgRole};

use crate::principal::AuthenticatedPrincipal;

#[derive(Debug, Clone)]
pub struct OrgMemberPrincipal {
    pub principal: AuthenticatedPrincipal,
    pub org_id: OrgId,
    pub role: OrgRole,
}

#[cfg(test)]
mod tests {
    use axum::{Extension, Router, body::Body, extract::Request, http::StatusCode, routing::get};
    use tower::ServiceExt;
    use uuid::Uuid;

    use dubbridge_domain::workspace::{OrgId, OrgRole};

    use crate::principal::AuthenticatedPrincipal;

    use super::OrgMemberPrincipal;

    fn make_principal() -> OrgMemberPrincipal {
        OrgMemberPrincipal {
            principal: AuthenticatedPrincipal::new(Uuid::new_v4(), std::iter::empty()),
            org_id: OrgId::new(),
            role: OrgRole::Editor,
        }
    }

    #[test]
    fn struct_builds_correctly() {
        let p = make_principal();
        assert_eq!(p.role, OrgRole::Editor);
    }

    #[test]
    fn clone_produces_equal_fields() {
        let p = make_principal();
        let cloned = p.clone();
        assert_eq!(p.principal.subject_id, cloned.principal.subject_id);
        assert_eq!(p.org_id, cloned.org_id);
        assert_eq!(p.role, cloned.role);
    }

    #[tokio::test]
    async fn insertable_and_extractable_from_axum_extension() {
        let member = make_principal();
        let subject_id = member.principal.subject_id;
        let org_id = member.org_id;

        let app = Router::new().route(
            "/",
            get(|Extension(m): Extension<OrgMemberPrincipal>| async move {
                format!("{}/{}", m.principal.subject_id, m.org_id)
            }),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .extension(member)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 256)
            .await
            .expect("body");
        let text = String::from_utf8(body.to_vec()).expect("utf8");
        assert_eq!(text, format!("{subject_id}/{org_id}"));
    }
}
