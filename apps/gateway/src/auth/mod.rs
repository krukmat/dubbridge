// P1-T2: OAuth 2.0 client module (PKCE token exchange + refresh)
pub mod handoff;
pub mod mobile_session;
pub mod oauth_client;
// P1-T4: pending state/verifier store, login/callback/logout routes
pub mod login;
pub mod logout;
pub mod pending;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::GatewayState;
use login::{callback_handler, login_handler};
use logout::logout_handler;
use mobile_session::mobile_session_handler;

/// Builds the /auth sub-router: login, callback, logout.
/// State is NOT fixed here — it is inherited from the parent router via nest().
pub fn auth_router() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/mobile/session", post(mobile_session_handler))
        .route("/logout", post(logout_handler))
}
