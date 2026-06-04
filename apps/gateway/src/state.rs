// P1-T1: gateway runtime state — shared across all request handlers
// P1-T3: extended with Arc<dyn SessionStore> (session contract, ADR-024)
// P1-T4: extended with Arc<PendingAuthStore> (single-use OAuth state pairs)
use std::sync::Arc;

use crate::auth::pending::PendingAuthStore;
use crate::session::SessionStore;

pub struct GatewayState {
    pub http_client: reqwest::Client,
    pub config: dubbridge_config::AppConfig,
    pub gateway: dubbridge_config::GatewaySettings,
    pub session_store: Arc<dyn SessionStore>,
    pub pending_store: Arc<PendingAuthStore>,
}

impl GatewayState {
    pub fn new(
        http_client: reqwest::Client,
        config: dubbridge_config::AppConfig,
        gateway: dubbridge_config::GatewaySettings,
        session_store: Arc<dyn SessionStore>,
        pending_store: Arc<PendingAuthStore>,
    ) -> Self {
        Self {
            http_client,
            config,
            gateway,
            session_store,
            pending_store,
        }
    }
}
