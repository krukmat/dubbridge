// S-100-T1: workspace repository — orgs, members, projects, target languages (ADR-027)
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::asset::{Asset, AssetId, IngestionStatus};
use dubbridge_domain::workspace::{
    OrgId, OrgMember, OrgRole, Organization, Project, ProjectId, TargetLanguage, parse_org_role,
};

use crate::error::DbError;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_org_role(s: &str) -> Result<OrgRole, DbError> {
    parse_org_role(s).ok_or_else(|| DbError::UnknownStoredValue {
        field: "org_members.role",
        value: s.to_owned(),
    })
}

// ── Organizations ─────────────────────────────────────────────────────────────

pub async fn insert_org(pool: &PgPool, org: &Organization) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO organizations (id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(org.id.0)
    .bind(&org.name)
    .bind(org.created_at)
    .bind(org.updated_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn insert_org_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    org: &Organization,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO organizations (id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(org.id.0)
    .bind(&org.name)
    .bind(org.created_at)
    .bind(org.updated_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct OrgRow {
    id: Uuid,
    name: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn org_from_row(r: OrgRow) -> Organization {
    Organization {
        id: OrgId(r.id),
        name: r.name,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

/// Returns all orgs the subject is a member of, ordered by name.
pub async fn list_orgs_for_subject(
    pool: &PgPool,
    subject_id: Uuid,
) -> Result<Vec<Organization>, DbError> {
    let rows = sqlx::query_as::<_, OrgRow>(
        r#"
        SELECT o.id, o.name, o.created_at, o.updated_at
        FROM organizations o
        JOIN org_members m ON m.org_id = o.id
        WHERE m.subject_id = $1
        ORDER BY o.name
        "#,
    )
    .bind(subject_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(rows.into_iter().map(org_from_row).collect())
}

// ── Members ───────────────────────────────────────────────────────────────────

pub async fn add_org_member(pool: &PgPool, member: &OrgMember) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO org_members (org_id, subject_id, role, joined_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (org_id, subject_id) DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(member.org_id.0)
    .bind(member.subject_id)
    .bind(member.role.to_string())
    .bind(member.joined_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn add_org_member_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    member: &OrgMember,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO org_members (org_id, subject_id, role, joined_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (org_id, subject_id) DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(member.org_id.0)
    .bind(member.subject_id)
    .bind(member.role.to_string())
    .bind(member.joined_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    org_id: Uuid,
    subject_id: Uuid,
    role: String,
    joined_at: OffsetDateTime,
}

fn member_from_row(r: MemberRow) -> Result<OrgMember, DbError> {
    Ok(OrgMember {
        org_id: OrgId(r.org_id),
        subject_id: r.subject_id,
        role: require_org_role(&r.role)?,
        joined_at: r.joined_at,
    })
}

/// Returns the membership row for a specific (org, subject) pair, or None if not a member.
pub async fn get_membership(
    pool: &PgPool,
    org_id: OrgId,
    subject_id: Uuid,
) -> Result<Option<OrgMember>, DbError> {
    let row = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT org_id, subject_id, role, joined_at
        FROM org_members
        WHERE org_id = $1 AND subject_id = $2
        "#,
    )
    .bind(org_id.0)
    .bind(subject_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(member_from_row).transpose()
}

pub async fn list_org_members(pool: &PgPool, org_id: OrgId) -> Result<Vec<OrgMember>, DbError> {
    let rows = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT org_id, subject_id, role, joined_at
        FROM org_members
        WHERE org_id = $1
        ORDER BY joined_at
        "#,
    )
    .bind(org_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(member_from_row).collect()
}

// ── Projects ──────────────────────────────────────────────────────────────────

pub async fn insert_project(pool: &PgPool, project: &Project) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO projects (id, org_id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(project.id.0)
    .bind(project.org_id.0)
    .bind(&project.name)
    .bind(project.created_at)
    .bind(project.updated_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn insert_project_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &Project,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO projects (id, org_id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(project.id.0)
    .bind(project.org_id.0)
    .bind(&project.name)
    .bind(project.created_at)
    .bind(project.updated_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: Uuid,
    org_id: Uuid,
    name: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn project_from_row(r: ProjectRow) -> Project {
    Project {
        id: ProjectId(r.id),
        org_id: OrgId(r.org_id),
        name: r.name,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

pub async fn list_projects_for_org(pool: &PgPool, org_id: OrgId) -> Result<Vec<Project>, DbError> {
    let rows = sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, org_id, name, created_at, updated_at
        FROM projects
        WHERE org_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(rows.into_iter().map(project_from_row).collect())
}

pub async fn get_project(pool: &PgPool, project_id: ProjectId) -> Result<Option<Project>, DbError> {
    let row = sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, org_id, name, created_at, updated_at
        FROM projects WHERE id = $1
        "#,
    )
    .bind(project_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(project_from_row))
}

// ── Project assets ────────────────────────────────────────────────────────────

/// Links an asset to a project after verifying the caller owns the asset.
/// Returns DbError::NotFound (as 403-safe) if the asset does not exist or is not
/// owned by caller_subject_id — no ownership information is leaked to the caller.
pub async fn link_asset_to_project(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    caller_subject_id: Uuid,
) -> Result<(), DbError> {
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT uploader_id FROM assets WHERE id = $1")
        .bind(asset_id.0)
        .fetch_optional(pool)
        .await
        .map_err(DbError::QueryFailed)?;

    match owner {
        Some(uid) if uid == caller_subject_id => {}
        _ => return Err(DbError::NotFound),
    }

    sqlx::query(
        r#"
        INSERT INTO project_assets (project_id, asset_id)
        VALUES ($1, $2)
        ON CONFLICT (project_id, asset_id) DO NOTHING
        "#,
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn link_asset_to_project_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: ProjectId,
    asset_id: AssetId,
    caller_subject_id: Uuid,
) -> Result<(), DbError> {
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT uploader_id FROM assets WHERE id = $1")
        .bind(asset_id.0)
        .fetch_optional(&mut **tx)
        .await
        .map_err(DbError::QueryFailed)?;

    match owner {
        Some(uid) if uid == caller_subject_id => {}
        _ => return Err(DbError::NotFound),
    }

    sqlx::query(
        r#"
        INSERT INTO project_assets (project_id, asset_id)
        VALUES ($1, $2)
        ON CONFLICT (project_id, asset_id) DO NOTHING
        "#,
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn unlink_asset_from_project(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM project_assets WHERE project_id = $1 AND asset_id = $2")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .map_err(DbError::QueryFailed)?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct AssetRow {
    id: Uuid,
    title: String,
    uploader_id: Uuid,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn parse_asset_status(s: &str) -> Result<IngestionStatus, DbError> {
    match s {
        "pending" => Ok(IngestionStatus::Pending),
        "finalized" => Ok(IngestionStatus::Finalized),
        "rejected_missing_rights" => Ok(IngestionStatus::RejectedMissingRights),
        "rejected_missing_uploader_context" => Ok(IngestionStatus::RejectedMissingUploaderContext),
        other => Err(DbError::UnknownStoredValue {
            field: "assets.status",
            value: other.to_owned(),
        }),
    }
}

fn asset_from_row(r: AssetRow) -> Result<Asset, DbError> {
    Ok(Asset {
        id: AssetId(r.id),
        title: r.title,
        uploader_id: r.uploader_id,
        status: parse_asset_status(&r.status)?,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}

/// Returns assets linked to a project, ordered by created_at DESC.
/// uploader_id on each asset is unchanged — assets are not reassigned (ADR-023).
pub async fn list_assets_for_project(
    pool: &PgPool,
    project_id: ProjectId,
) -> Result<Vec<Asset>, DbError> {
    let rows = sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT a.id, a.title, a.uploader_id, a.status, a.created_at, a.updated_at
        FROM assets a
        JOIN project_assets pa ON pa.asset_id = a.id
        WHERE pa.project_id = $1
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(project_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(asset_from_row).collect()
}

// ── Target languages ──────────────────────────────────────────────────────────

/// Inserts a target language for a project. If the (project_id, target_lang) pair
/// already exists, updates source_lang to allow corrections.
pub async fn upsert_target_language(pool: &PgPool, tl: &TargetLanguage) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO target_languages (id, project_id, source_lang, target_lang, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (project_id, target_lang)
        DO UPDATE SET source_lang = EXCLUDED.source_lang
        "#,
    )
    .bind(tl.id)
    .bind(tl.project_id.0)
    .bind(&tl.source_lang)
    .bind(&tl.target_lang)
    .bind(tl.created_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn upsert_target_language_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tl: &TargetLanguage,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO target_languages (id, project_id, source_lang, target_lang, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (project_id, target_lang)
        DO UPDATE SET source_lang = EXCLUDED.source_lang
        "#,
    )
    .bind(tl.id)
    .bind(tl.project_id.0)
    .bind(&tl.source_lang)
    .bind(&tl.target_lang)
    .bind(tl.created_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn delete_target_languages_for_project_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: ProjectId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM target_languages WHERE project_id = $1")
        .bind(project_id.0)
        .execute(&mut **tx)
        .await
        .map_err(DbError::QueryFailed)?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct TargetLanguageRow {
    id: Uuid,
    project_id: Uuid,
    source_lang: String,
    target_lang: String,
    created_at: OffsetDateTime,
}

pub async fn list_target_languages(
    pool: &PgPool,
    project_id: ProjectId,
) -> Result<Vec<TargetLanguage>, DbError> {
    let rows = sqlx::query_as::<_, TargetLanguageRow>(
        r#"
        SELECT id, project_id, source_lang, target_lang, created_at
        FROM target_languages
        WHERE project_id = $1
        ORDER BY target_lang
        "#,
    )
    .bind(project_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(rows
        .into_iter()
        .map(|r| TargetLanguage {
            id: r.id,
            project_id: ProjectId(r.project_id),
            source_lang: r.source_lang,
            target_lang: r.target_lang,
            created_at: r.created_at,
        })
        .collect())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_org_role_known_variants_succeed() {
        assert!(matches!(require_org_role("owner"), Ok(OrgRole::Owner)));
        assert!(matches!(require_org_role("admin"), Ok(OrgRole::Admin)));
        assert!(matches!(require_org_role("editor"), Ok(OrgRole::Editor)));
        assert!(matches!(
            require_org_role("reviewer"),
            Ok(OrgRole::Reviewer)
        ));
        assert!(matches!(require_org_role("viewer"), Ok(OrgRole::Viewer)));
    }

    #[test]
    fn require_org_role_unknown_value_fails_closed() {
        let err = require_org_role("superuser").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "org_members.role",
                ..
            }
        ));
        assert!(err.to_string().contains("superuser"));
    }

    #[test]
    fn require_org_role_empty_string_fails_closed() {
        assert!(require_org_role("").is_err());
    }

    #[test]
    fn parse_asset_status_known_variants_succeed() {
        assert!(matches!(
            parse_asset_status("pending"),
            Ok(IngestionStatus::Pending)
        ));
        assert!(matches!(
            parse_asset_status("finalized"),
            Ok(IngestionStatus::Finalized)
        ));
    }

    #[test]
    fn parse_asset_status_unknown_value_fails_closed() {
        let err = parse_asset_status("processing").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "assets.status",
                ..
            }
        ));
    }
}
