# Plan: S0 API Client Authentication

**Roadmap position:** Foundation slice **S0** — execute before S1 Task 5, the first
mutable HTTP API surface.

## Objective

Establish a reusable, fail-closed authentication and authorization boundary for
Axum routes. Convert a verified OAuth 2.0 JWT access token into a typed
`AuthenticatedPrincipal`, then require route-specific scopes. S1 ingestion handlers
must derive `uploader_id` from that principal instead of trusting caller-supplied
JSON.

## Scope

### Included
- Replace the `crates/auth` placeholder with a JWT access-token verifier boundary.
- Validate signature, token type, issuer, audience, expiry, subject, and optional
  not-before time according to ADR-023.
- Parse the verified UUID `sub` into `AuthenticatedPrincipal.subject_id`.
- Parse token scopes and expose reusable scope checks.
- Add Axum middleware that validates the bearer token, inserts the principal into
  request extensions, and returns `401` or `403` as appropriate.
- Add deterministic tests using local test keys and a test-only Axum router.
- Add typed runtime configuration for issuer, audience, and public verification
  key path.

### Excluded (deferred)
- Login UI, password storage, token issuance, refresh tokens, and authorization
  server deployment.
- External identity-provider selection.
- JWKS discovery and automatic key rotation.
- User lifecycle tables and opaque external-subject mapping.
- RTMP stream keys and SRT passphrases (S3, ADR-022).

## Governing ADRs
- ADR-023: API client authentication and principal propagation.
- ADR-008: Rights ledger is a mandatory fail-closed precondition.
- ADR-018: Governance events must remain traceable.

## Affected Files

### crates/auth/
- `Cargo.toml` — add JWT and Axum dependencies.
- `src/lib.rs` — re-export auth boundary.
- `src/config.rs` (new) — verifier configuration.
- `src/principal.rs` (new) — typed authenticated principal and scope checks.
- `src/verifier.rs` (new) — token verifier trait and RSA JWT implementation.
- `src/axum.rs` (new) — bearer middleware and scope authorization helpers.

### crates/config/
- `src/lib.rs` — add typed auth runtime settings consumed by `apps/api`.

### root
- `Cargo.toml` — add shared JWT dependency.

## Design Decisions

### Resource-server boundary
DubBridge validates access tokens but does not issue them. The external
authorization server owns login and token lifecycle. This keeps S0 bounded to the
API resource-server responsibility.

### Verified principal owns uploader identity
For v1, JWT `sub` is a UUID. Once verified, it becomes
`AuthenticatedPrincipal.subject_id`. S1 Task 5 passes this value into
`FinalizeIngestionCommand.uploader_id`; request DTOs do not accept `uploader_id`.

### Strict token validation
The verifier pins `RS256`, accepts the RFC 9068 `typ` values `at+jwt` and
`application/at+jwt`, and validates `iss`, `aud`, `exp`, `sub`, and optional
`nbf`. Invalid tokens fail closed with HTTP `401`. Verified principals lacking a
required route scope fail with HTTP `403`.

### Axum integration
Authentication middleware inserts `AuthenticatedPrincipal` into request extensions.
Protected handlers extract the verified principal. This matches Axum's middleware
and extension model and keeps route handlers independent of token parsing.

### Key-source boundary
The first implementation loads a configured RSA public key. The verifier is exposed
behind a trait so a future JWKS-backed key source can add discovery and rotation
without modifying handlers.

## Module Dependencies

```text
apps/api    -> crates/auth, crates/config
crates/auth -> axum, jsonwebtoken, serde, uuid
```

S1 Task 5 then extends the graph:

```text
apps/api -> crates/auth -> verified AuthenticatedPrincipal.subject_id
         -> ingestion handler -> FinalizeIngestionCommand.uploader_id
```

## Execution Order

```text
S0 T1 JWT verifier + typed principal
  -> S0 T2 Axum middleware + scope authorization
  -> resume S1 T5 Axum ingestion endpoints
```

## Lines Affected After Implementation

Tracked per task in `docs/tasks/s0-api-client-authentication.md`.
