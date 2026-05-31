# ADR-023: API client authentication and principal propagation

- **Status:** Accepted
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

## Context

DubBridge persists `assets.uploader_id` as part of the governance record. The S1
upload API cannot safely accept that identifier from an untrusted request body:
doing so would let a caller impersonate another uploader while creating an asset
and rights ledger entry.

The repository already reserves `crates/auth` for authentication and authorization
policy boundaries, but it currently contains only a placeholder. The first mutable
HTTP endpoints arrive in S1 Task 5, so API client identity must be established
before those handlers are implemented.

This concern is separate from live-source authentication in ADR-022. API client
authentication identifies the caller invoking Axum. Stream-source authentication
authorizes an RTMP or SRT encoder/source before capture starts.

## Decision

- `apps/api` acts as an OAuth 2.0 resource server. It accepts signed JWT access
  tokens from an external authorization server through
  `Authorization: Bearer <token>`. DubBridge does not implement login, password
  storage, token issuance, or refresh-token flows.
- JWT access-token validation fails closed and follows RFC 9068 / RFC 8725:
  verify the signature; pin the accepted asymmetric algorithm to `RS256`; require
  and validate `typ` as `at+jwt` or `application/at+jwt`, plus `iss`, `aud`, `exp`,
  and `sub`; validate `nbf` when present; reject malformed, expired, incorrectly
  targeted, or incorrectly signed tokens.
- `sub` must parse as a UUID in v1. The verified subject becomes
  `AuthenticatedPrincipal.subject_id` and is the only source for
  `assets.uploader_id`. Handlers must not accept `uploader_id` from request bodies.
- Authorization uses scopes carried by the verified access token:
  - `assets:ingest` for mutable upload-ingestion endpoints.
  - `assets:read` for asset reads.
  - `recordings:write` for recording create/start/stop endpoints.
  - `recordings:read` for recording reads.
- `/health/live` and `/health/ready` remain public.
- The v1 verifier accepts a configured public RSA verification key. The verifier
  boundary must allow a JWKS-backed implementation later without changing route
  handlers. JWKS discovery, automatic key rotation, the external authorization
  server deployment, and user lifecycle management are follow-ups.

## Consequences

**Positive**
- Persisted uploader identity is derived from a cryptographically verified
  principal instead of client-controlled JSON.
- S1 and S3 reuse one API authentication boundary while keeping RTMP/SRT source
  credentials separate.
- Asymmetric verification keeps private signing keys outside DubBridge.
- The verifier boundary permits a future JWKS implementation without coupling
  handlers to an identity provider.

**Negative / trade-offs**
- Local and deployed API instances need an issuer, audience, and RSA public key
  configuration before protected routes can be served.
- A static public key requires coordinated configuration updates for rotation in
  v1. Automatic JWKS refresh is required before production scale demands
  zero-downtime rotation.
- Requiring a UUID `sub` constrains the authorization-server contract. A subject
  mapping table is a follow-up if an external provider uses opaque non-UUID
  subjects.

## Alternatives considered

- **Accept `uploader_id` from the request body** — rejected: permits identity
  spoofing and weakens the audit ledger.
- **Shared API keys only** — rejected: cannot provide a stable per-uploader
  principal for the existing asset schema.
- **Implement login and token issuance inside DubBridge** — rejected: unnecessary
  identity-provider scope for the MVP resource server.
- **Fetch JWKS in the first implementation** — deferred: useful for automated key
  rotation, but not required to establish the handler contract before S1 Task 5.

## Related

- [RFC 9068](https://www.rfc-editor.org/rfc/rfc9068) (JWT profile for OAuth 2.0
  access tokens).
- [RFC 8725](https://www.rfc-editor.org/rfc/rfc8725) (JWT best current practices).
- ADR-008 (rights ledger fail-closed precondition) — uploader context is part of
  ingestion validation.
- ADR-018 (traceable governance events) — the authenticated subject becomes the
  auditable uploader identity.
- ADR-022 (RTMP/SRT source authentication) — separate edge-authentication layer
  for recorded streams.
- Implemented by: `docs/tasks/s0-api-client-authentication.md`,
  `crates/auth/src/{config,principal,verifier,axum}.rs`,
  `crates/config/src/lib.rs`.
