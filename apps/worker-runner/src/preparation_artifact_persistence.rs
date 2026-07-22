use anyhow::{Context, bail};
use dubbridge_db::preparation_repo;
use dubbridge_domain::{artifact::ArtifactRecord, asset::AssetId};
use dubbridge_jobs::PreparationJob;
use dubbridge_media::validate_hls_outputs;
use dubbridge_storage::{StorageAdapter, hls_manifest_key, hls_segment_key, probe_metadata_key};
use sqlx::PgPool;

use crate::{checksum_hex, preparation_runtime::HlsPackageOutput};

#[allow(dead_code)]
pub(crate) async fn load_source_artifact(
    pool: &PgPool,
    job: &PreparationJob,
) -> anyhow::Result<ArtifactRecord> {
    let asset_id = AssetId(job.asset_id);
    let source = preparation_repo::find_source_artifact(pool, asset_id)
        .await
        .context("failed to load source artifact for preparation")?
        .ok_or_else(|| anyhow::anyhow!("source artifact missing for asset {asset_id}"))?;

    if source.id != job.source_artifact_id {
        bail!(
            "preparation job source artifact mismatch for asset {asset_id}: expected {}, found {}",
            job.source_artifact_id,
            source.id
        );
    }

    Ok(source)
}

#[allow(dead_code)]
pub(crate) async fn persist_probe_artifact(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    asset_id_string: &str,
    probe_bytes: &[u8],
) -> anyhow::Result<()> {
    let storage_key = probe_metadata_key(asset_id_string);
    storage
        .put(&storage_key, probe_bytes.to_vec())
        .await
        .with_context(|| format!("failed to store probe metadata at '{storage_key}'"))?;

    preparation_repo::insert_probe_metadata_artifact(
        pool,
        asset_id,
        &storage_key,
        i64::try_from(probe_bytes.len()).context("probe metadata exceeds i64 size limit")?,
        &checksum_hex(probe_bytes),
    )
    .await
    .context("failed to persist probe metadata artifact")?;

    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn persist_hls_artifacts(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    asset_id_string: &str,
    hls_output: &HlsPackageOutput,
) -> anyhow::Result<()> {
    let manifest_raw = std::str::from_utf8(&hls_output.manifest_bytes)
        .context("HLS manifest is not valid UTF-8")?;
    let segment_names = hls_output
        .segments
        .iter()
        .map(|segment| segment.file_name.as_str())
        .collect::<Vec<_>>();
    validate_hls_outputs(manifest_raw, &segment_names).context("HLS output validation failed")?;

    let manifest_key = hls_manifest_key(asset_id_string);
    storage
        .put(&manifest_key, hls_output.manifest_bytes.clone())
        .await
        .with_context(|| format!("failed to store HLS manifest at '{manifest_key}'"))?;

    let mut segment_metadata = Vec::with_capacity(hls_output.segments.len());
    for segment in &hls_output.segments {
        let storage_key = hls_segment_key(asset_id_string, &segment.file_name);
        storage
            .put(&storage_key, segment.bytes.clone())
            .await
            .with_context(|| format!("failed to store HLS segment at '{storage_key}'"))?;
        segment_metadata.push((
            storage_key,
            i64::try_from(segment.bytes.len()).context("HLS segment exceeds i64 size limit")?,
            checksum_hex(&segment.bytes),
        ));
    }

    preparation_repo::insert_hls_artifacts(
        pool,
        asset_id,
        &manifest_key,
        i64::try_from(hls_output.manifest_bytes.len())
            .context("HLS manifest exceeds i64 size limit")?,
        &checksum_hex(&hls_output.manifest_bytes),
        &segment_metadata,
    )
    .await
    .context("failed to persist HLS derived artifacts")?;

    Ok(())
}
