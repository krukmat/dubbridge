use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub storage_bucket: String,
    pub worker_concurrency: usize,
    pub auth: Option<AuthSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub issuer: String,
    pub audience: String,
    pub rsa_public_key_path: String,
    pub clock_skew_leeway_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            api_port: std::env::var("DUBBRIDGE_API_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DUBBRIDGE_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://dubbridge:dubbridge@localhost:5432/dubbridge".to_string()
            }),
            redis_url: std::env::var("DUBBRIDGE_REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            storage_bucket: std::env::var("DUBBRIDGE_STORAGE_BUCKET")
                .unwrap_or_else(|_| "dubbridge-local".to_string()),
            worker_concurrency: std::env::var("DUBBRIDGE_WORKER_CONCURRENCY")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(4),
            auth: AuthSettings::from_env(),
        }
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

    // T1-T3: AppConfig::from_env defaults — protects against silent production misconfiguration
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
