use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::workspace::{ProjectId, TargetLanguage};

#[derive(sqlx::FromRow)]
pub(crate) struct TargetLanguageRow {
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

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct AssetSubtitleRouteRow {
    project_id: Uuid,
    target_lang: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetSubtitleRoute {
    pub project_id: ProjectId,
    pub target_language: String,
}

/// Return the `source_lang` for the asset's project (from any `target_languages` row),
/// or `None` if the asset is not linked to a project or the project has no target-language row.
pub async fn get_source_language_for_asset(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<String>, DbError> {
    let lang: Option<String> = sqlx::query_scalar(
        r#"
        SELECT tl.source_lang
        FROM target_languages tl
        JOIN project_assets pa ON pa.project_id = tl.project_id
        WHERE pa.asset_id = $1
        LIMIT 1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(lang)
}

/// Return the asset project plus the deterministic first subtitle target for enqueue.
///
/// This uses `COLLATE "C"` so cross-environment ordering stays byte-stable.
pub async fn get_asset_subtitle_route(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<AssetSubtitleRoute>, DbError> {
    let row = sqlx::query_as::<_, AssetSubtitleRouteRow>(
        r#"
        SELECT pa.project_id, tl.target_lang
        FROM project_assets pa
        JOIN target_languages tl ON tl.project_id = pa.project_id
        WHERE pa.asset_id = $1
        ORDER BY tl.target_lang COLLATE "C"
        LIMIT 1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(|row| AssetSubtitleRoute {
        project_id: ProjectId(row.project_id),
        target_language: row.target_lang,
    }))
}
