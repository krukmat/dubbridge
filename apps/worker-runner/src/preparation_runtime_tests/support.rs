use std::{env, sync::Arc};

use async_trait::async_trait;
use dubbridge_db::{artifact_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactRecord, PreparationStatus},
    asset::AssetId,
};
use dubbridge_media::canonical_ffprobe_json;
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::preparation_runtime::{HlsPackageOutput, HlsSegmentOutput, PreparationExecutor};

pub(super) async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    sqlx::query(
        "TRUNCATE TABLE pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await
    .expect("truncate");
    Some(pool)
}

pub(super) async fn insert_asset(pool: &PgPool) -> AssetId {
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

pub(super) async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
    let record = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{asset_id}/source.mp4"),
        "video/mp4".into(),
        1024,
        "sourcesum".into(),
    );
    artifact_repo::insert_artifact_record(pool, &record)
        .await
        .expect("insert source artifact");
    record
}

pub(super) fn valid_probe_bytes() -> Vec<u8> {
    canonical_ffprobe_json(
        r#"{
          "streams":[
            {"codec_type":"video","codec_name":"h264"},
            {"codec_type":"audio","codec_name":"aac"}
          ],
          "format":{"format_name":"mp4","duration":"10.000000"}
        }"#,
    )
    .expect("canonical probe bytes")
}

pub(super) fn valid_hls_output() -> HlsPackageOutput {
    HlsPackageOutput {
        manifest_bytes: b"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:6
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:6.0,
segment_00000.ts
#EXT-X-ENDLIST
"
        .to_vec(),
        segments: vec![HlsSegmentOutput {
            file_name: "segment_00000.ts".to_string(),
            bytes: b"segment-bytes".to_vec(),
        }],
    }
}

pub(super) async fn assert_status(pool: &PgPool, asset_id: AssetId, expected: PreparationStatus) {
    let status = preparation_repo::get_preparation_status(pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    assert_eq!(status.status, expected);
}

pub(super) struct FakePreparationExecutor {
    pub(super) pool: PgPool,
    pub(super) asset_id: AssetId,
    pub(super) stage_log: Arc<Mutex<Vec<&'static str>>>,
    pub(super) probe_result: Result<Vec<u8>, String>,
    pub(super) hls_result: Result<HlsPackageOutput, String>,
}

#[async_trait]
impl PreparationExecutor for FakePreparationExecutor {
    async fn extract_probe_metadata(&self, _source_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
        assert_status(&self.pool, self.asset_id, PreparationStatus::InProgress).await;
        self.stage_log.lock().await.push("probe");
        self.probe_result.clone().map_err(anyhow::Error::msg)
    }

    async fn transcode_hls(&self, _source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput> {
        assert_status(&self.pool, self.asset_id, PreparationStatus::InProgress).await;
        self.stage_log.lock().await.push("hls");
        self.hls_result.clone().map_err(anyhow::Error::msg)
    }
}
