# Tasks: S0 API Client Authentication

Governing plan: `docs/plan/s0-api-client-authentication.md`
Governing ADR: `docs/adr/ADR-023-api-client-authentication-and-principal-propagation.md`

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

## Default model recommendation (per AGENTS.md)
- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

---

## Task 1 — JWT verifier and typed principal

**Effort:** M
**Complexity:** Medium
**Depends on:** nothing
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Replace the `crates/auth` placeholder with a fail-closed JWT access-token verifier
and a typed `AuthenticatedPrincipal`.

### Scope
- Add `AuthConfig` with expected issuer, audience, RSA public-key path, and allowed
  clock-skew leeway.
- Add `AuthenticatedPrincipal { subject_id: Uuid, scopes }`.
- Add a `TokenVerifier` trait so verification-key retrieval can move to JWKS later
  without changing handlers.
- Implement an RSA JWT verifier that pins `RS256`, accepts RFC 9068 `typ` values
  `at+jwt` and `application/at+jwt`, and validates signature, `iss`, `aud`, `exp`,
  UUID `sub`, and optional `nbf`.
- Add a scope-check helper on the principal.
- Add typed auth settings to `crates/config`.

### Acceptance criteria
- A correctly signed token with expected `typ`, `iss`, `aud`, unexpired `exp`, and
  UUID `sub` returns an `AuthenticatedPrincipal`.
- Missing or invalid signature, `typ`, `iss`, `aud`, `exp`, or UUID `sub` is
  rejected.
- A future `nbf` is rejected, with only configured clock-skew leeway allowed.
- The accepted algorithm is pinned to `RS256`; algorithm substitution is rejected.
- Scope parsing and `principal.has_scope(...)` are unit-tested.
- No private signing key is loaded by production code.
- `cargo test -p dubbridge-auth` and `cargo check --workspace` pass.

### Files affected
- `Cargo.toml`
- `crates/auth/Cargo.toml`
- `crates/auth/src/lib.rs`
- `crates/auth/src/config.rs` (new)
- `crates/auth/src/principal.rs` (new)
- `crates/auth/src/verifier.rs` (new)
- `crates/config/src/lib.rs`

### Status: [x]

### Evidence
- Added `AuthConfig`, `AuthenticatedPrincipal`, `TokenVerifier`, and
  `RsaJwtTokenVerifier` in `crates/auth`.
- Added fail-closed validation for `RS256`, `typ`, `iss`, `aud`, `exp`, UUID
  `sub`, and optional `nbf` with configured leeway.
- Added scope parsing plus `principal.has_scope(...)` coverage.
- Added typed `auth` runtime settings to `crates/config::AppConfig`.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo test -p dubbridge-auth`
  - `~/.cargo/bin/cargo check --workspace`

---

## Task 2 — Axum bearer middleware and scope authorization

**Effort:** M
**Complexity:** Medium
**Depends on:** Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Expose reusable Axum middleware that authenticates bearer tokens, propagates the
verified principal to handlers, and enforces route scopes.

### Scope
- Parse `Authorization: Bearer <token>` without logging token contents.
- Validate through an injected `TokenVerifier`.
- Insert `AuthenticatedPrincipal` into request extensions.
- Add reusable scope authorization for protected routers/handlers.
- Use a test-only Axum router to verify middleware behavior before S1 routes exist.
- Keep health endpoints public when S1 mounts protected routers.

### Acceptance criteria
- Missing, malformed, expired, or invalid bearer tokens return HTTP `401`.
- A valid bearer token inserts an extractable `AuthenticatedPrincipal`.
- A valid principal without the required scope returns HTTP `403`.
- A principal with the required scope reaches the protected test handler.
- Authentication failures and traces never include raw bearer tokens.
- Public-route behavior is covered by a test.
- `cargo test -p dubbridge-auth` and `cargo check --workspace` pass.

### Files affected
- `crates/auth/src/lib.rs`
- `crates/auth/src/axum.rs` (new)
- `crates/auth/Cargo.toml`

### Status: [x]

### Evidence
- Added Axum middleware in `crates/auth/src/axum.rs` for bearer-token
  authentication and scope enforcement.
- Middleware injects `AuthenticatedPrincipal` into request extensions after
  verifier success and returns `401` on missing, malformed, expired, or invalid
  bearer tokens.
- Scope enforcement returns `403` when the verified principal lacks the required
  scope.
- Added a test-only Axum router covering public-route behavior, `401`, `403`,
  and extractable principal propagation without echoing raw bearer tokens.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo test -p dubbridge-auth`
  - `~/.cargo/bin/cargo check --workspace`

---

## Agent handoff prompt (for delegation)

```text
You are implementing S0 API Client Authentication for DubBridge.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/s0-api-client-authentication.md
Tasks: docs/tasks/s0-api-client-authentication.md
ADR: docs/adr/ADR-023-api-client-authentication-and-principal-propagation.md

Work one task at a time in order: T1 -> T2. After each task:
1. Run the touched crate tests and `cargo check --workspace`.
2. Mark the task [x] and record evidence in the task document.
3. Report the result and WAIT for approval before starting the next task.

Hard invariants:
- Fail closed: malformed, expired, incorrectly signed, or incorrectly targeted
  JWT access tokens are rejected.
- Pin RS256 and require an RFC 9068 access-token `typ`; do not trust the
  token-provided algorithm.
- Never accept uploader identity from an ingestion request body. S1 must derive
  uploader_id from the verified principal UUID subject.
- Do not log bearer tokens or private key material.
- Keep API client auth separate from RTMP/SRT source credentials in ADR-022.
- Do NOT commit if any test is broken.
- All user-facing communication is in Spanish; code/docs/commits in English.
```
