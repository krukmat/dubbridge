// Public auth surface retained after legacy OAuth/session retirement.
pub mod relay;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::state::GatewayState;
use relay::{login_relay_handler, register_relay_handler};

/// Builds the public /auth sub-router for direct credential relay.
/// State is NOT fixed here — it is inherited from the parent router via nest().
pub fn auth_router() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/login", post(login_relay_handler))
        .route("/register", post(register_relay_handler))
}
