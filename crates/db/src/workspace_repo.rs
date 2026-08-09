// S-100-T1: workspace repository — orgs, members, projects, target languages (ADR-027)
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::asset::{Asset, AssetId, IngestionStatus};
use dubbridge_domain::workspace::{
    OrgId, OrgMember, OrgRole, Organization, Project, ProjectId, parse_org_role,
};

use crate::error::DbError;

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn require_org_role(s: &str) -> Result<OrgRole, DbError> {
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

#[cfg(test)]
#[derive(sqlx::FromRow)]
pub(crate) struct OrgRow {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct OrgMembershipRow {
    id: Uuid,
    name: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    role: String,
}

#[cfg(test)]
pub(crate) fn org_from_row(r: OrgRow) -> Organization {
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
) -> Result<Vec<(Organization, OrgRole)>, DbError> {
    let rows = sqlx::query_as::<_, OrgMembershipRow>(
        r#"
        SELECT o.id, o.name, o.created_at, o.updated_at, m.role
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

    rows.into_iter()
        .map(|row| {
            let role = require_org_role(&row.role)?;
            Ok((
                Organization {
                    id: OrgId(row.id),
                    name: row.name,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                role,
            ))
        })
        .collect()
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
pub(crate) struct MemberRow {
    pub(crate) org_id: Uuid,
    pub(crate) subject_id: Uuid,
    pub(crate) role: String,
    pub(crate) joined_at: OffsetDateTime,
}

pub(crate) fn member_from_row(r: MemberRow) -> Result<OrgMember, DbError> {
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
pub(crate) struct ProjectRow {
    pub(crate) id: Uuid,
    pub(crate) org_id: Uuid,
    pub(crate) name: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

pub(crate) fn project_from_row(r: ProjectRow) -> Project {
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
pub(crate) struct AssetRow {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    pub(crate) uploader_id: Uuid,
    pub(crate) status: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

pub(crate) fn parse_asset_status(s: &str) -> Result<IngestionStatus, DbError> {
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

pub(crate) fn asset_from_row(r: AssetRow) -> Result<Asset, DbError> {
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
