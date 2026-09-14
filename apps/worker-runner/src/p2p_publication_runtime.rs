// MVP0-P2P P2.T4e: PostgreSQL-driven publication dispatcher/reconciler runtime.
//
// The runtime is intentionally inert unless the P2P Availability Node URL is
// configured. Once enabled, PostgreSQL remains authority: every loop iteration
// asks the dispatcher for one durable obligation, and stale leases naturally
// re-enter through the T4b claim query without a queue dependency.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use anyhow::Context;
use dubbridge_connectors::p2p_availability::AvailabilityPublicationClient;
use dubbridge_jobs::p2p_publication_job::{DispatchTick, P2pPublicationDispatcher};
use sqlx::PgPool;
use time::Duration;

const URL_ENV: &str = "DUBBRIDGE_P2P_AVAILABILITY_URL";
const CA_PATH_ENV: &str = "DUBBRIDGE_P2P_AVAILABILITY_CA_PEM";
const IDENTITY_PATH_ENV: &str = "DUBBRIDGE_P2P_AVAILABILITY_IDENTITY_PEM";
const CIPHERTEXT_ROOT_ENV: &str = "DUBBRIDGE_P2P_CIPHERTEXT_ROOT";

pub struct P2pPublicationRuntime {
    dispatcher: P2pPublicationDispatcher,
    idle_interval: StdDuration,
}

impl P2pPublicationRuntime {
    pub async fn from_env(pool: PgPool) -> anyhow::Result<Option<Self>> {
        let Some(base_url) = optional_nonempty_env(URL_ENV)? else {
            reject_partial_configuration()?;
            return Ok(None);
        };

        let ca_path = required_nonempty_env(CA_PATH_ENV)?;
        let identity_path = required_nonempty_env(IDENTITY_PATH_ENV)?;
        let ciphertext_root = PathBuf::from(required_nonempty_env(CIPHERTEXT_ROOT_ENV)?);
        let http_timeout = positive_std_duration("DUBBRIDGE_P2P_HTTP_TIMEOUT_SECS", 10)?;
        let lease_duration = positive_time_duration("DUBBRIDGE_P2P_LEASE_SECS", 30)?;
        let retry_delay = nonnegative_time_duration("DUBBRIDGE_P2P_RETRY_SECS", 5)?;
        let idle_interval = positive_std_millis("DUBBRIDGE_P2P_DISPATCH_INTERVAL_MS", 1_000)?;
        let max_attempts = positive_u32("DUBBRIDGE_P2P_MAX_ATTEMPTS", 5)?;

        let ca_pem = tokio::fs::read(&ca_path)
            .await
            .with_context(|| format!("failed to read {CA_PATH_ENV}"))?;
        let identity_pem = tokio::fs::read(&identity_path)
            .await
            .with_context(|| format!("failed to read {IDENTITY_PATH_ENV}"))?;
        let client = AvailabilityPublicationClient::from_mtls_pem(
            &base_url,
            &ca_pem,
            &identity_pem,
            http_timeout,
        )
        .context("invalid P2P Availability Node mTLS configuration")?;
        let dispatcher = P2pPublicationDispatcher::new(
            pool,
            Arc::new(client),
            ciphertext_root,
            lease_duration,
            retry_delay,
            max_attempts,
        )
        .context("invalid P2P dispatcher configuration")?;

        Ok(Some(Self {
            dispatcher,
            idle_interval,
        }))
    }

    pub async fn run(self) {
        loop {
            self.run_iteration().await;
            tokio::task::yield_now().await;
        }
    }

    async fn run_iteration(&self) {
        match self.dispatcher.dispatch_once().await {
            Ok(DispatchTick::Idle) => tokio::time::sleep(self.idle_interval).await,
            Ok(DispatchTick::Ready(publication_id)) => {
                tracing::info!(%publication_id, "P2P publication reached durable Ready");
            }
            Ok(DispatchTick::Retrying(publication_id)) => {
                tracing::warn!(%publication_id, "P2P publication scheduled for reconciliation");
            }
            Ok(DispatchTick::Failed(publication_id)) => {
                tracing::error!(%publication_id, "P2P publication entered terminal failure");
            }
            Err(error) => {
                tracing::error!(error = %error, "P2P publication reconciler iteration failed");
                tokio::time::sleep(self.idle_interval).await;
            }
        }
    }
}

fn reject_partial_configuration() -> anyhow::Result<()> {
    for name in [CA_PATH_ENV, IDENTITY_PATH_ENV, CIPHERTEXT_ROOT_ENV] {
        if env::var_os(name).is_some() {
            anyhow::bail!("{URL_ENV} is required when {name} is configured");
        }
    }
    Ok(())
}

fn optional_nonempty_env(name: &str) -> anyhow::Result<Option<String>> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => anyhow::bail!("{name} must not be empty"),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => anyhow::bail!("{name} must be valid UTF-8"),
    }
}

fn required_nonempty_env(name: &str) -> anyhow::Result<String> {
    optional_nonempty_env(name)?.ok_or_else(|| anyhow::anyhow!("{name} is required"))
}

fn parse_u64(name: &str, default: u64) -> anyhow::Result<u64> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .with_context(|| format!("{name} must be an unsigned integer")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => anyhow::bail!("{name} must be valid UTF-8"),
    }
}

fn positive_std_duration(name: &str, default: u64) -> anyhow::Result<StdDuration> {
    let value = parse_u64(name, default)?;
    if value == 0 {
        anyhow::bail!("{name} must be greater than zero");
    }
    Ok(StdDuration::from_secs(value))
}

fn positive_std_millis(name: &str, default: u64) -> anyhow::Result<StdDuration> {
    let value = parse_u64(name, default)?;
    if value == 0 {
        anyhow::bail!("{name} must be greater than zero");
    }
    Ok(StdDuration::from_millis(value))
}

fn positive_time_duration(name: &str, default: u64) -> anyhow::Result<Duration> {
    let value = parse_u64(name, default)?;
    if value == 0 || value > i64::MAX as u64 {
        anyhow::bail!("{name} must be a positive bounded duration");
    }
    Ok(Duration::seconds(value as i64))
}

fn nonnegative_time_duration(name: &str, default: u64) -> anyhow::Result<Duration> {
    let value = parse_u64(name, default)?;
    if value > i64::MAX as u64 {
        anyhow::bail!("{name} exceeds the supported duration range");
    }
    Ok(Duration::seconds(value as i64))
}

fn positive_u32(name: &str, default: u32) -> anyhow::Result<u32> {
    match env::var(name) {
        Ok(value) => {
            let parsed = value
                .parse::<u32>()
                .with_context(|| format!("{name} must be a positive integer"))?;
            if parsed == 0 {
                anyhow::bail!("{name} must be greater than zero");
            }
            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => anyhow::bail!("{name} must be valid UTF-8"),
    }
}
