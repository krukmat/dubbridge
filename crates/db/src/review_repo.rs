// S-160-T1c: review repository per ADR-030
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    asset::AssetId,
    review::{
        PublicationRow, PublicationStatus, ReviewDecisionRow, ReviewTask, ReviewTaskId,
        ReviewTaskState, ReviewVerdict, derive_review_state,
    },
    workspace::{OrgId, ProjectId},
};

use crate::error::DbError;

#[derive(Debug, Clone)]
pub struct ReviewTaskWithState {
    pub task: ReviewTask,
    pub state: ReviewTaskState,
}

#[derive(sqlx::FromRow)]
struct ReviewTaskRowDb {
    id: Uuid,
    org_id: Uuid,
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    assignee_subject_id: Option<Uuid>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    assigned_at: Option<OffsetDateTime>,
}

#[derive(sqlx::FromRow)]
struct ReviewDecisionRowDb {
    id: Uuid,
    review_task_id: Uuid,
    verdict: String,
    comment: Option<String>,
    reviewer_subject_id: Uuid,
    happened_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct PublicationRowDb {
    id: Uuid,
    review_task_id: Uuid,
    state: String,
    published_by: Uuid,
    published_at: OffsetDateTime,
}

fn parse_verdict(s: &str) -> Result<ReviewVerdict, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "review_decisions.verdict",
        value: s.to_owned(),
    })
}

fn parse_publication_status(s: &str) -> Result<PublicationStatus, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "publications.state",
        value: s.to_owned(),
    })
}

fn task_from_db(r: ReviewTaskRowDb) -> ReviewTask {
    ReviewTask {
        id: ReviewTaskId(r.id),
        org_id: OrgId(r.org_id),
        project_id: ProjectId(r.project_id),
        asset_id: AssetId(r.asset_id),
        target_language_id: r.target_language_id,
        assignee_subject_id: r.assignee_subject_id,
        created_at: r.created_at,
        updated_at: r.updated_at,
        assigned_at: r.assigned_at,
    }
}

fn decision_from_db(r: ReviewDecisionRowDb) -> Result<ReviewDecisionRow, DbError> {
    Ok(ReviewDecisionRow {
        id: r.id,
        review_task_id: ReviewTaskId(r.review_task_id),
        verdict: parse_verdict(&r.verdict)?,
        comment: r.comment,
        reviewer_subject_id: r.reviewer_subject_id,
        happened_at: r.happened_at,
    })
}

fn publication_from_db(r: PublicationRowDb) -> Result<PublicationRow, DbError> {
    Ok(PublicationRow {
        id: r.id,
        review_task_id: ReviewTaskId(r.review_task_id),
        status: parse_publication_status(&r.state)?,
        published_by: r.published_by,
        published_at: r.published_at,
    })
}

pub async fn insert_review_task(pool: &PgPool, task: &ReviewTask) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO review_tasks (
            id, org_id, project_id, asset_id, target_language_id,
            assignee_subject_id, created_at, updated_at, assigned_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(task.id.0)
    .bind(task.org_id.0)
    .bind(task.project_id.0)
    .bind(task.asset_id.0)
    .bind(task.target_language_id)
    .bind(task.assignee_subject_id)
    .bind(task.created_at)
    .bind(task.updated_at)
    .bind(task.assigned_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn get_review_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<Option<ReviewTask>, DbError> {
    let row = sqlx::query_as::<_, ReviewTaskRowDb>(
        r#"
        SELECT id, org_id, project_id, asset_id, target_language_id,
               assignee_subject_id, created_at, updated_at, assigned_at
        FROM review_tasks
        WHERE id = $1
        "#,
    )
    .bind(review_task_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(task_from_db))
}

pub async fn append_review_decision(pool: &PgPool, row: &ReviewDecisionRow) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO review_decisions (
            id, review_task_id, verdict, comment, reviewer_subject_id, happened_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(row.id)
    .bind(row.review_task_id.0)
    .bind(row.verdict.to_string())
    .bind(&row.comment)
    .bind(row.reviewer_subject_id)
    .bind(row.happened_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn list_review_decisions_for_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<Vec<ReviewDecisionRow>, DbError> {
    let rows = sqlx::query_as::<_, ReviewDecisionRowDb>(
        r#"
        SELECT id, review_task_id, verdict, comment, reviewer_subject_id, happened_at
        FROM review_decisions
        WHERE review_task_id = $1
        ORDER BY happened_at ASC, id ASC
        "#,
    )
    .bind(review_task_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(decision_from_db).collect()
}

pub async fn latest_review_decision(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<Option<ReviewDecisionRow>, DbError> {
    let row = sqlx::query_as::<_, ReviewDecisionRowDb>(
        r#"
        SELECT id, review_task_id, verdict, comment, reviewer_subject_id, happened_at
        FROM review_decisions
        WHERE review_task_id = $1
        ORDER BY happened_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(review_task_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(decision_from_db).transpose()
}

pub async fn latest_review_state(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<ReviewTaskState, DbError> {
    let rows = list_review_decisions_for_task(pool, review_task_id).await?;
    Ok(derive_review_state(&rows))
}

pub async fn list_review_tasks_for_scope(
    pool: &PgPool,
    org_id: OrgId,
    project_id: ProjectId,
    assignee_subject_id: Option<Uuid>,
) -> Result<Vec<ReviewTaskWithState>, DbError> {
    let rows = if let Some(assignee) = assignee_subject_id {
        sqlx::query_as::<_, ReviewTaskRowDb>(
            r#"
            SELECT id, org_id, project_id, asset_id, target_language_id,
                   assignee_subject_id, created_at, updated_at, assigned_at
            FROM review_tasks
            WHERE org_id = $1 AND project_id = $2 AND assignee_subject_id = $3
            ORDER BY created_at
            "#,
        )
        .bind(org_id.0)
        .bind(project_id.0)
        .bind(assignee)
        .fetch_all(pool)
        .await
        .map_err(DbError::QueryFailed)?
    } else {
        sqlx::query_as::<_, ReviewTaskRowDb>(
            r#"
            SELECT id, org_id, project_id, asset_id, target_language_id,
                   assignee_subject_id, created_at, updated_at, assigned_at
            FROM review_tasks
            WHERE org_id = $1 AND project_id = $2
            ORDER BY created_at
            "#,
        )
        .bind(org_id.0)
        .bind(project_id.0)
        .fetch_all(pool)
        .await
        .map_err(DbError::QueryFailed)?
    };

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let task = task_from_db(row);
        let state = latest_review_state(pool, task.id).await?;
        items.push(ReviewTaskWithState { task, state });
    }

    Ok(items)
}

pub async fn insert_publication(pool: &PgPool, row: &PublicationRow) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO publications (id, review_task_id, state, published_by, published_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(row.id)
    .bind(row.review_task_id.0)
    .bind(row.status.to_string())
    .bind(row.published_by)
    .bind(row.published_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn get_publication_for_review_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<Option<PublicationRow>, DbError> {
    let row = sqlx::query_as::<_, PublicationRowDb>(
        r#"
        SELECT id, review_task_id, state, published_by, published_at
        FROM publications
        WHERE review_task_id = $1
        "#,
    )
    .bind(review_task_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(publication_from_db).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_verdict_known_variants_succeed() {
        assert!(matches!(
            parse_verdict("approved"),
            Ok(ReviewVerdict::Approved)
        ));
        assert!(matches!(
            parse_verdict("rejected"),
            Ok(ReviewVerdict::Rejected)
        ));
    }

    #[test]
    fn parse_verdict_unknown_value_fails_closed() {
        let err = parse_verdict("pending").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "review_decisions.verdict",
                ..
            }
        ));
        assert!(err.to_string().contains("pending"));
    }

    #[test]
    fn parse_publication_status_known_variant_succeeds() {
        assert!(matches!(
            parse_publication_status("published"),
            Ok(PublicationStatus::Published)
        ));
    }

    #[test]
    fn parse_publication_status_unknown_value_fails_closed() {
        let err = parse_publication_status("queued").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "publications.state",
                ..
            }
        ));
        assert!(err.to_string().contains("queued"));
    }

    #[test]
    fn decision_from_db_unknown_verdict_fails_closed() {
        let row = ReviewDecisionRowDb {
            id: Uuid::new_v4(),
            review_task_id: Uuid::new_v4(),
            verdict: "pending".to_string(),
            comment: None,
            reviewer_subject_id: Uuid::new_v4(),
            happened_at: OffsetDateTime::now_utc(),
        };

        let err = decision_from_db(row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "review_decisions.verdict",
                ..
            }
        ));
    }

    #[test]
    fn publication_from_db_unknown_state_fails_closed() {
        let row = PublicationRowDb {
            id: Uuid::new_v4(),
            review_task_id: Uuid::new_v4(),
            state: "queued".to_string(),
            published_by: Uuid::new_v4(),
            published_at: OffsetDateTime::now_utc(),
        };

        let err = publication_from_db(row).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "publications.state",
                ..
            }
        ));
    }
}
