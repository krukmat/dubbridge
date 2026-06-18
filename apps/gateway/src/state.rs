pub struct GatewayState {
    pub http_client: reqwest::Client,
    pub config: dubbridge_config::AppConfig,
    pub gateway: dubbridge_config::GatewaySettings,
}

impl GatewayState {
    pub fn new(
        http_client: reqwest::Client,
        config: dubbridge_config::AppConfig,
        gateway: dubbridge_config::GatewaySettings,
    ) -> Self {
        Self {
            http_client,
            config,
            gateway,
        }
    }
}
