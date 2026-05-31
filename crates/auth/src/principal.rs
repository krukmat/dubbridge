use std::collections::BTreeSet;

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    pub subject_id: Uuid,
    scopes: BTreeSet<String>,
}

impl AuthenticatedPrincipal {
    pub fn new(subject_id: Uuid, scopes: impl IntoIterator<Item = String>) -> Self {
        let scopes = scopes
            .into_iter()
            .map(|scope| scope.trim().to_string())
            .filter(|scope| !scope.is_empty())
            .collect();

        Self { subject_id, scopes }
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope)
    }

    pub fn scopes(&self) -> &BTreeSet<String> {
        &self.scopes
    }
}
