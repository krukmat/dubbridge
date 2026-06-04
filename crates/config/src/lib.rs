use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// Task 1: fail-closed environment discriminator (ADR-026, Decision 1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    Local,
    Staging,
    Production,
}

impl AppEnv {
    /// Returns true for any environment that must meet production-grade constraints.
    pub fn is_production_like(&self) -> bool {
        matches!(self, AppEnv::Staging | AppEnv::Production)
    }

    /// Reads DUBBRIDGE_ENV from the process environment. No compiled default —
    /// a missing or unrecognised value is a hard error (ADR-026, Decision 1).
    pub fn from_process() -> Result<Self, ConfigError> {
        let raw = std::env::var("DUBBRIDGE_ENV").map_err(|_| ConfigError::MissingEnv)?;
        match raw.as_str() {
            "local" => Ok(AppEnv::Local),
            "staging" => Ok(AppEnv::Staging),
            "production" => Ok(AppEnv::Production),
            _ => Err(ConfigError::UnknownEnv(raw)),
        }
    }

    pub fn profile_name(&self) -> &'static str {
        match self {
            AppEnv::Local => "local",
            AppEnv::Staging => "staging",
            AppEnv::Production => "production",
        }
    }
}

// Task 1: typed error hierarchy for all configuration failure modes (ADR-026)
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("DUBBRIDGE_ENV is not set; set it to one of: local, staging, production")]
    MissingEnv,

    #[error("DUBBRIDGE_ENV has an unrecognised value '{0}'; expected: local, staging, production")]
    UnknownEnv(String),

    // T2-C: layered profile load failures.
    #[error("configuration load error: {0}")]
    Load(String),

    /// Reserved for Task 3: fail-closed validate() rejections.
    #[error("configuration validation error: {0}")]
    Validation(String),
}

// T2-B: storage backend selector — seam for S2 S3 adapter (ADR-006, ADR-026 Decision 5)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackend {
    LocalFs,
    S3,
}

// T2-B: log format selector — behavior wired in Phase 2 (ADR-018, ADR-026 Decision 5)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Pretty,
    Json,
}

// T2-B: grouped storage settings — absorbs StorageConfig::from_env (T2-D)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub backend: StorageBackend,
    pub base_path: String,
    pub bucket: String,
    pub endpoint_url: Option<String>,
}

// T2-B: grouped observability settings — consolidates RUST_LOG reader (ADR-026 §3, Phase 0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSettings {
    pub log_format: LogFormat,
    pub filter: String,
}

// T2-B: expanded typed schema; from_env() kept for Task 4 removal (ADR-026, Decision 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub env: AppEnv,
    pub api_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub worker_concurrency: usize,
    pub storage: StorageSettings,
    pub observability: ObsSettings,
    pub auth: Option<AuthSettings>,
    pub gateway: Option<GatewaySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub issuer: String,
    pub audience: String,
    pub rsa_public_key_path: String,
    pub clock_skew_leeway_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySettings {
    pub port: u16,
    pub upstream_api_base_url: String,
    pub oauth: GatewayOAuthSettings,
    pub session: GatewaySessionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayOAuthSettings {
    pub authorization_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySessionSettings {
    pub cookie_name: String,
    pub absolute_ttl_seconds: u64,
    pub idle_ttl_seconds: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let env = AppEnv::from_process()?;
        let config_dir = Self::resolve_config_dir();
        let default_path = config_dir.join("default.toml");
        let env_path = config_dir.join(format!("{}.toml", env.profile_name()));

        let config: Self = Figment::new()
            .merge(Toml::file(&default_path))
            .merge(Toml::file(&env_path))
            .merge(Env::prefixed("DUBBRIDGE_").split("__"))
            .extract()
            .map_err(|e| ConfigError::Load(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(gateway) = &self.gateway {
            gateway.validate(self.env.is_production_like())?;
        }

        if !self.env.is_production_like() {
            return Ok(());
        }

        if Self::is_local_address_url(&self.database_url) {
            return Err(ConfigError::Validation(
                "database_url must not target localhost or 127.0.0.1 in production-like environments"
                    .to_string(),
            ));
        }

        if Self::is_local_address_url(&self.redis_url) {
            return Err(ConfigError::Validation(
                "redis_url must not target localhost or 127.0.0.1 in production-like environments"
                    .to_string(),
            ));
        }

        if matches!(self.storage.backend, StorageBackend::LocalFs) {
            return Err(ConfigError::Validation(
                "storage.backend must not be local_fs in production-like environments".to_string(),
            ));
        }

        if self.auth.is_none() {
            return Err(ConfigError::Validation(
                "auth settings are required in production-like environments".to_string(),
            ));
        }

        if matches!(self.observability.log_format, LogFormat::Pretty) {
            return Err(ConfigError::Validation(
                "observability.log_format must not be pretty in production-like environments"
                    .to_string(),
            ));
        }

        Ok(())
    }

    pub fn gateway_settings(&self) -> Result<&GatewaySettings, ConfigError> {
        self.gateway.as_ref().ok_or_else(|| {
            ConfigError::Validation(
                "gateway settings are required to start the gateway".to_string(),
            )
        })
    }

    /// Legacy reader kept until Task 4 replaces call sites with load().
    /// Do not add new callers — use load() instead.
    pub fn from_env() -> Self {
        Self {
            env: AppEnv::Local,
            api_port: std::env::var("DUBBRIDGE_API_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DUBBRIDGE_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://dubbridge:dubbridge@localhost:5432/dubbridge".to_string()
            }),
            redis_url: std::env::var("DUBBRIDGE_REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            worker_concurrency: std::env::var("DUBBRIDGE_WORKER_CONCURRENCY")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(4),
            storage: StorageSettings {
                backend: StorageBackend::LocalFs,
                base_path: std::env::var("DUBBRIDGE_STORAGE_BASE_PATH")
                    .unwrap_or_else(|_| "/tmp/dubbridge-storage".to_string()),
                bucket: std::env::var("DUBBRIDGE_STORAGE_BUCKET")
                    .unwrap_or_else(|_| "dubbridge-local".to_string()),
                endpoint_url: std::env::var("DUBBRIDGE_STORAGE_ENDPOINT").ok(),
            },
            observability: ObsSettings {
                log_format: LogFormat::Pretty,
                filter: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            },
            auth: AuthSettings::from_env(),
            gateway: None,
        }
    }

    fn resolve_config_dir() -> PathBuf {
        std::env::var("DUBBRIDGE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                workspace_root.pop();
                workspace_root.pop();
                workspace_root.join("config")
            })
    }

    fn is_local_address_url(url: &str) -> bool {
        let normalized = url.to_ascii_lowercase();
        normalized.contains("localhost") || normalized.contains("127.0.0.1")
    }
}

impl GatewaySettings {
    fn validate(&self, production_like: bool) -> Result<(), ConfigError> {
        if self.upstream_api_base_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.upstream_api_base_url is required".to_string(),
            ));
        }

        if self.oauth.authorization_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.oauth.authorization_url is required".to_string(),
            ));
        }

        if self.oauth.token_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.oauth.token_url is required".to_string(),
            ));
        }

        if self.oauth.client_id.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.oauth.client_id is required".to_string(),
            ));
        }

        if self.oauth.redirect_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.oauth.redirect_url is required".to_string(),
            ));
        }

        if self.session.cookie_name.trim().is_empty() {
            return Err(ConfigError::Validation(
                "gateway.session.cookie_name is required".to_string(),
            ));
        }

        if self.session.absolute_ttl_seconds == 0 {
            return Err(ConfigError::Validation(
                "gateway.session.absolute_ttl_seconds must be greater than zero".to_string(),
            ));
        }

        if self.session.idle_ttl_seconds == 0 {
            return Err(ConfigError::Validation(
                "gateway.session.idle_ttl_seconds must be greater than zero".to_string(),
            ));
        }

        if production_like && self.oauth.client_secret.is_none() {
            return Err(ConfigError::Validation(
                "gateway.oauth.client_secret is required in production-like environments"
                    .to_string(),
            ));
        }

        if production_like && AppConfig::is_local_address_url(&self.upstream_api_base_url) {
            return Err(ConfigError::Validation(
                "gateway.upstream_api_base_url must not target localhost or 127.0.0.1 in production-like environments".to_string(),
            ));
        }

        if production_like && AppConfig::is_local_address_url(&self.oauth.authorization_url) {
            return Err(ConfigError::Validation(
                "gateway.oauth.authorization_url must not target localhost or 127.0.0.1 in production-like environments".to_string(),
            ));
        }

        if production_like && AppConfig::is_local_address_url(&self.oauth.token_url) {
            return Err(ConfigError::Validation(
                "gateway.oauth.token_url must not target localhost or 127.0.0.1 in production-like environments".to_string(),
            ));
        }

        Ok(())
    }
}

impl AuthSettings {
    pub fn from_env() -> Option<Self> {
        let issuer = std::env::var("DUBBRIDGE_AUTH_ISSUER").ok()?;
        let audience = std::env::var("DUBBRIDGE_AUTH_AUDIENCE").ok()?;
        let rsa_public_key_path = std::env::var("DUBBRIDGE_AUTH_RSA_PUBLIC_KEY_PATH").ok()?;

        Some(Self {
            issuer,
            audience,
            rsa_public_key_path,
            clock_skew_leeway_seconds: std::env::var("DUBBRIDGE_AUTH_CLOCK_SKEW_LEEWAY_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(30),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Task 1: AppEnv + ConfigError + fail-closed DUBBRIDGE_ENV resolution ---

    #[test]
    fn app_env_missing_dubbridge_env_returns_missing_env_error() {
        temp_env::with_var_unset("DUBBRIDGE_ENV", || {
            let result = AppEnv::from_process();
            assert!(
                matches!(result, Err(ConfigError::MissingEnv)),
                "expected MissingEnv, got: {result:?}"
            );
        });
    }

    #[test]
    fn app_env_unknown_value_prod_returns_unknown_env_error() {
        temp_env::with_var("DUBBRIDGE_ENV", Some("prod"), || {
            let result = AppEnv::from_process();
            assert!(
                matches!(result, Err(ConfigError::UnknownEnv(ref v)) if v == "prod"),
                "expected UnknownEnv(\"prod\"), got: {result:?}"
            );
        });
    }

    #[test]
    fn app_env_empty_string_returns_unknown_env_error() {
        temp_env::with_var("DUBBRIDGE_ENV", Some(""), || {
            let result = AppEnv::from_process();
            assert!(
                matches!(result, Err(ConfigError::UnknownEnv(ref v)) if v.is_empty()),
                "expected UnknownEnv(\"\"), got: {result:?}"
            );
        });
    }

    #[test]
    fn app_env_local_parses_to_local_variant() {
        temp_env::with_var("DUBBRIDGE_ENV", Some("local"), || {
            let result = AppEnv::from_process();
            assert_eq!(result.unwrap(), AppEnv::Local);
        });
    }

    #[test]
    fn app_env_staging_parses_to_staging_variant() {
        temp_env::with_var("DUBBRIDGE_ENV", Some("staging"), || {
            let result = AppEnv::from_process();
            assert_eq!(result.unwrap(), AppEnv::Staging);
        });
    }

    #[test]
    fn app_env_production_parses_to_production_variant() {
        temp_env::with_var("DUBBRIDGE_ENV", Some("production"), || {
            let result = AppEnv::from_process();
            assert_eq!(result.unwrap(), AppEnv::Production);
        });
    }

    #[test]
    fn app_env_is_production_like_false_for_local() {
        assert!(!AppEnv::Local.is_production_like());
    }

    #[test]
    fn app_env_is_production_like_true_for_staging() {
        assert!(AppEnv::Staging.is_production_like());
    }

    #[test]
    fn app_env_is_production_like_true_for_production() {
        assert!(AppEnv::Production.is_production_like());
    }

    // --- T2-B: schema round-trip — each profile deserializes into AppConfig ---

    fn fixtures_dir() -> String {
        // CARGO_MANIFEST_DIR points to crates/config/ at compile time; config/ is at workspace root
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let workspace_root = std::path::Path::new(&manifest)
            .parent()
            .and_then(|p| p.parent())
            .expect("could not derive workspace root from CARGO_MANIFEST_DIR")
            .to_str()
            .expect("workspace root is not valid UTF-8")
            .to_string();
        format!("{workspace_root}/config")
    }

    fn load_profile(env_name: &str) -> AppConfig {
        use figment::{
            Figment,
            providers::{Format, Toml},
        };
        let dir = fixtures_dir();
        let default_path = format!("{dir}/default.toml");
        let env_path = format!("{dir}/{env_name}.toml");
        Figment::new()
            .merge(Toml::file(&default_path))
            .merge(Toml::file(&env_path))
            .extract()
            .unwrap_or_else(|e| panic!("failed to deserialize {env_name}.toml: {e}"))
    }

    #[test]
    fn schema_local_profile_deserializes() {
        let cfg: AppConfig = load_profile("local");
        assert_eq!(cfg.env, AppEnv::Local);
        assert_eq!(cfg.api_port, 8080);
        assert!(cfg.database_url.contains("localhost"));
        assert!(cfg.redis_url.contains("127.0.0.1"));
        assert_eq!(cfg.storage.backend, StorageBackend::LocalFs);
        assert_eq!(cfg.storage.bucket, "dubbridge-local");
        assert_eq!(cfg.observability.log_format, LogFormat::Pretty);
        let gateway = cfg
            .gateway
            .expect("local gateway settings should deserialize");
        assert_eq!(gateway.port, 8081);
        assert_eq!(gateway.oauth.client_id, "dubbridge-web-local");
    }

    #[test]
    fn schema_staging_profile_deserializes() {
        // Inject secrets that staging.toml intentionally omits (they arrive via env at deploy time).
        // DUBBRIDGE__ (double underscore) is figment's nesting separator for env vars.
        temp_env::with_vars(
            [
                (
                    "DUBBRIDGE_DATABASE_URL",
                    Some("postgres://user:pass@staging-db/dubbridge"),
                ),
                ("DUBBRIDGE_REDIS_URL", Some("redis://staging-redis:6379")),
            ],
            || {
                use figment::providers::{Env, Format, Toml};
                let dir = fixtures_dir();
                let cfg: AppConfig = figment::Figment::new()
                    .merge(Toml::file(format!("{dir}/default.toml")))
                    .merge(Toml::file(format!("{dir}/staging.toml")))
                    .merge(Env::prefixed("DUBBRIDGE_").split("__"))
                    .extract()
                    .expect("staging profile should deserialize with injected secrets");
                assert_eq!(cfg.env, AppEnv::Staging);
                assert_eq!(cfg.storage.backend, StorageBackend::S3);
                assert_eq!(cfg.observability.log_format, LogFormat::Json);
                assert_eq!(
                    cfg.gateway
                        .expect("staging gateway settings should deserialize")
                        .upstream_api_base_url,
                    "https://api.staging.dubbridge.example"
                );
            },
        );
    }

    #[test]
    fn schema_production_profile_deserializes() {
        // Inject secrets that production.toml intentionally omits (they arrive via env at deploy time).
        temp_env::with_vars(
            [
                (
                    "DUBBRIDGE_DATABASE_URL",
                    Some("postgres://user:pass@prod-db/dubbridge"),
                ),
                ("DUBBRIDGE_REDIS_URL", Some("redis://prod-redis:6379")),
            ],
            || {
                use figment::providers::{Env, Format, Toml};
                let dir = fixtures_dir();
                let cfg: AppConfig = figment::Figment::new()
                    .merge(Toml::file(format!("{dir}/default.toml")))
                    .merge(Toml::file(format!("{dir}/production.toml")))
                    .merge(Env::prefixed("DUBBRIDGE_").split("__"))
                    .extract()
                    .expect("production profile should deserialize with injected secrets");
                assert_eq!(cfg.env, AppEnv::Production);
                assert_eq!(cfg.storage.backend, StorageBackend::S3);
                assert_eq!(cfg.observability.log_format, LogFormat::Json);
                assert_eq!(
                    cfg.gateway
                        .expect("production gateway settings should deserialize")
                        .oauth
                        .client_id,
                    "dubbridge-web"
                );
            },
        );
    }

    // --- T2-C: AppConfig::load() layered loader ---

    #[test]
    fn app_config_load_missing_dubbridge_env_returns_missing_env() {
        temp_env::with_var_unset("DUBBRIDGE_ENV", || {
            let result = AppConfig::load();
            assert!(
                matches!(result, Err(ConfigError::MissingEnv)),
                "expected MissingEnv, got: {result:?}"
            );
        });
    }

    #[test]
    fn app_config_load_unknown_dubbridge_env_returns_unknown_env() {
        temp_env::with_var("DUBBRIDGE_ENV", Some("qa"), || {
            let result = AppConfig::load();
            assert!(
                matches!(result, Err(ConfigError::UnknownEnv(ref v)) if v == "qa"),
                "expected UnknownEnv(\"qa\"), got: {result:?}"
            );
        });
    }

    #[test]
    fn app_config_load_local_profile_reads_local_toml_values() {
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("local")),
                ("DUBBRIDGE_CONFIG_DIR", Some(fixtures_dir().as_str())),
                (
                    "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET",
                    Some("local-gateway-secret"),
                ),
            ],
            || {
                let cfg = AppConfig::load().expect("local profile should load");
                assert_eq!(cfg.api_port, 8080);
                assert!(cfg.database_url.contains("localhost"));
                assert_eq!(cfg.storage.backend, StorageBackend::LocalFs);
                let gateway = cfg.gateway_settings().expect("gateway settings");
                assert_eq!(gateway.port, 8081);
                assert_eq!(
                    gateway.oauth.client_secret.as_deref(),
                    Some("local-gateway-secret")
                );
            },
        );
    }

    #[test]
    fn app_config_load_staging_profile_reads_staging_toml_values() {
        let config_dir = fixtures_dir();
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("staging")),
                ("DUBBRIDGE_CONFIG_DIR", Some(config_dir.as_str())),
                (
                    "DUBBRIDGE_DATABASE_URL",
                    Some("postgres://user:pass@staging-db/dubbridge"),
                ),
                ("DUBBRIDGE_REDIS_URL", Some("redis://staging-redis:6379")),
                ("DUBBRIDGE_AUTH__ISSUER", Some("https://issuer.example")),
                ("DUBBRIDGE_AUTH__AUDIENCE", Some("dubbridge-api")),
                (
                    "DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH",
                    Some("/tmp/public.pem"),
                ),
                ("DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS", Some("30")),
                (
                    "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET",
                    Some("staging-gateway-secret"),
                ),
            ],
            || {
                let cfg = AppConfig::load().expect("staging profile should load");
                assert_eq!(cfg.storage.backend, StorageBackend::S3);
                assert_eq!(cfg.observability.log_format, LogFormat::Json);
                assert_eq!(
                    cfg.gateway_settings()
                        .expect("gateway settings")
                        .oauth
                        .client_secret
                        .as_deref(),
                    Some("staging-gateway-secret")
                );
            },
        );
    }

    #[test]
    fn app_config_load_env_override_wins_over_toml() {
        let config_dir = fixtures_dir();
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("local")),
                ("DUBBRIDGE_CONFIG_DIR", Some(config_dir.as_str())),
                ("DUBBRIDGE_API_PORT", Some("9090")),
            ],
            || {
                let cfg = AppConfig::load().expect("env override should win over TOML");
                assert_eq!(cfg.api_port, 9090);
            },
        );
    }

    #[test]
    fn app_config_load_bad_config_dir_returns_load_error() {
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("local")),
                (
                    "DUBBRIDGE_CONFIG_DIR",
                    Some("/definitely/missing/dubbridge-config-dir"),
                ),
            ],
            || {
                let result = AppConfig::load();
                assert!(
                    matches!(result, Err(ConfigError::Load(_))),
                    "expected Load error, got: {result:?}"
                );
            },
        );
    }

    // --- Task 3: AppConfig::validate() fail-closed checks ---

    fn sample_auth() -> AuthSettings {
        AuthSettings {
            issuer: "https://issuer.example".to_string(),
            audience: "dubbridge-api".to_string(),
            rsa_public_key_path: "/tmp/public.pem".to_string(),
            clock_skew_leeway_seconds: 30,
        }
    }

    fn production_like_config() -> AppConfig {
        AppConfig {
            env: AppEnv::Production,
            api_port: 8080,
            database_url: "postgres://user:pass@db.example.com:5432/dubbridge".to_string(),
            redis_url: "redis://redis.example.com:6379".to_string(),
            worker_concurrency: 4,
            storage: StorageSettings {
                backend: StorageBackend::S3,
                base_path: String::new(),
                bucket: "dubbridge-production".to_string(),
                endpoint_url: None,
            },
            observability: ObsSettings {
                log_format: LogFormat::Json,
                filter: "info".to_string(),
            },
            auth: Some(sample_auth()),
            gateway: Some(sample_gateway()),
        }
    }

    fn sample_gateway() -> GatewaySettings {
        GatewaySettings {
            port: 8081,
            upstream_api_base_url: "https://api.example.com".to_string(),
            oauth: GatewayOAuthSettings {
                authorization_url: "https://auth.example.com/oauth/authorize".to_string(),
                token_url: "https://auth.example.com/oauth/token".to_string(),
                client_id: "dubbridge-web".to_string(),
                client_secret: Some("gateway-secret".to_string()),
                redirect_url: "https://gateway.example.com/auth/callback".to_string(),
            },
            session: GatewaySessionSettings {
                cookie_name: "dubbridge_session".to_string(),
                absolute_ttl_seconds: 28_800,
                idle_ttl_seconds: 1_800,
            },
        }
    }

    #[test]
    fn app_config_validate_rejects_localhost_database_url_in_production() {
        let mut cfg = production_like_config();
        cfg.database_url = "postgres://user:pass@localhost:5432/dubbridge".to_string();

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("database_url")),
            "expected Validation(database_url), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_loopback_redis_url_in_production() {
        let mut cfg = production_like_config();
        cfg.redis_url = "redis://127.0.0.1:6379".to_string();

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("redis_url")),
            "expected Validation(redis_url), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_local_fs_storage_in_production() {
        let mut cfg = production_like_config();
        cfg.storage.backend = StorageBackend::LocalFs;

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("storage.backend")),
            "expected Validation(storage.backend), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_missing_auth_in_production() {
        let mut cfg = production_like_config();
        cfg.auth = None;

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("auth")),
            "expected Validation(auth), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_pretty_logs_in_production() {
        let mut cfg = production_like_config();
        cfg.observability.log_format = LogFormat::Pretty;

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("observability.log_format")),
            "expected Validation(observability.log_format), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_allows_local_development_values() {
        let cfg = AppConfig {
            env: AppEnv::Local,
            api_port: 8080,
            database_url: "postgres://dubbridge:dubbridge@localhost:5432/dubbridge".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            worker_concurrency: 4,
            storage: StorageSettings {
                backend: StorageBackend::LocalFs,
                base_path: "/tmp/dubbridge-storage".to_string(),
                bucket: "dubbridge-local".to_string(),
                endpoint_url: None,
            },
            observability: ObsSettings {
                log_format: LogFormat::Pretty,
                filter: "info".to_string(),
            },
            auth: None,
            gateway: Some(sample_gateway()),
        };

        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn app_config_validate_rejects_missing_gateway_upstream_url() {
        let mut cfg = production_like_config();
        cfg.gateway.as_mut().expect("gateway").upstream_api_base_url = String::new();

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("gateway.upstream_api_base_url")),
            "expected Validation(gateway.upstream_api_base_url), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_local_gateway_authorization_url_in_production() {
        let mut cfg = production_like_config();
        cfg.gateway
            .as_mut()
            .expect("gateway")
            .oauth
            .authorization_url = "http://localhost:9000/oauth/authorize".to_string();

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("gateway.oauth.authorization_url")),
            "expected Validation(gateway.oauth.authorization_url), got: {result:?}"
        );
    }

    #[test]
    fn app_config_validate_rejects_missing_gateway_client_secret_in_production() {
        let mut cfg = production_like_config();
        cfg.gateway.as_mut().expect("gateway").oauth.client_secret = None;

        let result = cfg.validate();

        assert!(
            matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("gateway.oauth.client_secret")),
            "expected Validation(gateway.oauth.client_secret), got: {result:?}"
        );
    }

    #[test]
    fn app_config_load_runs_validate_for_production_profile() {
        let config_dir = fixtures_dir();
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("production")),
                ("DUBBRIDGE_CONFIG_DIR", Some(config_dir.as_str())),
                (
                    "DUBBRIDGE_DATABASE_URL",
                    Some("postgres://user:pass@localhost:5432/dubbridge"),
                ),
                (
                    "DUBBRIDGE_REDIS_URL",
                    Some("redis://redis.example.com:6379"),
                ),
                ("DUBBRIDGE_AUTH__ISSUER", Some("https://issuer.example")),
                ("DUBBRIDGE_AUTH__AUDIENCE", Some("dubbridge-api")),
                (
                    "DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH",
                    Some("/tmp/public.pem"),
                ),
                ("DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS", Some("30")),
                (
                    "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET",
                    Some("prod-gateway-secret"),
                ),
            ],
            || {
                let result = AppConfig::load();
                assert!(
                    matches!(result, Err(ConfigError::Validation(ref msg)) if msg.contains("database_url")),
                    "expected Validation(database_url), got: {result:?}"
                );
            },
        );
    }

    #[test]
    fn app_config_validate_production_profile_with_representative_secrets_passes() {
        let config_dir = fixtures_dir();
        temp_env::with_vars(
            [
                ("DUBBRIDGE_ENV", Some("production")),
                ("DUBBRIDGE_CONFIG_DIR", Some(config_dir.as_str())),
                (
                    "DUBBRIDGE_DATABASE_URL",
                    Some("postgres://user:pass@prod-db.example.com:5432/dubbridge"),
                ),
                (
                    "DUBBRIDGE_REDIS_URL",
                    Some("redis://prod-redis.example.com:6379"),
                ),
                ("DUBBRIDGE_AUTH__ISSUER", Some("https://issuer.example")),
                ("DUBBRIDGE_AUTH__AUDIENCE", Some("dubbridge-api")),
                (
                    "DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH",
                    Some("/tmp/public.pem"),
                ),
                ("DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS", Some("30")),
                (
                    "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET",
                    Some("prod-gateway-secret"),
                ),
            ],
            || {
                let cfg = AppConfig::load().expect("production profile should validate");
                assert_eq!(cfg.env, AppEnv::Production);
                assert_eq!(cfg.storage.backend, StorageBackend::S3);
            },
        );
    }

    // --- T1-T3: AppConfig::from_env defaults — kept until Task 4 removes from_env ---

    #[test]
    fn app_config_default_api_port() {
        temp_env::with_var_unset("DUBBRIDGE_API_PORT", || {
            temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
                let cfg = AppConfig::from_env();
                assert_eq!(cfg.api_port, 8080);
            });
        });
    }

    #[test]
    fn app_config_reads_api_port_from_env() {
        temp_env::with_var("DUBBRIDGE_API_PORT", Some("9090"), || {
            temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
                let cfg = AppConfig::from_env();
                assert_eq!(cfg.api_port, 9090);
            });
        });
    }

    #[test]
    fn app_config_invalid_api_port_falls_back_to_default() {
        temp_env::with_var("DUBBRIDGE_API_PORT", Some("not-a-number"), || {
            temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
                let cfg = AppConfig::from_env();
                assert_eq!(cfg.api_port, 8080);
            });
        });
    }

    #[test]
    fn app_config_default_database_url() {
        temp_env::with_var_unset("DUBBRIDGE_DATABASE_URL", || {
            temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
                let cfg = AppConfig::from_env();
                assert!(cfg.database_url.contains("dubbridge"));
            });
        });
    }

    #[test]
    fn app_config_default_worker_concurrency() {
        temp_env::with_var_unset("DUBBRIDGE_WORKER_CONCURRENCY", || {
            temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
                let cfg = AppConfig::from_env();
                assert_eq!(cfg.worker_concurrency, 4);
            });
        });
    }

    #[test]
    fn auth_settings_returns_none_when_issuer_absent() {
        temp_env::with_var_unset("DUBBRIDGE_AUTH_ISSUER", || {
            assert!(AuthSettings::from_env().is_none());
        });
    }

    #[test]
    fn auth_settings_returns_none_when_audience_absent() {
        temp_env::with_vars(
            [
                ("DUBBRIDGE_AUTH_ISSUER", Some("https://issuer")),
                ("DUBBRIDGE_AUTH_AUDIENCE", None),
            ],
            || {
                assert!(AuthSettings::from_env().is_none());
            },
        );
    }

    #[test]
    fn auth_settings_default_clock_skew() {
        temp_env::with_vars(
            [
                ("DUBBRIDGE_AUTH_ISSUER", Some("https://issuer")),
                ("DUBBRIDGE_AUTH_AUDIENCE", Some("api")),
                ("DUBBRIDGE_AUTH_RSA_PUBLIC_KEY_PATH", Some("/key.pem")),
                ("DUBBRIDGE_AUTH_CLOCK_SKEW_LEEWAY_SECONDS", None),
            ],
            || {
                let settings = AuthSettings::from_env().unwrap();
                assert_eq!(settings.clock_skew_leeway_seconds, 30);
            },
        );
    }
}
