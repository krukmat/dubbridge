use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::{AuthenticatedPrincipal, TokenVerifier};

pub type SharedTokenVerifier = Arc<dyn TokenVerifier + Send + Sync>;

pub async fn authenticate_bearer(
    State(verifier): State<SharedTokenVerifier>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = bearer_token(request.headers()).ok_or(StatusCode::UNAUTHORIZED)?;
    let principal = verifier
        .verify_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(principal);

    Ok(next.run(request).await)
}

pub async fn require_scope(
    State(required_scope): State<Arc<str>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let principal = request
        .extensions()
        .get::<AuthenticatedPrincipal>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !principal.has_scope(required_scope.as_ref()) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    let header_value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let mut parts = header_value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;

    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(token)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::{
        Extension, Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{AuthenticatedPrincipal, TokenVerificationError, TokenVerifier};

    use super::{SharedTokenVerifier, authenticate_bearer, require_scope};

    #[derive(Clone, Default)]
    struct StubTokenVerifier {
        responses: HashMap<String, Result<AuthenticatedPrincipal, TokenVerificationError>>,
    }

    impl StubTokenVerifier {
        fn with_token(
            mut self,
            token: &str,
            result: Result<AuthenticatedPrincipal, TokenVerificationError>,
        ) -> Self {
            self.responses.insert(token.to_string(), result);
            self
        }
    }

    impl TokenVerifier for StubTokenVerifier {
        fn verify_access_token(
            &self,
            token: &str,
        ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
            self.responses
                .get(token)
                .cloned()
                .unwrap_or(Err(TokenVerificationError::MalformedToken))
        }
    }

    fn principal_with_scopes(scopes: &[&str]) -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid"),
            scopes.iter().map(|scope| scope.to_string()),
        )
    }

    fn verifier_state(verifier: StubTokenVerifier) -> SharedTokenVerifier {
        Arc::new(verifier)
    }

    fn app(verifier: SharedTokenVerifier) -> Router {
        let protected = Router::new()
            .route("/protected", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(
                Arc::<str>::from("assets:ingest"),
                require_scope,
            ))
            .route_layer(middleware::from_fn_with_state(
                verifier,
                authenticate_bearer,
            ));

        Router::new()
            .route("/public", get(public_handler))
            .merge(protected)
    }

    async fn protected_handler(Extension(principal): Extension<AuthenticatedPrincipal>) -> String {
        principal.subject_id.to_string()
    }

    async fn public_handler() -> &'static str {
        "public"
    }

    #[tokio::test]
    async fn missing_bearer_token_returns_401() {
        let app = app(verifier_state(StubTokenVerifier::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn malformed_bearer_header_returns_401() {
        let app = app(verifier_state(StubTokenVerifier::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Basic not-a-bearer-token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_returns_401_without_echoing_token() {
        let token = "expired-token-value";
        let verifier =
            StubTokenVerifier::default().with_token(token, Err(TokenVerificationError::Expired));
        let app = app(verifier_state(verifier));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024).await.expect("body");

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(body.is_empty());
        assert!(!String::from_utf8_lossy(&body).contains(token));
    }

    #[tokio::test]
    async fn valid_token_inserts_extractable_principal() {
        let verifier = StubTokenVerifier::default().with_token(
            "valid-token",
            Ok(principal_with_scopes(&["assets:ingest", "assets:read"])),
        );
        let app = app(verifier_state(verifier));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Bearer valid-token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024).await.expect("body");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            String::from_utf8(body.to_vec()).expect("utf8"),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[tokio::test]
    async fn missing_scope_returns_403() {
        let verifier = StubTokenVerifier::default()
            .with_token("valid-token", Ok(principal_with_scopes(&["assets:read"])));
        let app = app(verifier_state(verifier));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Bearer valid-token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn public_route_remains_accessible_without_authentication() {
        let app = app(verifier_state(StubTokenVerifier::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/public")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024).await.expect("body");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(String::from_utf8(body.to_vec()).expect("utf8"), "public");
    }
}
