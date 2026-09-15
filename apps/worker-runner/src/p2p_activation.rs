// MVP0-P2P P2.T5b: fail-contained P2 activation after S-120 Ready + ASR enqueue.

use std::{env, path::PathBuf};

use anyhow::{Context, bail};
use dubbridge_db::{
    error::DbError,
    p2p_package_seal_repo::persist_sealed_package_evidence,
    p2p_publication_repo::{
        P2pPublicationRecord, ensure_publication_with_outbox, get_publication_by_asset,
        record_sealed_k1, transition_publication_state,
    },
    preparation_repo,
};
use dubbridge_domain::{
    artifact::ArtifactKind,
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use dubbridge_p2p::{
    Zeroizing,
    key_wrap::{WrappedKey, generate_ck, unwrap_ck, wrap_ck},
    package_builder::{PackageFileInput, build_package},
    package_writer::materialize,
};
use dubbridge_storage::{StorageAdapter, hls_prefix};
use sqlx::PgPool;
use uuid::Uuid;

const CIPHERTEXT_ROOT_ENV: &str = "DUBBRIDGE_P2P_CIPHERTEXT_ROOT";
const KEK_HEX_ENV: &str = "DUBBRIDGE_P2P_KEK_HEX";
const KEK_ID_ENV: &str = "DUBBRIDGE_P2P_KEK_ID";
const KEK_VERSION_ENV: &str = "DUBBRIDGE_P2P_KEK_VERSION";

struct ActivationConfig {
    ciphertext_root: PathBuf,
    kek_id: String,
    kek_version: u32,
    kek: Zeroizing<[u8; 32]>,
}

impl ActivationConfig {
    fn from_env() -> anyhow::Result<Option<Self>> {
        let Some(kek_hex) = optional_nonempty_env(KEK_HEX_ENV)? else {
            reject_partial_key_configuration()?;
            return Ok(None);
        };
        let kek_hex = Zeroizing::new(kek_hex);
        let ciphertext_root = PathBuf::from(required_nonempty_env(CIPHERTEXT_ROOT_ENV)?);
        let kek_id = required_nonempty_env(KEK_ID_ENV)?;
        let kek_version = required_nonempty_env(KEK_VERSION_ENV)?
            .parse::<u32>()
            .with_context(|| format!("{KEK_VERSION_ENV} must be a positive integer"))?;
        if kek_version == 0 || kek_version > i32::MAX as u32 {
            bail!("{KEK_VERSION_ENV} is outside the supported range");
        }
        Ok(Some(Self {
            ciphertext_root,
            kek_id,
            kek_version,
            kek: decode_32_byte_hex(&kek_hex)?,
        }))
    }
}

pub(crate) async fn activate_after_transcription(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
) {
    let Some(config) = load_activation_config(asset_id) else {
        return;
    };
    run_activation(pool, storage, asset_id, &config).await;
}

fn load_activation_config(asset_id: AssetId) -> Option<ActivationConfig> {
    match ActivationConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(asset_id = %asset_id, error = %error, "P2 activation configuration rejected");
            None
        }
    }
}

async fn run_activation(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    config: &ActivationConfig,
) {
    if let Err(error) = activate(pool, storage, asset_id, config).await {
        tracing::error!(asset_id = %asset_id, error = %error, "P2 activation failed after S-120 Ready");
    }
}

async fn activate(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    config: &ActivationConfig,
) -> anyhow::Result<()> {
    let publication = load_or_create_publication(pool, asset_id).await?;
    if publication.state != PublicationState::Building {
        return Ok(());
    }

    std::fs::create_dir_all(&config.ciphertext_root)
        .context("failed to create P2 ciphertext root")?;
    let inputs = load_hls_inputs(pool, storage, asset_id).await?;
    let ck = resolve_lineage_ck(pool, &publication, config).await?;
    let package = build_package(
        &asset_id.to_string(),
        &publication.id.to_string(),
        &publication.lineage_id.to_string(),
        &ck,
        &inputs,
    )
    .map_err(|error| anyhow::anyhow!("failed to build P2 ciphertext package: {error:?}"))?;

    let materialized = materialize(&config.ciphertext_root, &package)
        .map_err(|error| anyhow::anyhow!("failed to materialize P2 ciphertext package: {error}"))?;
    persist_sealed_package_evidence(
        pool,
        publication.id,
        publication.lineage_id,
        &package.manifest_digest_sha256,
        &materialized.package_ref,
    )
    .await
    .context("failed to persist P2 sealed package evidence")?;
    advance_to_publish_pending(pool, publication.id).await
}

async fn load_or_create_publication(
    pool: &PgPool,
    asset_id: AssetId,
) -> anyhow::Result<P2pPublicationRecord> {
    if let Some(existing) = get_publication_by_asset(pool, asset_id).await? {
        return ensure_existing_outbox(pool, asset_id, existing).await;
    }

    match ensure_publication_with_outbox(
        pool,
        asset_id,
        P2pPublicationId::new(),
        K1LineageId::new(),
        Uuid::new_v4(),
    )
    .await
    {
        Ok(result) => Ok(result.publication),
        Err(DbError::Conflict) => {
            let existing = get_publication_by_asset(pool, asset_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("P2 publication conflict without an existing row")
                })?;
            ensure_existing_outbox(pool, asset_id, existing).await
        }
        Err(error) => Err(error.into()),
    }
}

async fn ensure_existing_outbox(
    pool: &PgPool,
    asset_id: AssetId,
    existing: P2pPublicationRecord,
) -> anyhow::Result<P2pPublicationRecord> {
    Ok(ensure_publication_with_outbox(
        pool,
        asset_id,
        existing.id,
        existing.lineage_id,
        Uuid::new_v4(),
    )
    .await?
    .publication)
}

async fn load_hls_inputs(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
) -> anyhow::Result<Vec<PackageFileInput>> {
    let artifacts = preparation_repo::list_derived_artifacts(pool, asset_id)
        .await
        .context("failed to load prepared artifacts for P2 activation")?;
    let prefix = hls_prefix(&asset_id.to_string());
    let mut manifest = None;
    let mut segments = Vec::new();

    for artifact in artifacts {
        let is_manifest = artifact.kind == ArtifactKind::HlsManifest;
        let package_path = match artifact.kind {
            ArtifactKind::HlsManifest => "index.m3u8".to_string(),
            ArtifactKind::HlsSegment => artifact
                .storage_key
                .strip_prefix(&prefix)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("HLS segment storage key is outside the asset HLS prefix")
                })?
                .to_string(),
            _ => continue,
        };
        let bytes = storage.get(&artifact.storage_key).await.with_context(|| {
            format!(
                "failed to read prepared HLS object '{}'",
                artifact.storage_key
            )
        })?;
        let input = PackageFileInput {
            path: package_path,
            plaintext: bytes,
        };
        if is_manifest {
            if manifest.replace(input).is_some() {
                bail!("multiple HLS manifests found for P2 activation");
            }
        } else {
            segments.push(input);
        }
    }

    let manifest =
        manifest.ok_or_else(|| anyhow::anyhow!("HLS manifest missing for P2 activation"))?;
    if segments.is_empty() {
        bail!("HLS segments missing for P2 activation");
    }
    segments.sort_by(|left, right| left.path.cmp(&right.path));
    let mut inputs = Vec::with_capacity(segments.len() + 1);
    inputs.push(manifest);
    inputs.extend(segments);
    Ok(inputs)
}

async fn resolve_lineage_ck(
    pool: &PgPool,
    publication: &P2pPublicationRecord,
    config: &ActivationConfig,
) -> anyhow::Result<Zeroizing<[u8; 32]>> {
    match (
        publication.sealed_kek_id.as_deref(),
        publication.sealed_kek_version,
        publication.sealed_nonce.as_deref(),
        publication.sealed_wrapped_ck.as_deref(),
    ) {
        (None, None, None, None) => create_and_persist_lineage_ck(pool, publication, config).await,
        (Some(kek_id), Some(kek_version), Some(nonce), Some(wrapped_ck)) => {
            if kek_id != config.kek_id || kek_version != config.kek_version as i32 {
                bail!("persisted P2 lineage requires a different KEK resolver entry");
            }
            let nonce: [u8; 12] = nonce.try_into().map_err(|_| {
                anyhow::anyhow!("persisted P2 wrapped-key nonce has invalid length")
            })?;
            let wrapped = WrappedKey {
                kek_id: kek_id.to_string(),
                kek_version: config.kek_version,
                nonce,
                ciphertext: wrapped_ck.to_vec(),
            };
            unwrap_ck(&wrapped, &config.kek)
                .map_err(|_| anyhow::anyhow!("failed to unwrap persisted P2 lineage key"))
        }
        _ => bail!("persisted P2 sealed-key metadata is incomplete"),
    }
}

async fn create_and_persist_lineage_ck(
    pool: &PgPool,
    publication: &P2pPublicationRecord,
    config: &ActivationConfig,
) -> anyhow::Result<Zeroizing<[u8; 32]>> {
    let ck = generate_ck();
    let wrapped = wrap_ck(&ck, &config.kek, &config.kek_id, config.kek_version)
        .map_err(|_| anyhow::anyhow!("failed to wrap P2 lineage key"))?;
    record_sealed_k1(
        pool,
        publication.id,
        publication.lineage_id,
        &wrapped.kek_id,
        i32::try_from(wrapped.kek_version).context("P2 KEK version exceeds storage range")?,
        &wrapped.nonce,
        &wrapped.ciphertext,
    )
    .await
    .context("failed to persist P2 sealed-key metadata")?;
    Ok(ck)
}

async fn advance_to_publish_pending(
    pool: &PgPool,
    publication_id: P2pPublicationId,
) -> anyhow::Result<()> {
    match transition_publication_state(pool, publication_id, PublicationState::PublishPending, None)
        .await
    {
        Ok(_) => Ok(()),
        Err(DbError::Conflict) => {
            let current = dubbridge_db::p2p_publication_repo::get_publication(pool, publication_id)
                .await?
                .ok_or(DbError::NotFound)?;
            if matches!(
                current.state,
                PublicationState::PublishPending
                    | PublicationState::Publishing
                    | PublicationState::Reconciling
                    | PublicationState::Ready
            ) {
                Ok(())
            } else {
                Err(DbError::Conflict.into())
            }
        }
        Err(error) => Err(error.into()),
    }
}

fn optional_nonempty_env(name: &str) -> anyhow::Result<Option<String>> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => bail!("{name} must not be empty"),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{name} must be valid UTF-8"),
    }
}

fn required_nonempty_env(name: &str) -> anyhow::Result<String> {
    optional_nonempty_env(name)?.ok_or_else(|| anyhow::anyhow!("{name} is required"))
}

fn reject_partial_key_configuration() -> anyhow::Result<()> {
    for name in [KEK_ID_ENV, KEK_VERSION_ENV] {
        if env::var_os(name).is_some() {
            bail!("{KEK_HEX_ENV} is required when {name} is configured");
        }
    }
    Ok(())
}

fn decode_32_byte_hex(value: &str) -> anyhow::Result<Zeroizing<[u8; 32]>> {
    if value.len() != 64 {
        bail!("{KEK_HEX_ENV} must contain exactly 64 hexadecimal characters");
    }
    let mut output = Zeroizing::new([0_u8; 32]);
    for (index, byte) in output.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&value[start..start + 2], 16).map_err(|_| {
            anyhow::anyhow!("{KEK_HEX_ENV} must contain only hexadecimal characters")
        })?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_32_byte_hex_accepts_exact_key_material() {
        let decoded = decode_32_byte_hex(&"ab".repeat(32)).expect("decode key");
        assert_eq!(*decoded, [0xab; 32]);
    }

    #[test]
    fn decode_32_byte_hex_rejects_invalid_material() {
        assert!(decode_32_byte_hex("00").is_err());
        assert!(decode_32_byte_hex(&"zz".repeat(32)).is_err());
    }
}
