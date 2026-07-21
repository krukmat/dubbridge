// S-140-T1c-ii: integration tests for subtitle_repo — real DB, no mocks.
use std::env;

use dubbridge_db::transcription_repo::TranscriptArtifactMeta;
use dubbridge_db::{artifact_repo, subtitle_repo, transcription_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, SubtitleStatus},
    asset::AssetId,
};
use dubbridge_storage::{alignment_key, transcript_key};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId::new();
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("test-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");
    asset_id
}

async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
    let record = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{}/source.mp4", asset_id),
        "video/mp4".into(),
        1_000_000,
        "deadbeef".into(),
    );
    artifact_repo::insert_artifact_record(pool, &record)
        .await
        .expect("insert source artifact");
    record
}

/// Inserts a source artifact plus its TranscriptText/WordAlignment pair,
/// returning the WordAlignment artifact id (the only valid subtitle parent
/// per D1a).
async fn insert_word_alignment_parent(pool: &PgPool, asset_id: AssetId) -> Uuid {
    let source = insert_source_artifact(pool, asset_id).await;
    let (_transcript, alignment) = transcription_repo::insert_transcript_artifacts(
        pool,
        asset_id,
        source.id,
        TranscriptArtifactMeta {
            storage_key: &transcript_key(&asset_id.to_string()),
            size_bytes: 256,
            checksum: "transcriptsum",
        },
        TranscriptArtifactMeta {
            storage_key: &alignment_key(&asset_id.to_string()),
            size_bytes: 128,
            checksum: "alignmentsum",
        },
    )
    .await
    .expect("insert transcript artifacts");
    alignment.id
}

// HP-1: insert a subtitle artifact linked to a WordAlignment parent.
#[tokio::test]
async fn insert_subtitle_artifact_links_to_word_alignment_parent() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    let inserted = subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "subtitlesum",
    )
    .await
    .expect("insert subtitle artifact");

    assert_eq!(inserted.asset_id, asset_id);
    assert_eq!(inserted.parent_artifact_id, parent_id);
    assert_eq!(inserted.kind, ArtifactKind::Subtitle);
    assert_eq!(inserted.content_type, "text/vtt");
    assert_eq!(inserted.size_bytes, 64);
    assert_eq!(inserted.checksum, "subtitlesum");
}

// Artifact HP: distinct parents (e.g. two languages' WordAlignment artifacts) each accept a subtitle.
#[tokio::test]
async fn insert_subtitle_artifact_allows_distinct_parents() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_a = insert_word_alignment_parent(&pool, asset_id).await;
    let parent_b = insert_word_alignment_parent(&pool, asset_id).await;

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_a,
        &format!("subtitles/{asset_id}/a.vtt"),
        "text/vtt",
        64,
        "suma",
    )
    .await
    .expect("insert subtitle for parent a");

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_b,
        &format!("subtitles/{asset_id}/b.vtt"),
        "text/vtt",
        64,
        "sumb",
    )
    .await
    .expect("insert subtitle for parent b");
}

// Artifact HP: the persisted row round-trips via artifact_repo lookup (not just the returned value).
#[tokio::test]
async fn insert_subtitle_artifact_persists_row_visible_to_listing() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    let inserted = subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "subtitlesum",
    )
    .await
    .expect("insert subtitle artifact");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM artifact_records WHERE id = $1 AND kind = 'subtitle'",
    )
    .bind(inserted.id)
    .fetch_one(&pool)
    .await
    .expect("count subtitle row");
    assert_eq!(count, 1);
}

// EC-4: duplicate subtitle insert for the same (asset_id, parent_artifact_id) is rejected atomically.
#[tokio::test]
async fn insert_subtitle_artifact_rejects_duplicate_for_same_parent() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "first",
    )
    .await
    .expect("first insert succeeds");

    let err = subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en-2.vtt"),
        "text/vtt",
        64,
        "second",
    )
    .await
    .expect_err("duplicate subtitle insert must fail");

    assert!(matches!(err, dubbridge_db::error::DbError::Conflict));
}

// EC-4b: concurrent duplicate inserts for the same (asset_id, parent_artifact_id) race
// against the DB's unique index rather than a check-then-insert; exactly one must win.
//
// Uses a multi-thread runtime + tokio::spawn (not tokio::join! on the default
// current-thread runtime) so the two inserts run on separate OS threads and
// genuinely race, rather than merely interleaving cooperatively at .await
// points. Flagged by phase-2 review (peer-code-review-S-140-T1c-ii-v4.json).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn insert_subtitle_artifact_rejects_concurrent_duplicate() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let key_a = format!("subtitles/{asset_id}/racer-a.vtt");
    let key_b = format!("subtitles/{asset_id}/racer-b.vtt");

    let task_a = tokio::spawn(async move {
        subtitle_repo::insert_subtitle_artifact(
            &pool_a, asset_id, parent_id, &key_a, "text/vtt", 64, "racer-a",
        )
        .await
    });
    let task_b = tokio::spawn(async move {
        subtitle_repo::insert_subtitle_artifact(
            &pool_b, asset_id, parent_id, &key_b, "text/vtt", 64, "racer-b",
        )
        .await
    });

    let result_a = task_a.await.expect("task a did not panic");
    let result_b = task_b.await.expect("task b did not panic");

    let successes = [&result_a, &result_b]
        .into_iter()
        .filter(|r| r.is_ok())
        .count();
    let conflicts = [&result_a, &result_b]
        .into_iter()
        .filter(|r| matches!(r, Err(dubbridge_db::error::DbError::Conflict)))
        .count();

    assert_eq!(successes, 1, "exactly one concurrent insert must succeed");
    assert_eq!(conflicts, 1, "the other must fail with Conflict");
}

// EC-5: insert with a non-existent parent_artifact_id is rejected via FK violation
// (SQLSTATE 23503), mapped to DbError::QueryFailed per the task card — this repo
// has no dedicated FK-violation variant, but the wrapped sqlx::Error still
// carries the SQLSTATE detail for callers who need to distinguish it from EC-4's
// DbError::Conflict (unique violation).
#[tokio::test]
async fn insert_subtitle_artifact_rejects_missing_parent() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let missing_parent = Uuid::new_v4();

    let err = subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        missing_parent,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "subtitlesum",
    )
    .await
    .expect_err("missing parent artifact must fail");

    assert!(matches!(err, dubbridge_db::error::DbError::QueryFailed(_)));
}

// HP-2: subtitle status transitions Pending -> InProgress -> Ready round-trip.
#[tokio::test]
async fn subtitle_status_transitions_succeed() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    for status in [
        SubtitleStatus::Pending,
        SubtitleStatus::InProgress,
        SubtitleStatus::Ready,
    ] {
        subtitle_repo::upsert_subtitle_status(&pool, asset_id, status.clone(), None)
            .await
            .expect("upsert");

        let got = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get")
            .expect("Some");

        assert_eq!(got.status, status);
    }
}

// EC-1: failed status persists error_detail and remains queryable.
#[tokio::test]
async fn failed_subtitle_status_persists_error_detail() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    subtitle_repo::upsert_subtitle_status(
        &pool,
        asset_id,
        SubtitleStatus::Failed,
        Some("alignment source missing"),
    )
    .await
    .expect("upsert failed");

    let record = subtitle_repo::get_subtitle_status(&pool, asset_id)
        .await
        .expect("get")
        .expect("Some");

    assert_eq!(record.status, SubtitleStatus::Failed);
    assert_eq!(
        record.error_detail.as_deref(),
        Some("alignment source missing")
    );
}

// EC-2: get_subtitle_status returns None for an asset with no row.
#[tokio::test]
async fn get_subtitle_status_returns_none_when_not_initialised() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    let result = subtitle_repo::get_subtitle_status(&pool, asset_id)
        .await
        .expect("get");

    assert!(result.is_none());
}

// HP-3: readiness evidence is true only when the subtitle artifact exists AND status is Ready.
#[tokio::test]
async fn subtitle_readiness_evidence_true_only_when_artifact_and_status_ready() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "subtitlesum",
    )
    .await
    .expect("insert subtitle artifact");

    subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::InProgress, None)
        .await
        .expect("upsert in progress");

    assert!(
        !subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness"),
        "artifact exists but status is not Ready"
    );

    subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Ready, None)
        .await
        .expect("upsert ready");

    assert!(
        subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness"),
        "artifact exists and status is Ready"
    );
}

// EC-3: readiness evidence is false when no subtitle artifact row exists, regardless of status.
#[tokio::test]
async fn subtitle_readiness_evidence_false_without_artifact_even_if_status_ready() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Ready, None)
        .await
        .expect("upsert ready");

    assert!(
        !subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness"),
        "no subtitle artifact exists yet"
    );
}

// EC-3a: readiness evidence is false when a subtitle artifact exists but the
// persisted subtitle status is Failed — an artifact alone does not imply readiness.
#[tokio::test]
async fn subtitle_readiness_evidence_false_when_artifact_exists_and_status_failed() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_id = insert_word_alignment_parent(&pool, asset_id).await;

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_id,
        &format!("subtitles/{asset_id}/en.vtt"),
        "text/vtt",
        64,
        "subtitlesum",
    )
    .await
    .expect("insert subtitle artifact");

    subtitle_repo::upsert_subtitle_status(
        &pool,
        asset_id,
        SubtitleStatus::Failed,
        Some("alignment source missing"),
    )
    .await
    .expect("upsert failed status");

    assert!(
        !subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness"),
        "artifact exists but status is Failed"
    );
}

// EC-3b: readiness evidence with multiple subtitle artifacts for one asset
// (e.g. distinct WordAlignment parents / languages) reflects the known
// single-status-row limitation: any subtitle artifact existing while the
// shared asset-level status is Ready is sufficient, even if a specific
// language's artifact is the one under test. This is the documented,
// deliberately-deferred behavior from
// crates/db/src/subtitle_repo.rs::get_subtitle_readiness_evidence, not a bug
// this task fixes — captured here so a future per-language status change
// has an explicit test to update.
#[tokio::test]
async fn subtitle_readiness_evidence_true_with_multiple_artifacts_and_asset_level_ready() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let parent_a = insert_word_alignment_parent(&pool, asset_id).await;
    let parent_b = insert_word_alignment_parent(&pool, asset_id).await;

    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_a,
        &format!("subtitles/{asset_id}/a.vtt"),
        "text/vtt",
        64,
        "suma",
    )
    .await
    .expect("insert subtitle for parent a");
    subtitle_repo::insert_subtitle_artifact(
        &pool,
        asset_id,
        parent_b,
        &format!("subtitles/{asset_id}/b.vtt"),
        "text/vtt",
        64,
        "sumb",
    )
    .await
    .expect("insert subtitle for parent b");

    subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Ready, None)
        .await
        .expect("upsert ready");

    assert!(
        subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness"),
        "asset-level status is Ready and at least one subtitle artifact exists"
    );
}
