// S-100-T1: workspace domain — orgs, members, projects, target languages (ADR-027)
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

// ── Identifiers ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrgId(pub Uuid);

impl OrgId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OrgId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OrgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Role enum — strict decode, fail-closed (ADR-027) ─────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Viewer,
    Reviewer,
    Editor,
    Admin,
    Owner,
}

impl OrgRole {
    /// Returns true if self satisfies the minimum required role.
    pub fn satisfies(&self, min_role: OrgRole) -> bool {
        *self >= min_role
    }
}

impl std::fmt::Display for OrgRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Reviewer => "reviewer",
            Self::Viewer => "viewer",
        };
        write!(f, "{s}")
    }
}

/// Fail-closed parse — unknown values are never coerced to a default role.
pub fn parse_org_role(s: &str) -> Option<OrgRole> {
    match s {
        "owner" => Some(OrgRole::Owner),
        "admin" => Some(OrgRole::Admin),
        "editor" => Some(OrgRole::Editor),
        "reviewer" => Some(OrgRole::Reviewer),
        "viewer" => Some(OrgRole::Viewer),
        _ => None,
    }
}

// ── Aggregates ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrgId,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Organization {
    pub fn new(name: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: OrgId::new(),
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    pub org_id: OrgId,
    pub subject_id: Uuid,
    pub role: OrgRole,
    pub joined_at: OffsetDateTime,
}

impl OrgMember {
    pub fn new(org_id: OrgId, subject_id: Uuid, role: OrgRole) -> Self {
        Self {
            org_id,
            subject_id,
            role,
            joined_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub org_id: OrgId,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Project {
    pub fn new(org_id: OrgId, name: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: ProjectId::new(),
            org_id,
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLanguage {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: OffsetDateTime,
}

impl TargetLanguage {
    pub fn new(project_id: ProjectId, source_lang: String, target_lang: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            source_lang,
            target_lang,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn org_role_display_all_variants() {
        assert_eq!(OrgRole::Owner.to_string(), "owner");
        assert_eq!(OrgRole::Admin.to_string(), "admin");
        assert_eq!(OrgRole::Editor.to_string(), "editor");
        assert_eq!(OrgRole::Reviewer.to_string(), "reviewer");
        assert_eq!(OrgRole::Viewer.to_string(), "viewer");
    }

    #[test]
    fn parse_org_role_known_variants_succeed() {
        assert_eq!(parse_org_role("owner"), Some(OrgRole::Owner));
        assert_eq!(parse_org_role("admin"), Some(OrgRole::Admin));
        assert_eq!(parse_org_role("editor"), Some(OrgRole::Editor));
        assert_eq!(parse_org_role("reviewer"), Some(OrgRole::Reviewer));
        assert_eq!(parse_org_role("viewer"), Some(OrgRole::Viewer));
    }

    #[test]
    fn parse_org_role_unknown_value_fails_closed() {
        assert_eq!(parse_org_role("superuser"), None);
        assert_eq!(parse_org_role(""), None);
        assert_eq!(parse_org_role("OWNER"), None);
    }

    #[test]
    fn org_role_ordering_satisfies_hierarchy() {
        // Owner >= all
        assert!(OrgRole::Owner.satisfies(OrgRole::Viewer));
        assert!(OrgRole::Owner.satisfies(OrgRole::Owner));
        // Viewer only satisfies Viewer
        assert!(OrgRole::Viewer.satisfies(OrgRole::Viewer));
        assert!(!OrgRole::Viewer.satisfies(OrgRole::Reviewer));
        assert!(!OrgRole::Viewer.satisfies(OrgRole::Admin));
        // Admin does not satisfy Owner
        assert!(!OrgRole::Admin.satisfies(OrgRole::Owner));
        // Editor satisfies Reviewer and Viewer but not Admin
        assert!(OrgRole::Editor.satisfies(OrgRole::Reviewer));
        assert!(OrgRole::Editor.satisfies(OrgRole::Viewer));
        assert!(!OrgRole::Editor.satisfies(OrgRole::Admin));
    }

    #[test]
    fn organization_new_sets_name_and_timestamps() {
        let org = Organization::new("Acme".to_string());
        assert_eq!(org.name, "Acme");
        assert_eq!(org.created_at, org.updated_at);
    }

    #[test]
    fn project_new_sets_org_id_and_name() {
        let org_id = OrgId::new();
        let project = Project::new(org_id, "Season 1".to_string());
        assert_eq!(project.org_id, org_id);
        assert_eq!(project.name, "Season 1");
    }

    #[test]
    fn org_member_new_sets_role() {
        let org_id = OrgId::new();
        let subject = Uuid::new_v4();
        let member = OrgMember::new(org_id, subject, OrgRole::Reviewer);
        assert_eq!(member.role, OrgRole::Reviewer);
        assert_eq!(member.subject_id, subject);
    }

    #[test]
    fn target_language_new_stores_bcp47_codes() {
        let project_id = ProjectId::new();
        let tl = TargetLanguage::new(project_id, "en".to_string(), "es-ES".to_string());
        assert_eq!(tl.source_lang, "en");
        assert_eq!(tl.target_lang, "es-ES");
    }

    #[test]
    fn org_id_display_matches_inner_uuid() {
        let id = OrgId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn project_id_display_matches_inner_uuid() {
        let id = ProjectId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }
}
