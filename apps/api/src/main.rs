use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use dubbridge_api::{
    build_app,
    cleanup::{cleanup_expired_ingestions, run_ingest_reconciliation},
    state::{AppState, SharedAuthService},
};
use dubbridge_auth::{
    AuthService, Hs256Issuer, Hs256TokenVerifier, PgAccountStore, SharedTokenVerifier,
};
use sqlx::PgPool;

// T1-T2: cleanup interval for expired pending ingestions.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    let verifier = build_verifier(&config)?;
    let pool = dubbridge_db::create_pool(&config.database_url)
        .await
        .context("failed to create database pool")?;
    let auth_service = build_auth_service(&config, pool.clone())?;
    let storage_config = dubbridge_storage::StorageConfig::from(&config.storage);
    let storage = dubbridge_storage::build_adapter(&storage_config)
        .context("failed to initialize configured storage backend")?;
    let app_state = Arc::new(AppState::with_auth_service(
        pool,
        storage,
        verifier.clone(),
        config.clone(),
        auth_service,
    ));
    let api_port = app_state.config.api_port;
    let resolved_env = app_state.config.env.clone();
    let storage_backend = app_state.config.storage.backend.clone();
    let log_format = app_state.config.observability.log_format.clone();
    let storage_bucket = app_state.config.storage.bucket.clone();
    let auth_verifier = app_state.verifier.clone();

    // T1-T2: spawn background task that periodically removes expired pending sessions
    // and their stored blobs. The task is detached; it runs until the process exits.
    let cleanup_pool = app_state.pool.clone();
    let cleanup_storage = dubbridge_storage::build_adapter(&storage_config)
        .context("failed to initialize cleanup storage backend")?;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            cleanup_expired_ingestions(&cleanup_pool, cleanup_storage.as_ref()).await;
            if let Err(error) =
                run_ingest_reconciliation(&cleanup_pool, cleanup_storage.as_ref()).await
            {
                tracing::warn!(
                    error = %error,
                    "failed to run ingest object reconciliation"
                );
            }
        }
    });

    let app = build_app(app_state, auth_verifier);

    let addr = SocketAddr::from(([0, 0, 0, 0], api_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        env = ?resolved_env,
        port = api_port,
        storage_backend = ?storage_backend,
        log_format = ?log_format,
        storage_bucket = %storage_bucket,
        "starting api"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_verifier(config: &dubbridge_config::AppConfig) -> anyhow::Result<SharedTokenVerifier> {
    let secret = resolve_jwt_secret(config)?;
    let verifier =
        Hs256TokenVerifier::new(&secret).context("failed to initialize HS256 JWT verifier")?;

    Ok(Arc::new(verifier))
}

fn build_auth_service(
    config: &dubbridge_config::AppConfig,
    pool: PgPool,
) -> anyhow::Result<SharedAuthService> {
    let auth = auth_settings(config)?;
    let secret = resolve_jwt_secret(config)?;
    let expiry = auth
        .jwt_expiry_hours
        .checked_mul(60 * 60)
        .ok_or_else(|| anyhow!("auth.jwt_expiry_hours overflowed while building issuer"))?;
    if expiry == 0 {
        return Err(anyhow!(
            "auth.jwt_expiry_hours must be greater than zero to start the API"
        ));
    }

    let issuer = Hs256Issuer::new(&secret, Duration::from_secs(expiry))
        .context("failed to initialize HS256 JWT issuer")?;
    let account_store = PgAccountStore::new(pool);

    Ok(Arc::new(AuthService::new(account_store, issuer)))
}

fn auth_settings(
    config: &dubbridge_config::AppConfig,
) -> anyhow::Result<&dubbridge_config::AuthSettings> {
    config
        .auth
        .as_ref()
        .ok_or_else(|| anyhow!("auth settings are required to start the API"))
}

fn resolve_jwt_secret(config: &dubbridge_config::AppConfig) -> anyhow::Result<String> {
    let auth = auth_settings(config)?;
    match &auth.jwt_secret {
        Some(secret) => Ok(secret.clone()),
        None if !config.env.is_production_like() => {
            Ok("local-dev-jwt-secret-placeholder".to_string())
        }
        None => Err(anyhow!(
            "auth.jwt_secret is required to start the API in production-like environments"
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dubbridge_auth::{Hs256Issuer, TokenVerificationError};
    use uuid::Uuid;

    use super::*;

    fn sample_pool() -> PgPool {
        PgPool::connect_lazy("postgres://user:pass@db.example.com:5432/dubbridge")
            .expect("lazy pool")
    }

    fn sample_config(
        env: dubbridge_config::AppEnv,
        jwt_secret: Option<&str>,
    ) -> dubbridge_config::AppConfig {
        dubbridge_config::AppConfig {
            env,
            api_port: 8080,
            database_url: "postgres://user:pass@db.example.com:5432/dubbridge".to_string(),
            redis_url: "redis://redis.example.com:6379".to_string(),
            worker_concurrency: 4,
            storage: dubbridge_config::StorageSettings {
                backend: dubbridge_config::StorageBackend::S3,
                base_path: String::new(),
                bucket: "dubbridge-test".to_string(),
                endpoint_url: None,
            },
            observability: dubbridge_config::ObsSettings {
                log_format: dubbridge_config::LogFormat::Json,
                filter: "info".to_string(),
            },
            auth: Some(dubbridge_config::AuthSettings {
                issuer: "https://issuer.example".to_string(),
                audience: "dubbridge-api".to_string(),
                rsa_public_key_path: "/tmp/public.pem".to_string(),
                jwt_secret: jwt_secret.map(str::to_owned),
                jwt_expiry_hours: 24,
                clock_skew_leeway_seconds: 30,
            }),
            gateway: None,
        }
    }

    #[test]
    fn build_verifier_uses_configured_jwt_secret() {
        let config = sample_config(
            dubbridge_config::AppEnv::Production,
            Some("configured-secret"),
        );
        let verifier = build_verifier(&config).expect("verifier");
        let issuer =
            Hs256Issuer::new("configured-secret", Duration::from_secs(3600)).expect("issuer");
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &["assets:read".to_string()])
            .expect("token");

        let principal = verifier.verify_access_token(&token).expect("principal");

        assert!(principal.scopes().contains("assets:read"));
    }

    #[test]
    fn build_verifier_uses_local_placeholder_when_jwt_secret_is_missing() {
        let config = sample_config(dubbridge_config::AppEnv::Local, None);
        let verifier = build_verifier(&config).expect("verifier");
        let issuer = Hs256Issuer::new(
            "local-dev-jwt-secret-placeholder",
            Duration::from_secs(3600),
        )
        .expect("issuer");
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &[])
            .expect("token");

        assert!(verifier.verify_access_token(&token).is_ok());
    }

    #[test]
    fn build_verifier_rejects_local_tokens_signed_with_other_secret() {
        let config = sample_config(dubbridge_config::AppEnv::Local, None);
        let verifier = build_verifier(&config).expect("verifier");
        let issuer = Hs256Issuer::new("another-secret", Duration::from_secs(3600)).expect("issuer");
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &[])
            .expect("token");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid signature");

        assert_eq!(error, TokenVerificationError::InvalidSignature);
    }

    #[test]
    fn build_verifier_fails_closed_without_jwt_secret_in_production_like_env() {
        let config = sample_config(dubbridge_config::AppEnv::Production, None);

        let error = match build_verifier(&config) {
            Ok(_) => panic!("missing production secret should fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("auth.jwt_secret is required"));
    }

    #[tokio::test]
    async fn build_auth_service_uses_local_placeholder_when_jwt_secret_is_missing() {
        let config = sample_config(dubbridge_config::AppEnv::Local, None);

        let service = build_auth_service(&config, sample_pool()).expect("auth service");

        assert!(Arc::strong_count(&service) >= 1);
    }

    #[tokio::test]
    async fn build_auth_service_fails_closed_without_jwt_secret_in_production_like_env() {
        let config = sample_config(dubbridge_config::AppEnv::Production, None);

        let error = match build_auth_service(&config, sample_pool()) {
            Ok(_) => panic!("missing production secret should fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("auth.jwt_secret is required"));
    }

    #[tokio::test]
    async fn build_auth_service_rejects_zero_hour_expiry() {
        let mut config = sample_config(
            dubbridge_config::AppEnv::Production,
            Some("configured-secret"),
        );
        config
            .auth
            .as_mut()
            .expect("auth settings")
            .jwt_expiry_hours = 0;

        let error = match build_auth_service(&config, sample_pool()) {
            Ok(_) => panic!("zero expiry should fail"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("jwt_expiry_hours must be greater than zero")
        );
    }
}
