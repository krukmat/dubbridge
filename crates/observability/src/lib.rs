use dubbridge_config::ObsSettings;

pub fn init_tracing(obs: &ObsSettings) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(obs.filter.clone())
        .try_init();
}
