// S-130-T1: integration tests for transcription_repo — real DB, no mocks.
use std::env;

use dubbridge_db::transcription_repo::TranscriptArtifactMeta;
use dubbridge_db::{artifact_repo, transcription_repo};
use dubbridge_domain::{
    artifact::{ArtifactRecord, TranscriptionStatus},
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

// HP-1: insert TranscriptText and WordAlignment derived artifacts; list back with correct lineage.
#[tokio::test]
async fn insert_transcript_artifacts_creates_both_kinds_with_correct_lineage() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let t_key = transcript_key(&asset_id.to_string());
    let a_key = alignment_key(&asset_id.to_string());

    let (transcript, alignment) = transcription_repo::insert_transcript_artifacts(
        &pool,
        asset_id,
        source.id,
        TranscriptArtifactMeta {
            storage_key: &t_key,
            size_bytes: 256,
            checksum: "tchk",
        },
        TranscriptArtifactMeta {
            storage_key: &a_key,
            size_bytes: 512,
            checksum: "achk",
        },
    )
    .await
    .expect("insert transcript artifacts");

    use dubbridge_domain::artifact::ArtifactKind;
    assert_eq!(transcript.kind, ArtifactKind::TranscriptText);
    assert_eq!(transcript.parent_artifact_id, source.id);
    assert_eq!(transcript.storage_key, t_key);
    assert_eq!(transcript.checksum, "tchk");

    assert_eq!(alignment.kind, ArtifactKind::WordAlignment);
    assert_eq!(alignment.parent_artifact_id, source.id);
    assert_eq!(alignment.storage_key, a_key);
    assert_eq!(alignment.checksum, "achk");
}

// HP-2: WordAlignment artifact linked to same source artifact as TranscriptText.
#[tokio::test]
async fn both_artifacts_share_same_parent_artifact_id() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    let (transcript, alignment) = transcription_repo::insert_transcript_artifacts(
        &pool,
        asset_id,
        source.id,
        TranscriptArtifactMeta {
            storage_key: &transcript_key(&asset_id.to_string()),
            size_bytes: 10,
            checksum: "chk1",
        },
        TranscriptArtifactMeta {
            storage_key: &alignment_key(&asset_id.to_string()),
            size_bytes: 20,
            checksum: "chk2",
        },
    )
    .await
    .expect("insert");

    assert_eq!(transcript.parent_artifact_id, source.id);
    assert_eq!(alignment.parent_artifact_id, source.id);
    assert_eq!(transcript.parent_artifact_id, alignment.parent_artifact_id);
}

// HP-3: TranscriptionStatus transitions Pending → InProgress → Ready round-trip.
#[tokio::test]
async fn transcription_status_transitions_round_trip() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    for status in [
        TranscriptionStatus::Pending,
        TranscriptionStatus::InProgress,
        TranscriptionStatus::Ready,
    ] {
        transcription_repo::upsert_transcription_status(&pool, asset_id, status.clone(), None)
            .await
            .expect("upsert");

        let got = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get")
            .expect("Some");

        assert_eq!(got.status, status);
        assert_eq!(got.asset_id, asset_id);
    }
}

// HP-4: readiness evidence returns true when both artifact types exist.
#[tokio::test]
async fn readiness_evidence_true_when_both_artifacts_present() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    transcription_repo::insert_transcript_artifacts(
        &pool,
        asset_id,
        source.id,
        TranscriptArtifactMeta {
            storage_key: &transcript_key(&asset_id.to_string()),
            size_bytes: 10,
            checksum: "c1",
        },
        TranscriptArtifactMeta {
            storage_key: &alignment_key(&asset_id.to_string()),
            size_bytes: 20,
            checksum: "c2",
        },
    )
    .await
    .expect("insert");

    let ready = transcription_repo::get_transcription_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness");

    assert!(ready);
}

// EC-1: Failed status persists error_detail and is queryable.
#[tokio::test]
async fn failed_status_persists_error_detail() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    transcription_repo::upsert_transcription_status(
        &pool,
        asset_id,
        TranscriptionStatus::Failed,
        Some("asr worker exited with code 1"),
    )
    .await
    .expect("upsert failed");

    let record = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get")
        .expect("Some");

    assert_eq!(record.status, TranscriptionStatus::Failed);
    assert_eq!(
        record.error_detail.as_deref(),
        Some("asr worker exited with code 1")
    );
}

// EC-2: readiness evidence returns false when only TranscriptText exists.
#[tokio::test]
async fn readiness_evidence_false_when_only_transcript_present() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    use dubbridge_db::preparation_repo::insert_derived_artifact;
    use dubbridge_domain::artifact::{ArtifactKind, DerivedArtifact};

    let transcript = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::TranscriptText,
        transcript_key(&asset_id.to_string()),
        "application/json".into(),
        10,
        "c1".into(),
    );
    insert_derived_artifact(&pool, &transcript)
        .await
        .expect("insert transcript only");

    let ready = transcription_repo::get_transcription_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness");

    assert!(!ready);
}

// EC-3: unknown TranscriptionStatus in DB fails closed (UnknownStoredValue).
#[tokio::test]
async fn get_transcription_status_unknown_value_fails_closed() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    sqlx::query(
        "INSERT INTO asset_transcription_status (asset_id, status, updated_at) VALUES ($1, $2, now())",
    )
    .bind(asset_id.0)
    .bind("superseded")
    .execute(&pool)
    .await
    .expect("raw insert unknown status");

    let err = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect_err("must fail closed on unknown status");

    assert!(
        matches!(
            err,
            dubbridge_db::error::DbError::UnknownStoredValue {
                field: "asset_transcription_status.status",
                ..
            }
        ),
        "unexpected error: {err:?}"
    );
}

// EC-4: get_transcription_status returns None for asset with no status row.
#[tokio::test]
async fn get_transcription_status_returns_none_when_not_initialised() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    let result = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get");

    assert!(result.is_none());
}
