//! Local PostgreSQL authorization for the dormant DubBridge -> Fenix client.
//! No API route or worker currently calls this module.

use std::future::Future;

use dubbridge_connectors::fenix_s2s::{
    DubBridgeActor, FenixClientError, FenixScopeAuthorizer, TenantSelection,
};
use dubbridge_domain::workspace::{OrgRole, parse_org_role};
use sqlx::PgPool;

/// The minimum local organization role is selected by the trusted caller for
/// a future *registered* capability. No capability or HTTP route is enabled
/// here. Authorization is checked against the live PostgreSQL membership and
/// project-to-organization relation, never against an untrusted JSON claim.
pub struct PgFenixScopeAuthorizer {
    pool: PgPool,
    minimum_role: OrgRole,
}

impl PgFenixScopeAuthorizer {
    pub fn new(pool: PgPool, minimum_role: OrgRole) -> Self {
        Self { pool, minimum_role }
    }

    async fn check_scope(
        &self,
        actor: &DubBridgeActor,
        requested: &TenantSelection,
    ) -> Result<TenantSelection, FenixClientError> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM org_members WHERE subject_id = $1 AND org_id = $2",
        )
        .bind(actor.user_id)
        .bind(requested.org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| FenixClientError::Denied)?;

        let parsed_role = role
            .as_deref()
            .and_then(parse_org_role)
            .ok_or(FenixClientError::Denied)?;
        if !parsed_role.satisfies(self.minimum_role) {
            return Err(FenixClientError::Denied);
        }

        if let Some(project_id) = requested.project_id {
            let valid_project: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND org_id = $2)",
            )
            .bind(project_id)
            .bind(requested.org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| FenixClientError::Denied)?;
            if !valid_project {
                return Err(FenixClientError::Denied);
            }
        }
        Ok(*requested)
    }
}

impl FenixScopeAuthorizer for PgFenixScopeAuthorizer {
    fn authorize_scope<'a>(
        &'a self,
        actor: &'a DubBridgeActor,
        requested: &'a TenantSelection,
    ) -> impl Future<Output = Result<TenantSelection, FenixClientError>> + Send + 'a {
        self.check_scope(actor, requested)
    }
}
