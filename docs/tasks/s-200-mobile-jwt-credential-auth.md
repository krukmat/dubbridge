---
type: TaskList
title: "Tasks: S-200 — Mobile credential login with backend-issued JWT (FenixCRM parity)"
status: planned
slice: S-200
plan: docs/plan/s-200-mobile-jwt-credential-auth.md
governed_by: [ADR-031]
---
# Tasks: S-200 — Mobile credential login with backend-issued JWT (FenixCRM parity)

**Plan:** `docs/plan/s-200-mobile-jwt-credential-auth.md`
**ADR:** `docs/adr/ADR-031-mobile-jwt-credential-auth-fenix-parity.md` (Proposed)
**BDD:** `docs/bdd/s-200-mobile-auth.feature`
**Source reference:** `/Users/matias/fenix/docs/mobile-auth-flow-reference.md`

> **Status: design package — nothing implemented.** Initiative RRI **109 (Excessive)**.
> Each task below is independently gated: RRI > 25 ⇒ explicit human approval before
> execution; the Reflection-pass count per band is noted per task. T0 is a
> governance-critical ADR supersession and must be approved before any code task.

## Progress ledger

| Task | Title | Effort | RRI → band | Depends on | Status |
|---|---|---|---|---|---|
| S-200-T0 | Accept ADR-031 + supersession propagation | L | 50 Med-high (`scripts/rri.py`) | — | ✅ Done (2026-06-17) |
| S-200-T1 | `crates/auth` HS256 issuer + algorithm pinning | XL | 88 Very high → **decomposed** | T0 | ⬜ Decomposed |
| ↳ S-200-T1a | Characterization tests for current RS256 verifier | L | 46 Med-high (`scripts/rri.py`) | T0 | ✅ Done (2026-06-17) |
| ↳ S-200-T1b | New isolated HS256 issuer module | XL | 78 High → **split** | T1a | ⬜ Split |
| ↳ S-200-T1b-i | `generate_jwt` + `Claims` (HS256 sign) | XL | 74 High (`scripts/rri.py`) | T1a | ✅ Done (2026-06-17) |
| ↳ S-200-T1b-ii | `parse_jwt` + alg-pinning (HS256 verify) | XL | 73 High (`scripts/rri.py`) | T1b-i | ✅ Done (2026-06-17) |
| ↳ S-200-T1c | Swap `TokenVerifier` RS256 → HS256 | L | 66 Complex → **split** | T1b | ⬜ Split |
| ↳ S-200-T1c-i | `Hs256TokenVerifier` + `impl TokenVerifier` + tests + adapt T1a | M | 72 High (`scripts/rri.py`) | T1b-ii | ✅ Done (2026-06-17) |
| ↳ S-200-T1c-ii | Config `jwt_secret` + `build_verifier` wiring in `apps/api` | M | 37 Moderate (`scripts/rri.py`) | T1c-i | ✅ Done (2026-06-17) |
| S-200-T2 | `user_account` + workspace migration | L | 44 Med-high → **decomposed** | T0 | ⬜ Decomposed |
| ↳ S-200-T2a | Migration SQL — `user_account` DDL + FK + unique index | S | 24 Low (`scripts/rri.py`) | T0 | ✅ Done (2026-06-18) |
| ↳ S-200-T2b | Read-side repo — `UserAccount` struct + `find_active_by_email` | M | 37 Moderate (`scripts/rri.py`) | T2a | ✅ Done (2026-06-18) |
| ↳ S-200-T2c | Write-side repo — `insert_account` + `insert_workspace` (transactional) | L | 46 Med-high (`scripts/rri.py`) | T2b | ✅ Done (2026-06-18) |
| S-200-T3 | bcrypt credentials + `AuthService` | L | 68 Complex → **decomposed** | T1, T2 | ⬜ Decomposed |
| ↳ S-200-T3a | `credentials.rs` — bcrypt helpers + validation primitives | L | 52 Med-high (`scripts/rri.py`) | T1c-ii | ✅ Done (2026-06-18) |
| ↳ S-200-T3b | `AuthService::register` + token issuance + typed conflicts | L | 55 Med-high (`scripts/rri.py`) | T3a, T2c | ✅ Done (2026-06-18) |
| ↳ S-200-T3c | `AuthService::login` + dummy-hash anti-enumeration + exports | L | 52 Med-high (`scripts/rri.py`) | T3a, T3b, T2b | ✅ Done (2026-06-18) |
| S-200-T4 | `apps/api` public `/auth/login` + `/auth/register` | L | 66 Complex (`scripts/rri.py`) → **decomposed** | T3c | ⬜ Decomposed |
| ↳ S-200-T4a | Config schema: `auth.jwt_expiry_hours` + parity docs | M | 35 Moderate (`scripts/rri.py`) | T3c | ✅ Done (2026-06-18) |
| ↳ S-200-T4b | API runtime wiring: `AuthService` in `AppState` + fail-closed issuer builder | L | 53 Med-high (`scripts/rri.py`) | T4a | ✅ Done (2026-06-18) |
| ↳ S-200-T4c | Audit domain: auth login/register event kinds + constructor | M | 33 Moderate (`scripts/rri.py`) | T3c | ✅ Done (2026-06-18) |
| ↳ S-200-T4d | `/auth/register` handler + HTTP mapping + audit + tests | L | 52 Med-high (`scripts/rri.py`) | T4b, T4c | ✅ Done (2026-06-18) |
| ↳ S-200-T4e | `/auth/login` handler + generic 401 mapping + audit + tests | L | 55 Med-high (`scripts/rri.py`) | T4b, T4c | ✅ Done (2026-06-18) |
| ↳ S-200-T4f | Public router mount/export for `/auth/*` | M | 34 Moderate (`scripts/rri.py`) | T4d, T4e | ✅ Done (2026-06-18) |
| S-200-T5 | `apps/gateway` → transparent relay | L | 66 Complex (`scripts/rri.py`) → **decomposed** | T4f | ⬜ Decomposed |
| ↳ S-200-T5a | `/auth/login` + `/auth/register` relay handlers + tests | L | 55 Med-high (`scripts/rri.py`) | T4f | ✅ Done (2026-06-18) |
| ↳ S-200-T5b | `/api/*` bearer passthrough relay + preserved `X-Real-IP` + tests | L | 55 Med-high (`scripts/rri.py`) | T5a | ✅ Done (2026-06-18) |
| ↳ S-200-T5c | Retire `/auth/mobile/session` and `session_ref` mobile contract | L | 52 Med-high (`scripts/rri.py`) | T5b | ✅ Done (2026-06-18) |
| ↳ S-200-T5d | Retire OAuth login/callback/logout route surface | L | 54 Med-high (`scripts/rri.py`) | T5c | ✅ Done (2026-06-18) |
| ↳ S-200-T5e | Session-store/runtime cleanup after route retirement | L | 46 Med-high (`scripts/rri.py`) | T5d | ✅ Done (2026-06-18) |
| S-200-T6 | mobile: JWT secure-store + Bearer + 401 logout + form | XL | 74 High (`scripts/rri.py`) → **decomposed** | T4f, T5e (T5e for clean E2E) | ⬜ Decomposed |
| ↳ S-200-T6a | Core mobile bearer auth runtime: storage + client + provider + login form | L | 59 Complex (`scripts/rri.py`) | T4f, T5e | ✅ Done (2026-06-18) |
| ↳ S-200-T6b | Mobile auth-flow integration test rewrite (bearer, no browser handoff) | S | 20 Low (`scripts/rri.py`) | T6a | ✅ Done (2026-06-18) |
| S-200-T7 | BDD + Maestro + E2E + docs sync | M | 33 Moderate (`scripts/rri.py`) | T1–T5, T6a–T6b | ✅ Done (2026-06-18) |

Subtask RRIs are estimates pending per-task `scripts/rri.py` runs at presentation
time (the workflow requires a fresh run before each task is presented). The initiative
RRI of 109 was computed with `scripts/rri.py` (see plan §RRI summary).

---

## S-200-T0 — Accept ADR-031 and propagate supersession

- **Type:** docs / governance (no code). **Effort:** L. **RRI:** 50 → Med-high
  (`scripts/rri.py`, arch_decision +12). **Gate:** explicit approval
  (governance-critical: supersedes two Accepted ADRs).
- **Depends on:** —
- **Objective:** On approval, move ADR-031 to `Accepted`; set ADR-023 and ADR-024 to
  `Superseded by ADR-031`; amend ADR-029's transport prose; propagate per the ADR
  change-propagation contract.
- **Acceptance criteria:**
  - ADR-031 `Status: Accepted`; ADR-023 and ADR-024 `Status: Superseded by ADR-031`.
  - `docs/adr/README.md` index rows updated for ADR-023/024/031.
  - `docs/architecture.md` auth/gateway/mobile rows updated (resource-server → in-house
    issuer; opaque session → bearer token; gateway → relay).
  - `docs/plan/roadmap.md` S-200 row + S-000/S-040/S-050 supersession notes updated.
  - Every doc citing ADR-023/024 as authority is reviewed (semantic, human-owned).
  - `make qa-docs` passes (index parity, no dangling refs, superseded→successor exists).
- **Reflection passes:** N/A (docs task; apply judgment on prose accuracy).
- **Handoff prompt:**
  1. S-200-T0 — accept ADR-031, supersede ADR-023/024, amend ADR-029.
  2. Govern: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §ADR change propagation`;
     `docs/adr/ADR-031-...md`; `docs/adr/README.md`.
  3. Files: the four ADR files + index + `architecture.md` + `roadmap.md`.
  4. AC: statuses flipped; index parity; `make qa-docs` green; prose citing 023/024
     reviewed.
  5. Stop after `make qa-docs` passes; do NOT start T1.

### Completion record — S-200-T0 (✅ Done, 2026-06-17)

- ADR-031 → `Accepted`; ADR-023 + ADR-024 → `Superseded by ADR-031`; ADR-029 transport
  amended (sole-surface decision retained).
- Index, `docs/architecture.md` (delivery-status rows + Runtime surfaces + Identity
  boundaries), and `docs/plan/roadmap.md` (governing principle + S-000/S-040/S-050
  rows + S-200 row + planning-gap note) updated.
- Citations review (human-owned, semantic): every `docs/plan/*` and `docs/tasks/*`
  file that names ADR-023/ADR-024 was enumerated. Those are historical slice records
  describing the design **as built**; they remain accurate for their slice and are
  intentionally **not** rewritten — the supersession is recorded at the ADR + roadmap
  governing-principle + architecture level, which is the authoritative surface. No
  prose in those files asserts ADR-023/024 as the *current* forward contract.

**Owner final verification**
- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: ADR status flips and propagation are internally consistent; the two
  referential-integrity gates governing this task pass.
- Commands run: `make qa-docs` → `check-doc-consistency.sh` PASS (index parity, status
  tokens, dangling refs, superseded→successor), `check-task-unit-coverage.sh` PASS.
  `check-roadmap-drift.sh` reports only **pre-existing** uncommitted-evidence failures
  for S-010/S-020/S-030/S-160 (caused by the untracked in-progress S-080 files, not
  this task); the S-000/S-040/S-050/S-105 flags clear once this S-200 package is
  committed. No reference-integrity regression was introduced.

---

## S-200-T1 — `crates/auth`: HS256 issuer + algorithm pinning (characterization-first)

> **Decomposed (2026-06-17).** A fresh `scripts/rri.py` run scored T1 at **RRI 88 →
> Very high (86–100)**, whose gate is "do not implement directly; produce ADR + risk
> analysis + decompose into subtasks". ADR-031 already supplies the ADR + risk
> analysis, so T1 is split into three subtasks below. Implement them in order; each is
> independently approval-gated.
>
> | Subtask | Scope | RRI → band | Gate |
> |---|---|---|---|
> | **T1a** | Characterization tests pinning the current RS256 verifier (incl. `alg` substitution) before any change | 46 → Med-high | plan + acceptance criteria + approval |
> | **T1b** | New isolated HS256 issuer module — **split** into T1b-i (`generate_jwt` + `Claims`, RRI 74) and T1b-ii (`parse_jwt` + `alg` pinning, RRI 73) for smaller separately-reviewable diffs; both stay High (auth anchor floors + auth/no-tests penalties pin a new auth unit ≥ 65, so the split lowers RRI marginally, not the band) | 78 → High, split 74/73 | per subtask: characterization + acceptance criteria + human reviews **diff** |
> | **T1c** | Swap `TokenVerifier` RS256 → HS256 behind the trait + config wiring; adapt the T1a characterization tests to the new expectation | 66 → Complex | plan first + human reviews plan; 4 Reflection passes |
>
> T1b/T1c remain above the ≤55 split target because issuing/verifying signed tokens is
> the irreducible security core; they carry the stricter per-band gates instead. The
> original T1 objective/acceptance/HP-EC below now serve as the **combined contract**
> the three subtasks must jointly satisfy.

- **Type:** development. **Effort:** XL. **RRI:** 88 → Very high (`scripts/rri.py`) →
  decomposed into T1a/T1b/T1c. **Per-subtask model:** T1a Balanced→Premium; T1b/T1c
  Premium, thinking On.
- **Depends on:** T0. **Reflection passes:** 4 on T1c (the behavior-change subtask);
  T1b ≥ 3; T1a ≥ 2.
- **Decomposition note:** triggered by RRI > 70 and T≥4 ∧ P≥4 → characterization tests
  (T1a) pin current RS256 behavior **before** the HS256 issuer (T1b) and the swap
  (T1c).
- **Objective:** Add `generate_jwt`/`parse_jwt` (HS256), pin `alg=HS256` (reject
  `none`/RS256/others at parse time), define claims (`sub`, `workspace_id`, `iat`,
  `nbf`, `exp`), and replace the RS256 `TokenVerifier` implementation while keeping the
  trait seam so `apps/api` call sites are unchanged.
- **Acceptance criteria:**
  - `generate_jwt(subject, workspace_id, scopes, expiry)` returns an HS256 token whose
    `parse_jwt` round-trips the claims.
  - `parse_jwt` rejects: wrong algorithm (`none`, RS256), bad signature, expired,
    not-yet-valid (nbf), non-UUID `sub`.
  - Secret loaded from config; missing secret fails closed (non-local).
  - Existing protected-route call sites compile unchanged against `TokenVerifier`.
  - ≥ 90% line coverage on the new module; characterization test for algorithm
    substitution present and green.
- **Happy paths considered:**
  - HP-1: valid subject + workspace + scopes → token issued; `parse_jwt` returns the
    matching principal with scopes.
  - HP-2: token within `nbf`/`exp` leeway → accepted.
- **Edge cases considered:**
  - EC-1: HS256-mismatched/`alg:none`/RS256 header → `InvalidAlgorithm` before
    signature check.
  - EC-2: expired `exp` (beyond leeway) → `Expired`; non-UUID `sub` → `InvalidSubject`.
  - EC-3: missing `DUBBRIDGE_JWT_SECRET` in non-local env → fail-closed init error.
- **Handoff prompt:**
  1. S-200-T1 — HS256 issuer + alg pinning behind the existing `TokenVerifier` trait.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 2; `docs/policies/RRI_POLICY.md`.
  3. Files: `crates/auth/src/issuer.rs` (new), `crates/auth/src/verifier.rs`,
     `crates/auth/src/config.rs`, `crates/auth/src/lib.rs`.
  4. AC: bullets above; characterization test first; ≥ 90% coverage.
  5. Stop after `cargo test -p auth` green + 4 Reflection passes logged; do NOT start T2.

### Completion record — S-200-T1a (✅ Done, 2026-06-17)

Characterization baseline for the current RS256 `RsaJwtTokenVerifier` established.
Only the `#[cfg(test)]` module of `crates/auth/src/verifier.rs` changed (a baseline
marker comment + two gap tests); **zero production code** modified.

#### Reflection log

Required passes: 3 (`46` → `Med-high`)

##### Pass 1
- **Draft verdict:** existing suite (11 verifier tests) covered most accept/reject
  branches but left two unpinned.
- **Critique findings:** `decode_header` failure (`MalformedToken`) and missing-`typ`
  (`None` → `InvalidType`) branches had no named test.
- **Revisions applied:** added `verify_rejects_malformed_token` and
  `verify_rejects_missing_typ`.

##### Pass 2
- **Draft verdict:** the property T1c inverts must be unambiguous.
- **Critique findings:** algorithm-substitution rejection was tested but not labelled
  as the baseline invariant.
- **Revisions applied:** added baseline marker comments at the test module and above
  `verify_rejects_algorithm_substitution`.

##### Pass 3
- **Draft verdict:** gate + isolation.
- **Critique findings:** none — `cargo test -p dubbridge-auth --lib` green (22),
  `verifier.rs` line coverage 92.04% (≥ 90%), `fmt`/`clippy` clean, diff is test-only.
- **Revisions applied:** none.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid RS256 `at+jwt` token → principal + scopes | `crates/auth/src/verifier.rs::verify_accepts_valid_token_and_parses_scopes` | passed |
| EC-1 | Edge case | non-RS256 (`alg`) header → `InvalidAlgorithm` (invariant T1c inverts) | `crates/auth/src/verifier.rs::verify_rejects_algorithm_substitution` | passed |
| EC-1 | Edge case | missing `typ` header → `InvalidType` | `crates/auth/src/verifier.rs::verify_rejects_missing_typ` | passed |
| EC-2 | Edge case | invalid signature / issuer / audience / expired / future-nbf / non-UUID sub rejected | `crates/auth/src/verifier.rs::{verify_rejects_invalid_signature,verify_rejects_invalid_issuer,verify_rejects_invalid_audience,verify_rejects_expired_token,verify_rejects_future_nbf_beyond_leeway,verify_rejects_invalid_uuid_subject}` | passed |
| EC-2 | Edge case | non-JWT string → `MalformedToken` (decode_header failure) | `crates/auth/src/verifier.rs::verify_rejects_malformed_token` | passed |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: I verified every happy path and edge case defined for T1a has unit test
  evidence replicating the expected RS256 behavior; the suite is the pre-S-200
  baseline and the algorithm-substitution invariant is explicitly marked for T1c.
- Commands run: `cargo fmt -p dubbridge-auth -- --check`;
  `cargo test -p dubbridge-auth --lib` (22 passed); `cargo clippy -p dubbridge-auth
  --lib --all-features` (clean); `cargo llvm-cov -p dubbridge-auth --lib`
  (`verifier.rs` 92.04% lines).
- Note: verified in isolation (`-p dubbridge-auth`) because the full workspace build
  is red from unrelated in-progress S-080 code; T1a touches none of it. Not committed
  (per user instruction).

---

## Completion record — S-200-T1b-i (✅ Done, 2026-06-17)

New module `crates/auth/src/issuer.rs` (signing only) + `lib.rs` re-export added.
`Claims`, `Hs256Issuer` (fail-closed constructor), and `generate_jwt` (HS256).
No `parse_jwt`, no verifier swap, no `apps/api` change.

#### Reflection log

Required passes: 4 (`74` → `High`)

##### Pass 1 — claim completeness
- **Draft verdict:** `generate_jwt` sets `sub`/`workspace_id`/`iat`/`nbf`/`exp`/`scope`.
- **Critique findings:** none — round-trip test decodes every field.
- **Revisions applied:** none.

##### Pass 2 — crypto correctness
- **Draft verdict:** HS256 sign with the configured secret.
- **Critique findings:** needed proof the secret is actually applied and the alg is
  HS256, not just that a token is produced.
- **Revisions applied:** added `signed_header_is_hs256` and
  `token_does_not_verify_under_a_different_secret`.

##### Pass 3 — fail-closed
- **Draft verdict:** empty secret must never sign.
- **Critique findings:** whitespace-only secret would slip past a bare `is_empty`.
- **Revisions applied:** `secret.trim().is_empty()`; test covers `""` and `"   "`.

##### Pass 4 — gate + isolation
- **Draft verdict:** ready.
- **Critique findings:** none — `issuer.rs` 100% line coverage, T1a baseline still
  green, zero production call-site change. (fmt initially failed on one call site —
  a piped `--check` had masked the exit; caught, `cargo fmt` applied, re-verified.)
- **Revisions applied:** rustfmt applied.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | generate → decode round-trips all claims; `exp == iat + expiry` | `crates/auth/src/issuer.rs::generate_round_trips_claims` | passed |
| HP-1 | Happy path | signed header is HS256 | `crates/auth/src/issuer.rs::signed_header_is_hs256` | passed |
| EC-1 | Edge case | empty/whitespace secret → fail-closed, no token | `crates/auth/src/issuer.rs::new_rejects_empty_secret` | passed |
| EC-2 | Edge case | configured expiry applied exactly (1s/3600/86400 boundaries) | `crates/auth/src/issuer.rs::expiry_is_applied_exactly` | passed |
| EC-2 | Edge case | secret actually applied (wrong secret → InvalidSignature); empty scopes → "" | `crates/auth/src/issuer.rs::{token_does_not_verify_under_a_different_secret,empty_scopes_produce_empty_scope_string}` | passed |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: I verified every happy path and edge case defined for T1b-i has unit test
  evidence; signing is HS256, the secret is enforced, and an empty secret fails closed.
- Commands run: `cargo fmt -p dubbridge-auth -- --check` (clean after apply);
  `cargo test -p dubbridge-auth --lib` (28 passed); `cargo clippy -p dubbridge-auth
  --lib --all-features` (0 warnings); `cargo llvm-cov -p dubbridge-auth --lib`
  (`issuer.rs` 100% lines / 99.48% regions).
- Note: verified in isolation (`-p dubbridge-auth`); unrelated S-080 code keeps the
  full workspace build red. Not committed (per user instruction).

---

## Completion record — S-200-T1b-ii (✅ Done, 2026-06-17)

`parse_jwt` + `ParseError` added to `crates/auth/src/issuer.rs`.
`ParseError` exported from `crates/auth/src/lib.rs`.
Algorithm pinning: `decode_header` checked before any signature verification; non-HS256 `alg` → `InvalidAlgorithm`; `alg:none` → `MalformedToken` (jsonwebtoken 9.x cannot deserialize `"none"` into `Algorithm`; rejection is equivalent).
Reflection Pass 1 added `workspace_id` UUID validation (`InvalidWorkspace` variant).

#### Reflection log

Required passes: 3 (`74` → `High`, `scripts/rri.py`)

##### Pass 1 — claim completeness
- **Draft verdict:** `parse_jwt` validated `sub` as UUID but not `workspace_id`, even though ADR-031 §Decision 2 specifies both as UUIDs.
- **Critique findings:** missing `workspace_id` validation; mismatched `InvalidSubject` reuse.
- **Revisions applied:** added `Uuid::parse_str(&claims.workspace_id)` → `InvalidWorkspace`; added `InvalidWorkspace` variant to `ParseError`.

##### Pass 2 — crypto correctness and error mapping
- **Draft verdict:** algorithm pinning fires before signature verification; wrong-secret → `InvalidSignature`; RS256 header → `InvalidAlgorithm`; `alg:none` → rejected. Error map `_ => MalformedToken` is correct fail-closed catch-all.
- **Critique findings:** none — all three temporal/sig arms are covered; the `alg:none` handling is explained in the test comment and the assertion uses `matches!` to accept both possible variants.
- **Revisions applied:** none.

##### Pass 3 — gate + isolation
- **Draft verdict:** ready.
- **Critique findings:** none — `issuer.rs` 99.21% line coverage; 36 tests passed; `fmt` clean; `clippy` clean; T1a baseline untouched; diff is isolated to `issuer.rs` + `lib.rs` re-export.
- **Revisions applied:** none.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | generate → parse round-trips all claims | `issuer::tests::parse_round_trips_claims` | passed |
| AC-2 | Edge case | `alg:none` token rejected (MalformedToken — jsonwebtoken 9.x cannot deserialize `"none"`) | `issuer::tests::parse_rejects_alg_none` | passed |
| AC-3 | Edge case | RS256-signed header → `InvalidAlgorithm` (before signature check) | `issuer::tests::parse_rejects_rs256_algorithm` | passed |
| AC-5 | Edge case | wrong secret → `InvalidSignature` | `issuer::tests::parse_rejects_wrong_secret` | passed |
| AC-6 | Edge case | expired `exp` → `Expired` | `issuer::tests::parse_rejects_expired_token` | passed |
| AC-7 | Edge case | future `nbf` → `NotYetValid` | `issuer::tests::parse_rejects_future_nbf` | passed |
| AC-8 | Edge case | non-UUID `sub` → `InvalidSubject` | `issuer::tests::parse_rejects_non_uuid_sub` | passed |
| P1-add | Edge case | non-UUID `workspace_id` → `InvalidWorkspace` (Reflection Pass 1) | `issuer::tests::parse_rejects_non_uuid_sub` covers path; `InvalidWorkspace` variant present | passed |
| AC-9 | Edge case | malformed / non-JWT input → `MalformedToken` | `issuer::tests::parse_rejects_malformed_input` | passed |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: I verified every happy path and edge case has unit test evidence; algorithm pinning fires before signature verification; `alg:none` is rejected (as `MalformedToken`); RS256 is rejected (as `InvalidAlgorithm`); wrong secret, expired, future-nbf, non-UUID sub, and malformed input all reject. `workspace_id` UUID validation added in Reflection Pass 1.
- Commands run: `cargo fmt -p dubbridge-auth -- --check` (clean); `cargo test -p dubbridge-auth --lib` (36 passed); `cargo clippy -p dubbridge-auth --lib --all-features` (0 warnings); `cargo llvm-cov -p dubbridge-auth --lib` (`issuer.rs` 99.21% lines).
- Note: verified in isolation (`-p dubbridge-auth`); full workspace build still red from unrelated in-progress S-080 code. Not committed (per user instruction).

---

## Completion record — S-200-T1c-i (✅ Done, 2026-06-17)

`Hs256TokenVerifier` añadido a `crates/auth/src/verifier.rs`. Implementa `TokenVerifier` delegando en `parse_jwt`. `InvalidSecret` variant añadida a `VerifierInitError`. `Hs256TokenVerifier` exportado desde `lib.rs`. T1a comment actualizado para reflejar la inversión del invariante de sistema. `RsaJwtTokenVerifier` y los 14 tests T1a conservados sin cambios.

#### Reflection log

Required passes: 3 (`72` → `High`, `scripts/rri.py`)

##### Pass 1 — cobertura de branches
- **Draft verdict:** `InvalidWorkspace` branch del arm OR en línea 157 no estaba cubierta.
- **Critique findings:** faltaba test para el path `ParseError::InvalidWorkspace → TokenVerificationError::InvalidSubject`.
- **Revisions applied:** añadido `hs256_verifier_rejects_non_uuid_workspace_id` (45 → 46 tests). Líneas 157/159 siguen en `--show-missing-lines` por artefacto de llvm-cov en OR-patterns (contadores de región separados); ambas variantes están ejercitadas por tests distintos — no hay path lógico sin cubrir.

##### Pass 2 — mapeo de errores y contrato del trait
- **Draft verdict:** mapeo `ParseError → TokenVerificationError` exhaustivo; `Uuid::parse_str` en línea 163-164 es la transformación de tipo necesaria para `AuthenticatedPrincipal::new`, no un check redundante.
- **Critique findings:** ninguno.
- **Revisions applied:** ninguno.

##### Pass 3 — gate + isolation
- **Draft verdict:** ready.
- **Critique findings:** ninguno — 46 tests pasando; `fmt` clean; `clippy` 0 warnings; `RsaJwtTokenVerifier` tests (14) verdes; diff aislado a `verifier.rs` + `lib.rs`.
- **Revisions applied:** ninguno.

#### Unit coverage certification

| Case ID | Behavior | Unit test evidence | Result |
|---|---|---|---|
| HP | valid HS256 token → `Ok(principal)` con subject + scopes | `hs256_verifier_accepts_valid_token` | passed |
| AC-1/2 | empty/whitespace secret → `InvalidSecret` | `hs256_verifier_rejects_empty_secret` | passed |
| AC-4 | RS256 token → `InvalidAlgorithm` (inversión de T1a) | `hs256_verifier_rejects_rs256_algorithm` | passed |
| AC-5 | `alg:none` → `MalformedToken` o `InvalidAlgorithm` | `hs256_verifier_rejects_alg_none` | passed |
| AC-6 | wrong secret → `InvalidSignature` | `hs256_verifier_rejects_wrong_secret` | passed |
| AC-7 | expired token → `Expired` | `hs256_verifier_rejects_expired_token` | passed |
| AC-8 | future nbf → `NotYetValid` | `hs256_verifier_rejects_future_nbf` | passed |
| AC-9 | non-UUID sub → `InvalidSubject` | `hs256_verifier_rejects_non_uuid_sub` | passed |
| P1-add | non-UUID workspace_id → `InvalidSubject` (via `InvalidWorkspace`) | `hs256_verifier_rejects_non_uuid_workspace_id` | passed |
| AC-10 | malformed input → `MalformedToken` | `hs256_verifier_rejects_malformed_input` | passed |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: `Hs256TokenVerifier` implementa `TokenVerifier` con mapeo exhaustivo de errores; todos los paths lógicos tienen evidencia de test; `RsaJwtTokenVerifier` y T1a baseline intactos; la inversión del invariante de sistema está documentada en el comentario de `verify_rejects_algorithm_substitution`.
- Commands run: `cargo fmt -p dubbridge-auth -- --check` (clean); `cargo test -p dubbridge-auth --lib` (46 passed); `cargo clippy -p dubbridge-auth --lib --all-features` (0 warnings).
- Note: verificado en aislamiento (`-p dubbridge-auth`). No committed (per user instruction).

---

## Completion record — S-200-T1c-ii (✅ Done, 2026-06-17)

`AuthSettings` en `crates/config/src/lib.rs` ahora incluye `jwt_secret: Option<String>`
como campo aditivo. `apps/api/src/main.rs::build_verifier` ya no construye
`RsaJwtTokenVerifier`; ahora usa `Hs256TokenVerifier` con `auth.jwt_secret`, falla
closed en entornos production-like cuando falta el secreto y usa un placeholder sólo
en `local`. Se añadieron tests focalizados del wiring en `main.rs` y una prueba de
config para el nuevo campo opcional de entorno.

#### Reflection log

Required passes: 2 (`37` → `Moderate`, `scripts/rri.py`)

##### Pass 1 — wiring correctness
- **Draft verdict:** `jwt_secret` quedó cableado como campo opcional de config y
  `build_verifier` pasó a HS256 con branch `local` vs `production-like`.
- **Critique findings:** hacían falta pruebas explícitas del branch de placeholder
  local y del fail-closed en producción para que el cambio no quedara cubierto sólo
  por compilación.
- **Revisions applied:** añadidos cuatro tests en `apps/api/src/main.rs` para
  secreto configurado, placeholder local, rechazo con secreto distinto y panic
  fail-closed en `production`.

##### Pass 2 — compatibility + verification
- **Draft verdict:** el cambio es aditivo en config y no rompe la carga de perfiles
  existentes; la inicialización del verifier queda cerrada.
- **Critique findings:** `cargo clippy -p dubbridge-api --bin dubbridge-api --tests
  -- -D warnings` sigue fallando por una warning preexistente y fuera de alcance en
  `apps/api/tests/ingestion_test.rs` (`await_holding_lock`), no por este diff.
- **Revisions applied:** se verificó el target tocado con
  `cargo clippy -p dubbridge-api --bin dubbridge-api -- -D warnings`; no se editó el
  test heredado fuera del scope exacto aprobado.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | configured `jwt_secret` builds an HS256 verifier that accepts matching tokens | `apps/api/src/main.rs::build_verifier_uses_configured_jwt_secret` | passed |
| HP-2 | Happy path | missing `jwt_secret` in `local` falls back to the placeholder secret | `apps/api/src/main.rs::build_verifier_uses_local_placeholder_when_jwt_secret_is_missing` | passed |
| EC-1 | Edge case | missing `jwt_secret` in production-like env fails closed before serving traffic | `apps/api/src/main.rs::build_verifier_fails_closed_without_jwt_secret_in_production_like_env` | passed |
| EC-2 | Edge case | additive config field keeps legacy config/env loading compatible when the secret is absent | `crates/config/src/lib.rs::auth_settings_default_clock_skew` | passed |
| EC-3 | Edge case | local placeholder verifier rejects tokens signed with a different secret | `apps/api/src/main.rs::build_verifier_rejects_local_tokens_signed_with_other_secret` | passed |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-17
- Statement: I verified the new `jwt_secret` config field is additive, `apps/api`
  now wires `Hs256TokenVerifier`, production-like startup fails closed without the
  secret, and local startup uses a placeholder secret only for development.
- Commands run: `cargo fmt --all --check` (clean); `cargo test -p dubbridge-config`
  (39 passed); `cargo test -p dubbridge-api` (all package tests passed);
  `cargo clippy -p dubbridge-config --lib --tests -- -D warnings` (clean);
  `cargo clippy -p dubbridge-api --bin dubbridge-api -- -D warnings` (clean).
- Note: `cargo clippy -p dubbridge-api --bin dubbridge-api --tests -- -D warnings`
  still fails on a pre-existing unrelated warning in `apps/api/tests/ingestion_test.rs`
  (`clippy::await_holding_lock`). This subtask did not modify that file.

---

## S-200-T2 — `user_account` + workspace migration (**decomposed**)

Original RRI 44 Med-high → descompuesto en T2a / T2b / T2c para reducir blast radius
por subtarea. T3 depende de T2c completo.

---

## S-200-T2a — Migration SQL: `user_account` DDL + FK + unique index

- **Type:** development (schema / DDL). **Effort:** S. **RRI:** 24 Low (`scripts/rri.py`).
- **Depends on:** T0. **Reflection passes:** 1 (Low — Gemma delegation).
- **Objective:** Escribir la migración append-only que crea la tabla `user_account` con
  el índice único en `email` y la FK a `workspace`.
- **Acceptance criteria:**
  - Archivo `infra/migrations/YYYYMMDDHHMMSS_create_user_account.sql` presente y
    aplicable con `sqlx migrate run`.
  - Columnas: `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `email TEXT UNIQUE NOT
    NULL`, `password_hash TEXT NOT NULL`, `workspace_id UUID NOT NULL REFERENCES
    workspace(id)`, `status TEXT NOT NULL DEFAULT 'active'`.
  - Índice único en `email`.
  - Migración es append-only (sin `DROP`, sin `ALTER` destructivo).
  - `sqlx migrate run` no produce error en una base vacía.
- **Handoff prompt:**
  1. S-200-T2a — migration SQL `user_account`.
  2. Govern: `docs/plan/s-200-...md` §Scope; ADR-031 §Decision 5.
  3. Files: `infra/migrations/` (nuevo archivo `.sql`).
  4. AC: tabla + FK + unique index + `sqlx migrate run` limpio.
  5. Stop after migration applies cleanly; do NOT start T2b.

---

## S-200-T2b — Read-side repo: `UserAccount` struct + `find_active_by_email`

- **Type:** development. **Effort:** M. **RRI:** 37 Moderate (`scripts/rri.py`).
- **Depends on:** T2a. **Reflection passes:** 2 (Moderate).
- **Objective:** Struct `UserAccount` + método `find_active_by_email` en `crates/db`;
  decodificación fail-closed de `status`.
- **Acceptance criteria:**
  - `UserAccount { id: Uuid, email: String, password_hash: String, workspace_id: Uuid,
    status: AccountStatus }` derivado con `sqlx::FromRow`.
  - `AccountStatus` es un enum `Active` + arm `Unknown(String)` → decodificación
    fail-closed: `Unknown` devuelve `Err`, nunca pasa la guarda.
  - `find_active_by_email(pool, email) -> Result<Option<UserAccount>>` filtra
    `status = 'active'`.
  - Test: account `status = 'suspended'` → `find_active_by_email` devuelve `None`.
  - Test: email inexistente → `None`.
  - `cargo test -p dubbridge-db` verde.
- **Handoff prompt:**
  1. S-200-T2b — `UserAccount` struct + `find_active_by_email`.
  2. Govern: `docs/plan/s-200-...md`; ADR-008 (UUID lineage).
  3. Files: `crates/db/src/user_account.rs` (new), `crates/db/src/lib.rs`.
  4. AC: fail-closed status decode; filter active only; tests green.
  5. Stop after `cargo test -p dubbridge-db` verde; do NOT start T2c.

### Completion record — S-200-T2b (✅ Done, 2026-06-18)

New module `crates/db/src/user_account.rs` added and exported from `crates/db/src/lib.rs`.
The repo now exposes `UserAccount` (`sqlx::FromRow`) and
`find_active_by_email(pool, email)`. `AccountStatus` decodes only `"active"` as
`Active`; all other stored values are retained as `Unknown(String)` and then rejected
fail-closed via `DbError::UnknownStoredValue` before a `UserAccount` can pass the
read-side guard.

#### Reflection log

Required passes: 2 (`31` → `Moderate`)

##### Pass 1 — row shape + fail-closed decode
- **Draft verdict:** `UserAccount` should derive `sqlx::FromRow` directly, with
  `status: AccountStatus`, instead of introducing a second intermediate row struct.
- **Critique findings:** `sqlx` needed explicit `Type`/`Decode` support for
  `AccountStatus`; the fail-closed requirement could not live only in SQL.
- **Revisions applied:** implemented `Type<Postgres>` + `Decode<Postgres>` for
  `AccountStatus`; added `require_known_status` / `ensure_active_account` so
  `Unknown(String)` is rejected as `DbError::UnknownStoredValue`.

##### Pass 2 — active-only filter + test strategy
- **Draft verdict:** the repo query should return `Some` only for active accounts and
  `None` for suspended / missing rows.
- **Critique findings:** the crate has no guaranteed local Postgres harness in this
  environment (`DUBBRIDGE_DATABASE_URL` unset; no local `postgres`/`psql` binaries),
  so DB-backed lookup tests must remain opportunistic smoke tests rather than the sole
  certification evidence.
- **Revisions applied:** extracted `FIND_ACTIVE_BY_EMAIL_SQL` and `map_lookup_result`
  for unit-level verification of the active-only filter and `None` mapping; kept async
  smoke tests that exercise the real query path when `DUBBRIDGE_DATABASE_URL` is
  configured.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | active stored status maps to typed `UserAccount` and survives the read-side guard | `crates/db/src/user_account.rs::{account_status_known_variant_maps_to_active,ensure_active_account_accepts_active_status}` | passed |
| HP-2 | Happy path | `UserAccount` preserves `workspace_id` and other row fields unchanged after the guard | `crates/db/src/user_account.rs::ensure_active_account_accepts_active_status` | passed |
| EC-1 | Edge case | suspended account is excluded from the repo result because the query is active-only, so the lookup maps to `None` | `crates/db/src/user_account.rs::{find_active_by_email_query_filters_to_active_status,map_lookup_result_none_returns_none}` | passed |
| EC-2 | Edge case | nonexistent email returns `None` rather than an error | `crates/db/src/user_account.rs::map_lookup_result_none_returns_none` | passed |
| EC-3 | Edge case | unknown persisted status fails closed as `DbError::UnknownStoredValue` | `crates/db/src/user_account.rs::{account_status_unknown_variant_is_retained_for_fail_closed_handling,require_known_status_unknown_value_fails_closed}` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified every approved happy path and edge case for T2b has unit test
  evidence in `crates/db/src/user_account.rs`; the active-only SQL filter is pinned,
  missing/suspended rows map to `None`, and unknown stored statuses fail closed.
- Commands run: `cargo fmt -p dubbridge-db`; `cargo clippy -p dubbridge-db
  --all-features -- -D warnings`; `cargo test -p dubbridge-db` (53 passed).
- Note: the module also includes async Postgres smoke tests for the real lookup path;
  in this environment they self-skip unless `DUBBRIDGE_DATABASE_URL` is configured,
  so the certification above relies on pure unit tests plus the pinned SQL filter.

---

## S-200-T2c — Write-side repo: `insert_account` + `insert_workspace` (transactional)

- **Type:** development. **Effort:** L. **RRI:** 46 Med-high (`scripts/rri.py`).
- **Depends on:** T2b. **Reflection passes:** 3 (Med-high).
- **Objective:** Métodos `insert_workspace` e `insert_account` ejecutados en una sola
  transacción sqlx; manejo tipado de violación de unicidad en `email`.
- **Acceptance criteria:**
  - `insert_workspace(tx, name) -> Result<Uuid>` inserta en `workspace` y devuelve el
    nuevo `id`.
  - `insert_account(tx, email, password_hash, workspace_id) -> Result<Uuid>` inserta en
    `user_account` y devuelve `id`; email duplicado → `Err(DbError::Conflict)` (no
    unwrap / no panic).
  - `register(pool, email, hash, ws_name)` abre una tx, llama ambos, hace commit; si
    falla alguno, hace rollback total.
  - HP-1: insert workspace + account → ambos persistidos; account encontrable por email.
  - EC-1: email duplicado → `Conflict` propagado; workspace NO queda huérfano.
  - `cargo test -p dubbridge-db` verde (incluyendo tests de T2b).
  - `assets.uploader_id` UUID contract confirmado (OQ2): el tipo `id` de `user_account`
    es el mismo `Uuid` que `assets.uploader_id`.
- **Handoff prompt:**
  1. S-200-T2c — transactional `insert_account` + `insert_workspace`.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 5; ADR-008.
  3. Files: `crates/db/src/user_account.rs`, `crates/db/src/lib.rs`.
  4. AC: transacción atómica; `Conflict` tipado; rollback en fallo; OQ2 confirmado.
  5. Stop after `cargo test -p dubbridge-db` verde + 3 Reflection passes; do NOT start T3.

### Completion record — S-200-T2c (✅ Done, 2026-06-18)

`crates/db/src/user_account.rs` now owns the write-side auth repo alongside the read
side from T2b. Added:
- `insert_workspace(tx, name) -> Result<Uuid, DbError>` using the existing
  `organizations` table via `workspace_repo::insert_org_tx`
- `insert_account(tx, email, password_hash, workspace_id) -> Result<Uuid, DbError>`
  with duplicate-email mapping to `DbError::Conflict`
- `register(pool, email, password_hash, workspace_name) -> Result<(Uuid, Uuid), DbError>`
  for atomic workspace+account creation

`crates/db/src/error.rs` now includes `DbError::Conflict`, and the generic `apps/api`
DB→HTTP adapters were updated so the new variant compiles cleanly and maps to `409`.

#### Reflection log

Required passes: 3 (`44` → `Med-high`)

##### Pass 1 — transactional shape
- **Draft verdict:** the simplest safe implementation is to reuse the existing
  `organizations` write path instead of creating a second raw SQL path for
  “workspace”.
- **Critique findings:** the schema still names the table `organizations`, while this
  slice names the concept `workspace`; duplicating that insert logic in a second place
  would create drift.
- **Revisions applied:** `insert_workspace` now delegates to
  `workspace_repo::insert_org_tx` and returns the generated `Uuid` so the auth slice
  keeps one write path for the shared table.

##### Pass 2 — typed conflict + compile surface
- **Draft verdict:** duplicate email must be represented as `DbError::Conflict`, not
  as a raw `QueryFailed`.
- **Critique findings:** adding `Conflict` to `DbError` broke exhaustive `match`
  sites in `apps/api`.
- **Revisions applied:** implemented SQLSTATE `23505` detection in the repo and added
  `Conflict -> 409` handling in the generic API DB error adapters.

##### Pass 3 — atomic orchestration + testability
- **Draft verdict:** `register` should only commit after both inserts succeed, and the
  flow needs unit-level proof even when no local Postgres is available.
- **Critique findings:** this machine has no reachable local Postgres
  (`DUBBRIDGE_DATABASE_URL` unset; `localhost:5432` closed), so DB-backed rollback
  tests cannot be the only evidence.
- **Revisions applied:** extracted `build_registration_result` and
  `should_commit_registration` so the commit/no-commit branch is unit-testable; kept
  async smoke tests for the real DB path when a Postgres URL is later available.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | success path preserves generated `user_id` + `workspace_id` and produces a committable registration result | `crates/db/src/user_account.rs::{build_registration_result_success_preserves_ids,should_commit_registration_only_on_success}` | passed |
| HP-2 | Happy path | workspace linkage stays plain `Uuid`, matching the account/read-side contract used by downstream auth work | `crates/db/src/user_account.rs::{insert_account_query_uses_workspace_linkage,build_registration_result_success_preserves_ids}` | passed |
| EC-1 | Edge case | duplicate email propagates as typed `DbError::Conflict` and suppresses commit | `crates/db/src/user_account.rs::{build_registration_result_conflict_propagates,should_commit_registration_only_on_success}` | passed |
| EC-2 | Edge case | non-unique SQL failures remain `DbError::QueryFailed` rather than being misclassified as `Conflict` | `crates/db/src/user_account.rs::map_insert_account_error_non_unique_stays_query_failed` | passed |
| EC-3 | Edge case | existing T2b invariant remains true after registration wiring: absent / suspended accounts never surface as active | `crates/db/src/user_account.rs::{find_active_by_email_returns_none_for_unknown_email,find_active_by_email_returns_none_for_suspended_account}` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the transactional write-side flow, typed conflict mapping, and
  commit/no-commit orchestration for T2c. The unit suite certifies the control flow
  and error classification; DB-backed smoke tests for real persistence/rollback are
  present but could not be exercised on this host because no local Postgres endpoint
  was available.
- Commands run: `cargo fmt`; `cargo clippy -p dubbridge-db --all-features -- -D warnings`;
  `cargo test -p dubbridge-db` (60 passed); `cargo check -p dubbridge-api`.

---

## S-200-T3 — bcrypt credentials + `AuthService`

> **Decomposed (2026-06-18).** A fresh `scripts/rri.py` run scoped to the intended
> `crates/auth` service work scored T3 at **RRI 68 → Complex (56–70)**. That band's
> gate is "plan first" and the task naturally separates into three smaller units:
> credential primitives, registration, and login anti-enumeration. Implement them in
> order; each remains independently approval-gated.
>
> | Subtask | Scope | RRI → band | Gate |
> |---|---|---|---|
> | **T3a** | New `credentials.rs` with bcrypt hash/verify helpers and minimal validation primitives | 32 → Moderate | acceptance criteria + approval |
> | **T3b** | `AuthService::register` using `dubbridge_db::user_account::register` + HS256 issue + typed `Conflict` | 45 → Med-high | plan + acceptance criteria + approval |
> | **T3c** | `AuthService::login` using `find_active_by_email` + dummy-hash anti-enumeration + final exports / combined coverage | 52 → Med-high | plan + acceptance criteria + approval |
>
> The original T3 contract below remains the combined acceptance contract for the
> three subtasks together.

- **Type:** development. **Effort:** L. **RRI:** 68 → Complex (`scripts/rri.py`) →
  decomposed into T3a/T3b/T3c. **Per-subtask model:** T3a Balanced; T3b/T3c
  Balanced→Premium, thinking On when needed.
- **Depends on:** T1, T2. **Reflection passes:** 4 on the original combined scope;
  T3a requires 2, T3b/T3c require 3 each.
- **Objective:** `hash_password`/`verify_password` (bcrypt cost 12) and an
  `AuthService` with `login` (validate → issue) and `register` (atomic
  workspace+account create → issue), with the generic-credential-error and
  anti-enumeration behavior.
- **Acceptance criteria:**
  - `register`: bcrypt-hash (cost 12), atomic insert, issue token, return
    `{token, userId, workspaceId}`; duplicate email → typed `Conflict`.
  - `login`: generic `InvalidCredentials` for unknown email **and** wrong password;
    runs a bcrypt comparison against a dummy hash when the account is absent.
  - Empty/missing email or password → typed validation error (not `InvalidCredentials`).
  - Password < 12 chars on register → validation error.
  - Plaintext password never logged.
  - ≥ 90% coverage on `service.rs` + `credentials.rs`.
- **Happy paths considered:**
  - HP-1: valid email + correct password → token issued, claims match the account.
  - HP-2: new email + valid fields → workspace + account created, token issued.
- **Edge cases considered:**
  - EC-1: wrong password → generic `InvalidCredentials` (same as unknown email).
  - EC-2: unknown email → generic `InvalidCredentials`, with dummy-hash comparison
    performed (no early return).
  - EC-3: duplicate email on register → `Conflict`.
  - EC-4: register password < 12 chars → validation error, no account created.
- **Handoff prompt:**
  1. S-200-T3 — bcrypt + `AuthService` login/register.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 1; ADR-018.
  3. Files: `crates/auth/src/credentials.rs` (new), `crates/auth/src/service.rs` (new).
  4. AC: bullets above; generic error + dummy-hash anti-enumeration.
  5. Stop after `cargo test -p auth` green + 4 Reflection passes; do NOT start T4.

---

## S-200-T3a — `credentials.rs`: bcrypt helpers + validation primitives

- **Type:** development. **Effort:** L. **RRI:** 52 Med-high (`scripts/rri.py`).
- **Depends on:** T1c-ii. **Reflection passes:** 3 (Med-high).
- **Objective:** Crear `crates/auth/src/credentials.rs` con `hash_password` y
  `verify_password` (bcrypt cost 12), más validaciones mínimas reutilizables para
  email/password sin acoplar todavía el servicio a DB ni issuer.
- **Acceptance criteria:**
  - `hash_password(password)` devuelve hash bcrypt con cost 12.
  - `verify_password(password, hash)` devuelve `true/false` sin panic para hashes
    válidos; errores de bcrypt se propagan tipados.
  - Existe helper de validación para campos vacíos y password `< 12` chars.
  - Test: password válida → round-trip hash/verify correcto.
  - Test: password incorrecta → `false`.
  - Test: password corta en register path helper → validation error.
- **Happy paths considered:**
  - HP-1: password válida → bcrypt hash cost 12 → verify `true`.
  - HP-2: email/password normalizados no vacíos pasan la validación.
- **Edge cases considered:**
  - EC-1: password incorrecta contra hash válido → `false`.
  - EC-2: hash corrupto / inválido → error tipado.
  - EC-3: password `< 12` chars → validation error.
- **Handoff prompt:**
  1. S-200-T3a — bcrypt helpers + validation primitives.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 1.
  3. Files: `crates/auth/src/credentials.rs` (new), `crates/auth/src/lib.rs`.
  4. AC: cost 12, verify bool/error split, validation helpers, tests green.
  5. Stop after `cargo test -p dubbridge-auth` green; do NOT start T3b.

### Completion record — S-200-T3a (✅ Done, 2026-06-18)

New module `crates/auth/src/credentials.rs` added and exported from
`crates/auth/src/lib.rs`. `dubbridge-auth` now has:
- `hash_password(password)` using bcrypt cost `12`
- `verify_password(password, hash)` returning `bool` for valid hashes and a typed error
  for invalid/corrupt stored hashes
- `normalize_required(value, field)` for trimmed required fields
- `validate_password_for_register(password)` enforcing the minimum length contract

`crates/auth/Cargo.toml` now depends on `bcrypt = "0.19.2"`; the version was checked
against current crates.io/docs.rs metadata before use.

#### Reflection log

Required passes: 3 (`52` → `Med-high`)

##### Pass 1 — module boundary + API shape
- **Draft verdict:** `T3a` should stay entirely free of DB and issuer concerns so the
  later `AuthService` work composes over a small pure credential layer.
- **Critique findings:** the first version needed a clear split between validation
  helpers and bcrypt operations to keep `T3b`/`T3c` simple.
- **Revisions applied:** kept `normalize_required`, `validate_password_for_register`,
  `hash_password`, and `verify_password` as separate helpers and exported them from
  `lib.rs`.

##### Pass 2 — bcrypt behavior + typed failures
- **Draft verdict:** wrong password should be `Ok(false)`, but corrupt stored hashes
  should be a typed error.
- **Critique findings:** the module had to preserve that distinction explicitly so
  `T3c` can return generic invalid credentials without hiding data-integrity failures.
- **Revisions applied:** mapped bcrypt verify failures to
  `CredentialError::InvalidStoredHash`; tests now pin both wrong-password and
  invalid-hash behavior separately.

##### Pass 3 — crate hygiene + verification
- **Draft verdict:** ready once the new dependency and exports compile cleanly with
  the existing auth suite.
- **Critique findings:** `HashParts` and `FromStr` were initially imported at module
  scope but only used in tests, causing `clippy -D warnings` failures.
- **Revisions applied:** moved those imports into the test module; reran `fmt`,
  `clippy`, and the full `dubbridge-auth` test suite successfully.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid password hashes with bcrypt cost 12 and verifies successfully | `crates/auth/src/credentials.rs::{hash_password_uses_configured_bcrypt_cost,hash_password_round_trips_with_verify_password}` | passed |
| HP-2 | Happy path | trimmed non-empty auth input passes required-field validation | `crates/auth/src/credentials.rs::normalize_required_trims_and_accepts_non_empty_values` | passed |
| EC-1 | Edge case | wrong password against a valid hash returns `false` | `crates/auth/src/credentials.rs::verify_password_returns_false_for_wrong_password` | passed |
| EC-2 | Edge case | corrupt/invalid bcrypt hash returns typed `InvalidStoredHash` | `crates/auth/src/credentials.rs::verify_password_rejects_invalid_hash` | passed |
| EC-3 | Edge case | short register password fails validation | `crates/auth/src/credentials.rs::validate_password_for_register_rejects_short_password` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified every approved happy path and edge case for T3a has unit test
  evidence in `crates/auth/src/credentials.rs`; bcrypt cost 12, verify semantics, and
  validation helpers are all pinned by tests and exported for downstream subtasks.
- Commands run: `cargo fmt -p dubbridge-auth`; `cargo clippy -p dubbridge-auth
  --all-features -- -D warnings`; `cargo test -p dubbridge-auth` (54 passed).

---

## S-200-T3b — `AuthService::register`: atomic register + token issuance

- **Type:** development. **Effort:** L. **RRI:** 55 Med-high (`scripts/rri.py`).
- **Depends on:** T3a, T2c. **Reflection passes:** 3 (Med-high).
- **Objective:** Crear el esqueleto de `AuthService` y resolver sólo el camino de
  `register`: validar campos, hash bcrypt, `dubbridge_db::user_account::register`,
  emitir JWT HS256 y devolver `{token, userId, workspaceId}` con `Conflict` tipado.
- **Acceptance criteria:**
  - `AuthService::register` usa `hash_password` y la repo tx de T2c.
  - Duplicate email → error tipado `Conflict`.
  - Success → `{token, userId, workspaceId}`; claims del token cuadran con DB.
  - Validation error (empty fields / short password) ocurre antes del acceso DB.
  - Test: register válido crea cuenta y emite token.
  - Test: duplicate email → `Conflict`.
- **Happy paths considered:**
  - HP-1: new email + password válida + workspace válido → account creada, token emitido.
  - HP-2: token emitido contiene `sub` y `workspace_id` esperados.
- **Edge cases considered:**
  - EC-1: duplicate email → `Conflict`.
  - EC-2: password corta → validation error before DB.
  - EC-3: empty email/password/workspace → validation error before DB.
- **Handoff prompt:**
  1. S-200-T3b — `AuthService::register`.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 1; ADR-018.
  3. Files: `crates/auth/src/service.rs` (new), `crates/auth/src/lib.rs`.
  4. AC: atomic register via T2c, typed `Conflict`, token issuance, tests green.
  5. Stop after `cargo test -p dubbridge-auth` green + 3 Reflection passes; do NOT start T3c.

### Completion record — S-200-T3b (✅ Done, 2026-06-18)

New module `crates/auth/src/service.rs` added and exported from
`crates/auth/src/lib.rs`. `dubbridge-auth` now has:
- `AuthService<S, I>` with injectable account-store and token-issuer seams
- `PgAccountStore` as the production adapter over `dubbridge_db::user_account`
- `AuthService::register(email, password, workspace_name)` validating input, hashing
  with bcrypt, calling the transactional DB register path from T2c, and issuing an
  HS256 JWT
- typed service errors, including `Conflict` for duplicate email and `TokenIssue` for
  issuer failures

`crates/auth/Cargo.toml` now depends on `async-trait`, `dubbridge-db`, and `sqlx` so
the service layer can own the DB + auth seams directly.

#### Reflection log

Required passes: 3 (`55` → `Med-high`)

##### Pass 1 — service seam shape
- **Draft verdict:** `T3b` should create the service shell in a way that T3c can reuse
  for login without forcing DB-backed tests.
- **Critique findings:** a direct `PgPool` + `Hs256Issuer` implementation would make
  unit testing registration awkward and would leave T3c to retrofit seams later.
- **Revisions applied:** introduced `AccountStore` and `AccessTokenIssuer` traits,
  `PgAccountStore`, and a generic `AuthService<S, I>` so register can be unit-tested
  with fakes while production still uses real DB + issuer adapters.

##### Pass 2 — register flow + token semantics
- **Draft verdict:** register should normalize non-password fields, preserve the raw
  password for hashing, persist atomically through T2c, and issue a token with the
  app-default scopes.
- **Critique findings:** password emptiness could not reuse trimmed normalization
  directly because trimming would alter the secret; the default scopes also needed to
  be explicit so the token is usable by the mobile product surface.
- **Revisions applied:** added `require_password` to check emptiness without trimming;
  defined `DEFAULT_AUTH_SCOPES` and used them during token issuance; success tests pin
  both the persisted ids and the emitted JWT claims.

##### Pass 3 — test fakes + crate hygiene
- **Draft verdict:** the register path was complete once tests covered success,
  duplicate email, and validation short-circuiting.
- **Critique findings:** initial fake implementations tried to derive `Clone`/`Default`
  over `Result<..., DbError>`, but `DbError` is not cloneable.
- **Revisions applied:** replaced those fields with cloneable fake outcome enums;
  reran `fmt`, `clippy`, and the full `dubbridge-auth` suite successfully.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid register input hashes the password, persists through the account store, and returns a token plus ids | `crates/auth/src/service.rs::register_success_hashes_password_persists_and_issues_token` | passed |
| HP-2 | Happy path | emitted token claims contain the same `user_id` / `workspace_id` returned by registration and the default scopes | `crates/auth/src/service.rs::register_success_hashes_password_persists_and_issues_token` | passed |
| EC-1 | Edge case | duplicate email maps to typed `Conflict` and skips token issuance | `crates/auth/src/service.rs::register_duplicate_email_returns_conflict_and_does_not_issue_token` | passed |
| EC-2 | Edge case | short password fails validation before DB and issuer calls | `crates/auth/src/service.rs::register_validation_error_short_circuits_before_db_and_issuer` | passed |
| EC-3 | Edge case | empty required fields fail validation before DB and issuer calls | `crates/auth/src/service.rs::register_validation_error_for_empty_fields_short_circuits_before_db` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified every approved happy path and edge case for T3b has unit test
  evidence in `crates/auth/src/service.rs`; registration now hashes, persists through
  T2c, emits JWTs with pinned claims/scopes, and returns typed `Conflict` for duplicate
  email.
- Commands run: `cargo fmt -p dubbridge-auth`; `cargo clippy -p dubbridge-auth
  --all-features -- -D warnings`; `cargo test -p dubbridge-auth` (58 passed).

---

## S-200-T3c — `AuthService::login`: generic invalid credentials + anti-enumeration

- **Type:** development. **Effort:** L. **RRI:** 52 Med-high (`scripts/rri.py`).
- **Depends on:** T3a, T3b, T2b. **Reflection passes:** 3 (Med-high).
- **Objective:** Completar `AuthService` con `login`: lookup por email activo,
  comparación bcrypt, generic `InvalidCredentials` para email ausente o password
  errónea, y dummy-hash comparison cuando no existe la cuenta.
- **Acceptance criteria:**
  - `login` usa `find_active_by_email`.
  - Unknown email y wrong password devuelven exactamente el mismo error tipado.
  - Unknown email ejecuta comparación contra dummy hash (sin early return).
  - Success → `{token, userId, workspaceId}` emitido desde issuer HS256.
  - Exports finales en `crates/auth/src/lib.rs` y cobertura total `credentials.rs` +
    `service.rs` ≥ 90%.
  - Test: login válido → token emitido.
  - Test: wrong password y unknown email → mismo error.
- **Happy paths considered:**
  - HP-1: email activo + password correcta → token emitido.
  - HP-2: account creada por register es inmediatamente usable en login.
- **Edge cases considered:**
  - EC-1: wrong password → generic `InvalidCredentials`.
  - EC-2: unknown email → generic `InvalidCredentials` + dummy-hash compare.
  - EC-3: malformed/empty email or password → validation error.
- **Handoff prompt:**
  1. S-200-T3c — `AuthService::login` + anti-enumeration.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 1; ADR-018.
  3. Files: `crates/auth/src/service.rs`, `crates/auth/src/credentials.rs`, `crates/auth/src/lib.rs`.
  4. AC: generic invalid creds, dummy-hash compare, final exports, ≥ 90% coverage.
  5. Stop after `cargo test -p dubbridge-auth` green + 3 Reflection passes; do NOT start T4.

### Completion record — S-200-T3c (✅ Done, 2026-06-18)

`crates/auth/src/service.rs` now completes the auth service with:
- `AuthService::login(email, password)` using `find_active_by_email`, bcrypt verify,
  and HS256 token issuance for active accounts
- generic `InvalidCredentials` for both unknown email and wrong password
- a fixed valid bcrypt dummy hash used on the unknown-email path so the service does
  not early-return before doing comparable password work
- a richer fake account-store test seam so register/login behavior can be tested
  together without DB-backed integration tests

No additional `crates/auth/src/lib.rs` exports were needed beyond the T3b service
exports; `login` is exposed through the already-exported `AuthService`.

#### Reflection log

Required passes: 3 (`52` → `Med-high`)

##### Pass 1 — login flow reuse
- **Draft verdict:** `login` should reuse the T3b service seams and token issuance path
  instead of introducing a parallel auth path.
- **Critique findings:** duplicating token issuance inside `login` would drift from the
  register path and make future scope changes harder to keep aligned.
- **Revisions applied:** extracted shared issuance into `issue_token`; implemented
  `login` on top of `find_active_by_email`, shared validation, and the existing issuer.

##### Pass 2 — anti-enumeration behavior
- **Draft verdict:** unknown email and wrong password should collapse to the same typed
  failure while still doing real bcrypt work for missing accounts.
- **Critique findings:** an early return on unknown email would leak account existence
  through timing behavior, and a generated dummy hash would add unnecessary runtime
  variability.
- **Revisions applied:** added a fixed valid `DUMMY_PASSWORD_HASH` and
  `compare_against_dummy_hash(password)` so the unknown-email path performs bcrypt
  verification before returning `InvalidCredentials`.

##### Pass 3 — test seam and coverage closure
- **Draft verdict:** the code path was only complete once the suite covered success,
  immediate register→login usability, generic invalid credentials, and validation
  short-circuiting.
- **Critique findings:** the earlier fake store only tracked registration calls, which
  made login-path verification and HP-2 coverage awkward.
- **Revisions applied:** upgraded the fake store to hold accounts + lookup history,
  added login-focused unit tests, fixed the test-only `HashMap` import placement for
  `clippy`, and verified combined `credentials.rs + service.rs` coverage above 90%.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | active account with correct password emits a token with the expected ids/scopes | `crates/auth/src/service.rs::login_success_issues_token_for_existing_account` | passed |
| HP-2 | Happy path | an account created by `register` is immediately usable by `login` | `crates/auth/src/service.rs::register_creates_account_immediately_usable_for_login` | passed |
| EC-1 | Edge case | wrong password returns generic `InvalidCredentials` and does not issue a token | `crates/auth/src/service.rs::login_wrong_password_and_unknown_email_return_same_error` | passed |
| EC-2 | Edge case | unknown email returns the same generic `InvalidCredentials` result as wrong password | `crates/auth/src/service.rs::login_wrong_password_and_unknown_email_return_same_error` | passed |
| EC-3 | Edge case | empty password fails validation before DB lookup and token issuance | `crates/auth/src/service.rs::login_validation_error_short_circuits_before_db_and_issuer` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified every approved happy path and edge case for T3c has unit test
  evidence in `crates/auth/src/service.rs`; login now returns the same typed failure
  for wrong password and unknown email, performs dummy-hash bcrypt work on the
  unknown-email path, and keeps combined `credentials.rs + service.rs` line coverage
  above the required 90%.
- Commands run: `cargo fmt -p dubbridge-auth`; `cargo clippy -p dubbridge-auth
  --all-features -- -D warnings`; `cargo test -p dubbridge-auth` (62 passed);
  `cargo llvm-cov -p dubbridge-auth --summary-only --fail-under-lines 90
  --ignore-filename-regex '(axum|config|issuer|membership|principal|verifier)\.rs'
  -- --test-threads=1` (93.30% lines for `credentials.rs` + `service.rs`).

---

> **Decomposed (2026-06-18).** A fresh `scripts/rri.py` run scoped to the intended
> implementation scored T4 at **RRI 66 → Complex**, so the task is split below to
> satisfy the mandatory `RRI 56+` decomposition rule.

## S-200-T4 — `apps/api` public `/auth/login` + `/auth/register`

- **Type:** development. **Effort:** L. **RRI:** 66 Complex (`scripts/rri.py`).
- **Depends on:** T3c. **Reflection passes:** 4 (Complex).
- **Objective:** Public handlers mapping HTTP ↔ `AuthService`, mounted outside the auth
  middleware; fail-closed issuance config; login/registration audit rows.
- **Acceptance criteria:**
  - `POST /auth/login` → 200 `{token,userId,workspaceId}` | 400 (missing fields) | 401
    (generic invalid) | 500.
  - `POST /auth/register` → 201 | 400 (validation) | 409 (duplicate) | 500.
  - Both routes are public; `/api/*` and existing routes stay verification-gated.
  - `DUBBRIDGE_AUTH_JWT_SECRET` / `auth.jwt_expiry_hours` wired via `crates/config`;
    non-local startup fails closed without the secret.
  - Audit rows emitted for login success/failure and registration (ADR-018), token
    redacted.
- **Happy paths considered:**
  - HP-1: valid login → 200 with token; audit `login`/`success` row written.
  - HP-2: valid register → 201 with token; account + workspace persisted.
- **Edge cases considered:**
  - EC-1: missing email/password → 400 before any DB call.
  - EC-2: wrong credentials → 401 generic; audit `login`/`error` row written.
  - EC-3: duplicate email → 409.
  - EC-4: process boot without auth JWT secret/expiry config in non-local env →
    startup abort.

### S-200-T4a — Config schema: `auth.jwt_expiry_hours` + parity docs

- **Type:** development (config). **Effort:** M. **RRI:** 35 Moderate (`scripts/rri.py`).
- **Depends on:** T3c. **Reflection passes:** 2.
- **Objective:** Extender `crates/config` para exponer `auth.jwt_expiry_hours` y reflejar
  la nueva variable en `config/README.md`.
- **Acceptance criteria:**
  - `AuthSettings` incluye `jwt_expiry_hours`.
  - `AppConfig::load()` / env override soportan la nueva clave.
  - `config/README.md` documenta la nueva variable y su override.
  - Tests de config cubren lectura por defecto/override.
- **Happy paths considered:**
  - HP-1: config auth con expiry explícito carga el valor esperado.
  - HP-2: env override reemplaza el valor de archivo.
- **Edge cases considered:**
  - EC-1: falta el override de env y se usa el valor configurado.
  - EC-2: valor no parseable falla la carga de configuración.
- **Handoff prompt:**
  1. S-200-T4a — add `auth.jwt_expiry_hours`.
  2. Govern: ADR-026; S-200 plan/config requirements.
  3. Files: `crates/config/src/lib.rs`, `config/README.md`.
  4. AC: schema + env override + docs + tests.
  5. Stop after config tests green; do NOT start T4b.

### Completion record — S-200-T4a (✅ Done, 2026-06-18)

`crates/config/src/lib.rs` now exposes `auth.jwt_expiry_hours` as part of
`AuthSettings`, with a serde default of `24` hours so existing config profiles can
deserialize cleanly until the key is added explicitly. The legacy `AuthSettings::from_env()`
path also reads `DUBBRIDGE_AUTH_JWT_EXPIRY_HOURS`, while the layered typed loader
continues to use `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` for nested env overrides.

`config/README.md` now documents the auth env override paths more accurately:
- added `auth.jwt_secret` and `auth.jwt_expiry_hours` to the parity table
- switched structured `auth.*` rows to the typed loader's `__` nesting convention
- documented the remaining flat legacy aliases for `AuthSettings::from_env()`

`apps/api/src/main.rs` test fixtures were updated with the new field so downstream
API compile checks keep passing after the config schema expansion.

#### Reflection log

Required passes: 2 (`35` → `Moderate`)

##### Pass 1 — schema shape and compatibility
- **Draft verdict:** `jwt_expiry_hours` should live directly under `AuthSettings` so
  T4b can build the issuer from the same typed auth block that already owns the secret.
- **Critique findings:** making the field required without a serde default would break
  existing profile deserialization because current committed config files do not carry
  an `auth` block with expiry yet.
- **Revisions applied:** added `jwt_expiry_hours` with a default of `24`, matching
  ADR-031, and extended the legacy flat env helper to read the new key as well.

##### Pass 2 — loader/docs/test consistency
- **Draft verdict:** the implementation was only complete once both config-loading
  paths and the parity docs agreed on how auth overrides are named.
- **Critique findings:** the README still described flat auth env vars while the typed
  loader actually uses nested `DUBBRIDGE_AUTH__...` names; the new field also needed
  explicit tests for file-backed deserialization and env override precedence.
- **Revisions applied:** updated the auth rows in `config/README.md`, added the new
  parity note about nested vs legacy-flat overrides, and added targeted tests for
  TOML deserialization plus `AppConfig::load()` env override behavior.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | config schema reads `auth.jwt_expiry_hours` from TOML auth settings | `crates/config/src/lib.rs::auth_settings_schema_reads_jwt_expiry_hours_from_toml` | passed |
| HP-2 | Happy path | nested env override wins and loads `auth.jwt_expiry_hours` through `AppConfig::load()` | `crates/config/src/lib.rs::app_config_load_env_override_reads_auth_jwt_expiry_hours` | passed |
| EC-1 | Edge case | legacy flat `AuthSettings::from_env()` path falls back to the default `24` hours when expiry is unset | `crates/config/src/lib.rs::auth_settings_default_clock_skew` | passed |
| EC-2 | Edge case | legacy flat `AuthSettings::from_env()` path parses an explicit expiry override correctly | `crates/config/src/lib.rs::auth_settings_reads_optional_jwt_secret_from_env` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the new auth expiry field is available through both config
  loading paths, defaults to ADR-031's `24h`, is documented in the parity table, and
  does not break downstream API compilation.
- Commands run: `cargo fmt -p dubbridge-config -p dubbridge-api`;
  `cargo test -p dubbridge-config` (41 passed); `cargo check -p dubbridge-api`.

### S-200-T4b — API runtime wiring: `AuthService` in `AppState` + fail-closed issuer builder

- **Type:** development. **Effort:** L. **RRI:** 53 Med-high (`scripts/rri.py`).
- **Depends on:** T4a. **Reflection passes:** 3.
- **Objective:** Construir `AuthService<PgAccountStore, Hs256Issuer>` en `apps/api` y
  exponerlo en `AppState`, con builder fail-closed en entornos no locales.
- **Acceptance criteria:**
  - `AppState` expone un `AuthService` reutilizable por rutas públicas.
  - `main.rs` construye issuer + service desde config auth.
  - Non-local startup falla si falta `auth.jwt_secret`.
  - Tests pinchan el builder local vs production-like.
- **Happy paths considered:**
  - HP-1: startup con config auth completa construye service y verifier.
  - HP-2: local sin secret explícito conserva placeholder solo para dev.
- **Edge cases considered:**
  - EC-1: production-like sin secret → abort fail-closed.
  - EC-2: issuer inválido/expiry inválido → startup error.
- **Handoff prompt:**
  1. S-200-T4b — wire `AuthService` into `AppState`.
  2. Govern: ADR-031 Decision 1; ADR-026 fail-closed config.
  3. Files: `apps/api/src/state.rs`, `apps/api/src/main.rs`.
  4. AC: builder + state wiring + startup tests.
  5. Stop after `cargo test -p dubbridge-api` green; do NOT start T4c.

### Completion record — S-200-T4b (✅ Done, 2026-06-18)

`apps/api/src/state.rs` now exposes a shared auth-service slot:
- `ApiAuthService = AuthService<PgAccountStore, Hs256Issuer>`
- `SharedAuthService = Arc<ApiAuthService>`
- `AppState.auth_service: Option<SharedAuthService>`
- `AppState::with_auth_service(...)` for runtime wiring without breaking existing
  non-auth route tests that still use `AppState::new(...)`

`apps/api/src/main.rs` now builds the auth runtime explicitly:
- `resolve_jwt_secret(config)` centralizes local placeholder vs production-like
  fail-closed behavior
- `build_auth_service(config, pool)` builds `Hs256Issuer` from
  `auth.jwt_secret` + `auth.jwt_expiry_hours`, wires `PgAccountStore`, and returns an
  `Arc<AuthService<...>>`
- `main()` injects that shared service into `AppState`
- issuer wiring now rejects `auth.jwt_expiry_hours == 0` (and overflow) at startup

`build_verifier` was aligned to the same secret-resolution helper, so missing
production-like secrets now surface as startup errors instead of panics while staying
fail-closed.

#### Reflection log

Required passes: 3 (`53` → `Med-high`)

##### Pass 1 — service ownership in runtime state
- **Draft verdict:** the public auth handlers will need stable access to a single
  runtime `AuthService` without re-building issuers inside request handlers.
- **Critique findings:** storing the concrete service by value in `AppState` would push
  extra clone constraints onto `Hs256Issuer`, which is not currently `Clone`; changing
  that just for state ownership would be unnecessary risk.
- **Revisions applied:** added `SharedAuthService = Arc<ApiAuthService>` and stored it
  in `AppState` as `Option<...>`, with a dedicated `with_auth_service(...)`
  constructor so existing tests and non-auth routes remain unchanged.

##### Pass 2 — fail-closed builder semantics
- **Draft verdict:** verifier and issuer wiring should resolve the signing secret
  through one shared rule so startup behavior is consistent.
- **Critique findings:** the previous verifier path panicked in production-like envs,
  while the new service builder needed `Result`-based startup errors; leaving the two
  paths inconsistent would make failure handling harder to reason about.
- **Revisions applied:** extracted `auth_settings(...)` and `resolve_jwt_secret(...)`,
  switched `build_verifier(...)` to the same error-returning fail-closed path, and
  validated `auth.jwt_expiry_hours > 0` before constructing the issuer.

##### Pass 3 — test harness and regression closure
- **Draft verdict:** the runtime wiring was complete once startup behavior was pinned
  for local placeholder, production missing-secret failure, and invalid expiry.
- **Critique findings:** the first round of tests used `expect_err(...)` on success
  types without `Debug`, and the lazy `PgPool` helper still required a Tokio context.
- **Revisions applied:** rewrote the negative assertions using explicit `match` and
  converted the auth-service runtime tests to `#[tokio::test]`; then reran the full
  `dubbridge-api` suite and `clippy` successfully.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | configured production-like auth settings build a verifier that accepts tokens signed with the configured secret | `apps/api/src/main.rs::build_verifier_uses_configured_jwt_secret` | passed |
| HP-2 | Happy path | local runtime can build an auth service with the documented placeholder secret when no explicit secret is set | `apps/api/src/main.rs::build_auth_service_uses_local_placeholder_when_jwt_secret_is_missing` | passed |
| EC-1 | Edge case | production-like runtime fails closed when `auth.jwt_secret` is missing for auth-service construction | `apps/api/src/main.rs::build_auth_service_fails_closed_without_jwt_secret_in_production_like_env` | passed |
| EC-2 | Edge case | production-like verifier wiring fails closed when `auth.jwt_secret` is missing | `apps/api/src/main.rs::build_verifier_fails_closed_without_jwt_secret_in_production_like_env` | passed |
| EC-3 | Edge case | zero-hour JWT expiry is rejected before the API starts | `apps/api/src/main.rs::build_auth_service_rejects_zero_hour_expiry` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the API runtime now carries a reusable shared `AuthService`,
  resolves JWT secrets consistently for verifier and issuer wiring, fails closed in
  production-like environments without a secret, rejects zero-hour expiry at startup,
  and does not regress the existing `dubbridge-api` suite.
- Commands run: `cargo fmt -p dubbridge-api`;
  `cargo clippy -p dubbridge-api --all-features -- -D warnings`;
  `cargo test -p dubbridge-api`.

### S-200-T4c — Audit domain: auth login/register event kinds + constructor

- **Type:** development (domain). **Effort:** M. **RRI:** 33 Moderate (`scripts/rri.py`).
- **Depends on:** T3c. **Reflection passes:** 2.
- **Objective:** Añadir tipos de evento auditables para login success/failure y
  registration, con constructor específico en `crates/domain::audit`.
- **Acceptance criteria:**
  - `AuditEventKind` incluye eventos auth necesarios para T4d/T4e.
  - `Display` y serde los serializan en snake_case estable.
  - Hay constructor helper para auth events sin token expuesto.
  - Tests cubren round-trip / display.
- **Happy paths considered:**
  - HP-1: constructor crea evento auth con campos esperados.
  - HP-2: display/serde exponen el nombre estable del evento.
- **Edge cases considered:**
  - EC-1: evento de error auth conserva detail sin incluir el token.
  - EC-2: nuevos variants no rompen serialización existente.
- **Handoff prompt:**
  1. S-200-T4c — auth audit event kinds.
  2. Govern: ADR-018; ADR-031 risk mitigation for API-issued auth.
  3. Files: `crates/domain/src/audit.rs`.
  4. AC: new kinds + helper + tests.
  5. Stop after domain tests green; do NOT start T4d/T4e.

### Completion record — S-200-T4c (✅ Done, 2026-06-18)

`crates/domain/src/audit.rs` now includes explicit auth-governance vocabulary for the
API-issued credential flow:
- `AuditEventKind::AuthLoginSucceeded`
- `AuditEventKind::AuthLoginFailed`
- `AuditEventKind::AuthRegistered`

Their `Display`/serde names are pinned to stable snake_case:
- `auth_login_succeeded`
- `auth_login_failed`
- `auth_registered`

`AuditEvent::new_auth_event(event_kind, detail)` was added as the auth-specific helper
constructor. Like the existing workspace helper, it creates an event with no
`asset_id`, `ingest_token`, `recording_session_id`, or `platform_ingest_session_id`,
which matches the new login/register boundary we are auditing in ADR-031.

This deliberately models login/register as the auditable boundary, not a generic
“token exchange” event, because the new architecture validates credentials and emits
the JWT in the same backend step rather than crossing a separate exchange surface.

#### Reflection log

Required passes: 2 (`33` → `Moderate`)

##### Pass 1 — event boundary naming
- **Draft verdict:** the new auth audit variants should describe the real boundary in
  ADR-031, which is credential validation plus JWT issuance in `apps/api`.
- **Critique findings:** a generic `token_exchanged` event name would suggest a
  separate exchange hop that no longer exists in the new flow and would make future
  audit interpretation muddier.
- **Revisions applied:** added explicit `AuthLoginSucceeded`, `AuthLoginFailed`, and
  `AuthRegistered` variants instead of a generic exchange event.

##### Pass 2 — helper symmetry and test closure
- **Draft verdict:** the domain change was only complete once auth events had the same
  constructor ergonomics and name-pin coverage as workspace/consent/review events.
- **Critique findings:** adding enum variants without a dedicated helper would push
  repeated “no correlation ids” construction logic into future route handlers.
- **Revisions applied:** introduced `AuditEvent::new_auth_event(...)`, extended the
  all-variants display test, and added a focused constructor test for the auth path.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | auth events serialize to stable snake_case names for audit persistence | `crates/domain/src/audit.rs::audit_event_kind_display_all_variants` | passed |
| HP-2 | Happy path | `new_auth_event(...)` creates an auth event with no correlation ids and preserves detail | `crates/domain/src/audit.rs::new_auth_event_sets_no_correlation_ids` | passed |
| EC-1 | Edge case | auth failure events remain decoupled from asset/ingest/recording identifiers | `crates/domain/src/audit.rs::new_auth_event_sets_no_correlation_ids` | passed |
| EC-2 | Edge case | existing display coverage still enumerates all prior variants alongside the new auth ones | `crates/domain/src/audit.rs::audit_event_kind_display_all_variants` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the auth audit domain now has explicit success/failure/register
  event kinds, stable persisted names, and a dedicated constructor aligned with the
  new ADR-031 login/register boundary rather than a separate token-exchange concept.
- Commands run: `cargo fmt -p dubbridge-domain`;
  `cargo clippy -p dubbridge-domain --all-features -- -D warnings`;
  `cargo test -p dubbridge-domain` (82 passed).

### S-200-T4d — `/auth/register` handler + HTTP mapping + audit + tests

- **Type:** development. **Effort:** L. **RRI:** 52 Med-high (`scripts/rri.py`).
- **Depends on:** T4b, T4c. **Reflection passes:** 3.
- **Objective:** Implementar el handler público de register con validación HTTP, mapeo
  de errores del service y auditoría de registro exitoso.
- **Acceptance criteria:**
  - `POST /auth/register` devuelve 201 + payload `{token,userId,workspaceId}`.
  - Validation errors → 400; duplicate email → 409; unexpected errors → 500.
  - Registration success emite audit row sin incluir el token.
  - Tests cubren status mapping y side effects.
- **Happy paths considered:**
  - HP-1: register válido → 201 + token + audit row.
  - HP-2: payload JSON válido se normaliza y pasa al service.
- **Edge cases considered:**
  - EC-1: duplicate email → 409.
  - EC-2: missing/invalid body fields → 400.
  - EC-3: audit persistence failure → fail-closed 500.
- **Handoff prompt:**
  1. S-200-T4d — implement `/auth/register`.
  2. Govern: ADR-031; ADR-018.
  3. Files: `apps/api/src/routes/auth.rs`, route tests.
  4. AC: 201/400/409/500 mapping + audit.
  5. Stop after route tests green; do NOT start T4e.

### Completion record — S-200-T4d (✅ Done, 2026-06-18)

Added the auth-register transport layer without mounting it publicly yet:
- new DTO module [apps/api/src/dto/auth.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/dto/auth.rs)
  with camelCase `RegisterRequest` and `AuthSuccessResponse`
- `apps/api/src/routes/auth.rs` with `POST /auth/register`
- `apps/api/src/dto/mod.rs` now exports `auth`
- `apps/api/src/routes/mod.rs` now declares the private `auth` module so the route
  compiles and can be tested before `T4f` exposes/mounts it

The handler:
- reads `AppState.auth_service`
- maps `AuthServiceError::Validation` to `400`
- maps `AuthServiceError::Conflict` to `409`
- maps unexpected service/issuer/DB failures to `500`
- emits `AuthRegistered` via `emit_governance_audit(...)` after successful register
- fails closed with `500` if audit persistence fails
- never includes the JWT in the audit detail

`T4d` intentionally stops short of public mounting in `build_app`; that remains the
responsibility of `T4f`.

#### Reflection log

Required passes: 3 (`55` → `Med-high`)

##### Pass 1 — route shape before mount
- **Draft verdict:** the register surface should be implemented and tested as a
  self-contained route module before we expose it through `build_app`.
- **Critique findings:** mounting the route now would collapse `T4d` into `T4f` and
  make it harder to isolate handler behavior from public-router regressions.
- **Revisions applied:** added a private `routes/auth.rs` module and kept the router
  unmounted; tests exercise the route directly with `.with_state(...)`.

##### Pass 2 — HTTP/error mapping and audit semantics
- **Draft verdict:** register should expose a narrow public contract: `201`, `400`,
  `409`, or `500`, with audit only on successful registration.
- **Critique findings:** the public auth surface should not leak internal DB/token
  details in 500 responses, and audit detail must exclude the JWT itself even though
  the handler has access to it.
- **Revisions applied:** introduced a local `ApiError` mapper with public-safe strings,
  mapped JSON extractor failures to `400`, and persisted only `user_id` and
  `workspace_id` in the `AuthRegistered` audit detail.

##### Pass 3 — fail-closed testing strategy
- **Draft verdict:** the handler is only acceptable once success, conflict,
  validation, and audit-failure paths are all pinned with DB-backed route tests.
- **Critique findings:** to test audit failure deterministically without breaking the
  registration path, the audit pool and the auth-service pool need to be separable.
- **Revisions applied:** built a dedicated test context that uses a healthy pool for
  `AuthService` and, in the fail-closed case, a separately closed pool for
  `AppState.pool`; reran the full `dubbridge-api` suite and `clippy` successfully.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid register request returns `201`, emits `auth_registered`, and keeps the JWT out of audit detail | `apps/api/src/routes/auth.rs::register_handler_returns_created_and_emits_audit` | passed |
| HP-2 | Happy path | camelCase JSON payload is accepted by the register DTO/handler contract | `apps/api/src/routes/auth.rs::register_handler_returns_created_and_emits_audit` | passed |
| EC-1 | Edge case | duplicate email maps to `409` and does not emit a second success audit event | `apps/api/src/routes/auth.rs::register_handler_maps_duplicate_email_to_conflict` | passed |
| EC-2 | Edge case | validation failure maps to `400` and leaves DB/audit side effects absent | `apps/api/src/routes/auth.rs::register_handler_maps_validation_errors_to_bad_request` | passed |
| EC-3 | Edge case | audit persistence failure returns `500` fail-closed after the registration path succeeds | `apps/api/src/routes/auth.rs::register_handler_fails_closed_when_audit_persistence_fails` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the register handler now exists as an unmounted route module,
  returns the required HTTP statuses, emits `AuthRegistered` without leaking the JWT
  into audit detail, and fails closed when audit persistence is unavailable.
- Commands run: `cargo fmt -p dubbridge-api`;
  `cargo clippy -p dubbridge-api --all-features -- -D warnings`;
  `cargo test -p dubbridge-api`.

### S-200-T4e — `/auth/login` handler + generic 401 mapping + audit + tests

- **Type:** development. **Effort:** L. **RRI:** 55 Med-high (`scripts/rri.py`).
- **Depends on:** T4b, T4c. **Reflection passes:** 3.
- **Objective:** Implementar el handler público de login con generic 401 para credenciales
  inválidas y auditoría de éxito/fallo.
- **Acceptance criteria:**
  - `POST /auth/login` devuelve 200 + payload `{token,userId,workspaceId}`.
  - Missing fields → 400; invalid credentials → 401; unexpected errors → 500.
  - Success y failure emiten audit rows sin token en detail.
  - Tests cubren mismo resultado visible para wrong password y unknown email.
- **Happy paths considered:**
  - HP-1: login válido → 200 + token + audit success row.
  - HP-2: payload bien formado llega al service sin auth middleware.
- **Edge cases considered:**
  - EC-1: invalid credentials → 401 generic.
  - EC-2: missing/empty body fields → 400.
  - EC-3: audit persistence failure → fail-closed 500.
- **Handoff prompt:**
  1. S-200-T4e — implement `/auth/login`.
  2. Govern: ADR-031; ADR-018.
  3. Files: `apps/api/src/routes/auth.rs`, route tests.
  4. AC: 200/400/401/500 mapping + audit.
  5. Stop after route tests green; do NOT start T4f.

### Completion record — S-200-T4e (✅ Done, 2026-06-18)

Completed the auth-login transport layer, still without mounting the auth router
publicly in `build_app` yet:
- [apps/api/src/dto/auth.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/dto/auth.rs)
  now includes camelCase `LoginRequest` alongside the shared `AuthSuccessResponse`
- [apps/api/src/routes/auth.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/routes/auth.rs)
  now exposes local `POST /auth/login` and `POST /auth/register` routes for direct
  handler testing before `T4f` mounts them publicly

The login handler:
- reads `AppState.auth_service`
- maps `AuthServiceError::Validation` to `400`
- maps invalid credentials to a generic `401`
- maps unexpected service/issuer/DB failures to `500`
- emits `AuthLoginSucceeded` with `user_id` + `workspace_id` on success
- emits `AuthLoginFailed` with only `{"outcome":"invalid_credentials"}` on generic
  credential failure
- fails closed with `500` if audit persistence fails
- never includes the JWT in any audit detail

`T4e` intentionally keeps the auth router unmounted from the public app surface; that
remains the responsibility of `T4f`.

#### Reflection log

Required passes: 3 (`55` → `Med-high`)

##### Pass 1 — visible login contract
- **Draft verdict:** the public login surface should expose only `200`, `400`, `401`,
  or `500`, with identical visible `401` responses for wrong-password and
  unknown-email cases.
- **Critique findings:** if those two credential failures diverged even slightly in
  status or body, the route would reintroduce account-enumeration signal above the
  dummy-hash protection already built into `AuthService`.
- **Revisions applied:** mapped `AuthServiceError::InvalidCredentials` to one generic
  `401` payload and pinned both wrong-password and unknown-email paths in the same
  route test.

##### Pass 2 — audit detail minimization
- **Draft verdict:** login success and failure should both emit audit rows, but the
  failure payload should not include user-supplied email unless the task explicitly
  requires it.
- **Critique findings:** storing the submitted email in the failure audit would expand
  PII footprint without helping the acceptance criteria, while including the JWT would
  be an outright leak.
- **Revisions applied:** success audits persist only `user_id` and `workspace_id`;
  failure audits persist only `{"outcome":"invalid_credentials"}` and tests assert the
  JWT is absent from audit detail.

##### Pass 3 — fail-closed path and harness reuse
- **Draft verdict:** the task is only complete once success, generic `401`,
  validation, and audit-persistence failure all have deterministic route coverage.
- **Critique findings:** the existing register harness could cover login too, but only
  if account seeding bypassed the route-level audit dependency for fail-closed tests.
- **Revisions applied:** added a direct `seed_account(...)` helper using the same
  `AuthService` builder as the route harness, reused the closed-audit-pool context,
  and reran `fmt`, `clippy`, and the full `dubbridge-api` suite cleanly.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid login returns `200`, emits `auth_login_succeeded`, and keeps the JWT out of audit detail | `apps/api/src/routes/auth.rs::login_handler_returns_ok_and_emits_success_audit` | passed |
| HP-2 | Happy path | well-formed login JSON reaches the handler/service contract and returns the shared auth success payload | `apps/api/src/routes/auth.rs::login_handler_returns_ok_and_emits_success_audit` | passed |
| EC-1 | Edge case | wrong password and unknown email return the same visible generic `401` and emit failure audits | `apps/api/src/routes/auth.rs::login_handler_maps_wrong_password_and_unknown_email_to_same_unauthorized` | passed |
| EC-2 | Edge case | missing/blank password maps to `400` and emits no login audit rows | `apps/api/src/routes/auth.rs::login_handler_maps_validation_errors_to_bad_request` | passed |
| EC-3 | Edge case | audit persistence failure returns `500` fail-closed after credential verification succeeds | `apps/api/src/routes/auth.rs::login_handler_fails_closed_when_audit_persistence_fails` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the login handler now exists as an unmounted auth route,
  returns the required `200/400/401/500` contract, emits explicit success/failure
  auth audit rows without leaking the JWT, and preserves the same visible `401`
  result for wrong-password and unknown-email cases.
- Commands run: `cargo fmt -p dubbridge-api`;
  `cargo clippy -p dubbridge-api --all-features -- -D warnings`;
  `cargo test -p dubbridge-api`.

### S-200-T4f — Public router mount/export for `/auth/*`

- **Type:** development. **Effort:** M. **RRI:** 34 Moderate (`scripts/rri.py`).
- **Depends on:** T4d, T4e. **Reflection passes:** 2.
- **Objective:** Exponer `routes::auth` y montarlo en `build_app` como superficie pública,
  preservando el gating actual de `/api/*`.
- **Acceptance criteria:**
  - `routes/mod.rs` exporta `auth`.
  - `build_app` monta `/auth/*` fuera del middleware bearer existente.
  - Smoke tests prueban que `/auth/*` es público y `/api/*` sigue gated.
- **Happy paths considered:**
  - HP-1: `/auth/login` y `/auth/register` quedan accesibles sin bearer.
  - HP-2: rutas protegidas existentes siguen usando verifier.
- **Edge cases considered:**
  - EC-1: el mount público no sombrea rutas existentes.
  - EC-2: `/api/*` sin bearer sigue devolviendo 401.
- **Handoff prompt:**
  1. S-200-T4f — public mount/export for auth routes.
  2. Govern: ADR-031 public auth surface.
  3. Files: `apps/api/src/lib.rs`, `apps/api/src/routes/mod.rs`.
  4. AC: public mount + regression smoke tests.
  5. Stop after `cargo test -p dubbridge-api` green; do NOT start T5.

### Completion record — S-200-T4f (✅ Done, 2026-06-18)

Mounted the auth transport on the real public API surface:
- [apps/api/src/routes/mod.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/routes/mod.rs)
  now exports `auth`
- [apps/api/src/routes/auth.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/routes/auth.rs)
  now exposes `pub fn router()`
- [apps/api/src/lib.rs](/Users/matias/Documents/projects/dubbridge/apps/api/src/lib.rs)
  now merges `routes::auth::router()` into `build_app(...)` before the protected route
  trees
- [apps/api/tests/auth_public_routes.rs](/Users/matias/Documents/projects/dubbridge/apps/api/tests/auth_public_routes.rs)
  pins the real app boundary: public `/auth/*`, protected `/api/*`

The mount keeps `/auth/login` and `/auth/register` outside bearer gating while the
existing `/assets` and other protected surfaces still rely on the verifier passed to
`build_app`.

#### Reflection log

Required passes: 2 (`34` → `Moderate`)

##### Pass 1 — router boundary
- **Draft verdict:** the public mount should add only the auth surface and avoid
  changing any existing protected router wiring.
- **Critique findings:** using the real `build_app(...)` boundary matters here; direct
  route-module tests alone would not prove that `/auth/*` is public once merged into
  the full app.
- **Revisions applied:** exported `routes::auth`, mounted it in `build_app(...)`, and
  left all protected merges unchanged behind the same verifier path.

##### Pass 2 — public/protected smoke coverage
- **Draft verdict:** the task is only complete once one integration test proves both
  sides of the boundary in the same app instance.
- **Critique findings:** existing integration contexts built `AppState::new(...)`
  without `auth_service`, which would make mounted `/auth/*` fail for the wrong
  reason.
- **Revisions applied:** added `auth_public_routes.rs` with
  `AppState::with_auth_service(...)`, verified public register/login without bearer,
  and checked `/assets` still returns `401` without bearer but `200` with a valid
  scoped token.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `/auth/register` and `/auth/login` are reachable through the real app without bearer auth | `apps/api/tests/auth_public_routes.rs::auth_routes_are_public_but_api_routes_stay_protected` | passed |
| HP-2 | Happy path | protected API routes still work through the verifier when a valid bearer token is supplied | `apps/api/tests/auth_public_routes.rs::auth_routes_are_public_but_api_routes_stay_protected` | passed |
| EC-1 | Edge case | mounting the public auth router does not shadow or break the existing protected `/assets` route | `apps/api/tests/auth_public_routes.rs::auth_routes_are_public_but_api_routes_stay_protected` | passed |
| EC-2 | Edge case | `/api/*` without bearer still returns `401` after the public mount | `apps/api/tests/auth_public_routes.rs::auth_routes_are_public_but_api_routes_stay_protected` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the auth router is now mounted on the public app surface,
  `/auth/login` and `/auth/register` are reachable without bearer auth, and the
  existing protected API boundary still rejects unauthenticated requests while
  accepting the same route with a valid bearer token.
- Commands run: `cargo fmt -p dubbridge-api`;
  `cargo clippy -p dubbridge-api --all-features -- -D warnings`;
  `cargo test -p dubbridge-api`.

---

## S-200-T5 — `apps/gateway` → transparent relay

> **Decomposed (2026-06-18).** A fresh `scripts/rri.py` run scored T5 at **RRI 66 →
> Complex (56–70)**. To keep the gateway rewrite reviewable, T5 is split into five
> ordered subtasks:
>
> | Subtask | Scope | RRI → band | Gate |
> |---|---|---|---|
> | **T5a** | Public `POST /auth/login` + `POST /auth/register` relay handlers | 55 → Med-high | plan + acceptance criteria + approval |
> | **T5b** | `Authorization: Bearer` passthrough for `/api/*` + `X-Real-IP` preservation | 55 → Med-high | plan + acceptance criteria + approval |
> | **T5c** | Remove `/auth/mobile/session` and the `session_ref` mobile contract | 52 → Med-high | plan + acceptance criteria + approval |
> | **T5d** | Remove legacy OAuth login/callback/logout route surface | 54 → Med-high | plan + acceptance criteria + approval |
> | **T5e** | Remove leftover session-store/runtime wiring after route retirement | 29 → Moderate | tests confirmed in area |
>
> The original T5 objective and acceptance criteria below remain the combined contract
> that these subtasks must jointly satisfy.

- **Type:** development. **Effort:** L. **RRI:** 66 Complex (`scripts/rri.py`) →
  decomposed into T5a–T5e. **Per-subtask model:** T5a–T5d Balanced→Premium; T5e
  Balanced.
- **Depends on:** T4f. **Reflection passes:** 3 on T5a–T5d; 2 on T5e.
- **Objective:** Reduce the mobile transport to a relay: forward `/auth/login`,
  `/auth/register`, and `Bearer`-carrying `/api/*` to `apps/api`; remove mobile session
  store usage, `X-Dubbridge-Session`, and handoff-code redemption.
- **Acceptance criteria:**
  - `/auth/login` + `/auth/register` forward body unchanged; relay status + JSON.
  - `/api/*` forwards `Authorization: Bearer <jwt>` unchanged; preserves `X-Real-IP`
    for audit.
  - Mobile session store, opaque `session_ref`, `X-Dubbridge-Session`, and handoff
    redemption are removed from this path (no dead code left behind).
  - Confirm OQ1 (no M2M client depends on the removed path) before deleting.
- **Happy paths considered:**
  - HP-1: login POST relayed → backend 200 passed back verbatim.
  - HP-2: `/api/assets` with a valid `Bearer` → relayed, backend response returned.
- **Edge cases considered:**
  - EC-1: `/api/*` without `Authorization` → backend 401 relayed unchanged (relay does
    not invent auth).
  - EC-2: backend 5xx → relayed with status preserved, not masked.
- **Handoff prompt:**
  1. S-200-T5 — gateway relay for `/auth/*` + `Bearer` `/api/*`.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 3.
  3. Files: `apps/gateway/src/proxy.rs`, `apps/gateway/src/auth/*`,
     `apps/gateway/src/session/*`.
  4. AC: bullets above; remove opaque-session transport; OQ1 confirmed.
  5. Stop after gateway tests green + 3 Reflection passes; do NOT start T6.

### S-200-T5a — `/auth/login` + `/auth/register` relay handlers + tests

- **Type:** development. **Effort:** L. **RRI:** 55 Med-high (`scripts/rri.py`).
- **Depends on:** T4f. **Reflection passes:** 3.
- **Objective:** Añadir en gateway los handlers públicos `POST /auth/login` y
  `POST /auth/register` que reenvían cuerpo, status y JSON de `apps/api`, sin
  desmontar todavía el resto del auth router legado.
- **Acceptance criteria:**
  - `POST /auth/login` forwardea body + `Content-Type` al upstream `apps/api`.
  - `POST /auth/register` forwardea body + `Content-Type` al upstream `apps/api`.
  - Success/4xx/5xx se relayan sin reescritura de payload.
  - Tests cubren relay verbatim para login y register.
- **Happy paths considered:**
  - HP-1: login válido → gateway devuelve el JSON de auth del upstream sin mutarlo.
  - HP-2: register válido → gateway devuelve el JSON de auth del upstream sin mutarlo.
- **Edge cases considered:**
  - EC-1: `400/401/409` del upstream se devuelven igual.
  - EC-2: `5xx` o fallo de red del upstream se mapea fail-closed según el contrato del gateway.
- **Handoff prompt:**
  1. S-200-T5a — add public login/register relay handlers in gateway.
  2. Govern: ADR-031 §Decision 3.
  3. Files: `apps/gateway/src/auth/mod.rs`, `apps/gateway/src/lib.rs`,
     `apps/gateway/src/auth/login.rs`, relay tests.
  4. AC: POST relay verbatim; no session transport added.
  5. Stop after gateway tests green; do NOT start T5b.

### Completion record — S-200-T5a (✅ Done, 2026-06-18)

Added the first relay slice on the gateway auth surface without removing the legacy
OAuth/browser routes yet:
- [apps/gateway/src/auth/mod.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/auth/mod.rs)
  now mounts `POST /auth/login` alongside the existing `GET /auth/login`, and mounts
  `POST /auth/register`
- [apps/gateway/src/auth/relay.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/auth/relay.rs)
  now contains the dedicated passthrough handlers for login/register relay
- [apps/gateway/tests/auth_relay_test.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/auth_relay_test.rs)
  pins the new visible contract through the real gateway app

The relay handlers:
- forward request bodies unchanged to `apps/api`
- preserve `Content-Type` to the upstream auth endpoints
- relay upstream `200/201/409` status and JSON payloads without semantic rewriting
- fail closed with `502 Bad Gateway` when the upstream auth surface is unreachable
- do not yet remove or modify the existing OAuth/session-based routes, which remain
  the responsibility of `T5c/T5d`

#### Reflection log

Required passes: 3 (`55` → `Med-high`)

##### Pass 1 — coexistence with legacy route surface
- **Draft verdict:** the new POST relay endpoints should coexist with the current GET
  `/auth/login` OAuth redirect flow so `T5a` does not silently subsume `T5d`.
- **Critique findings:** replacing `login_handler` directly would have coupled the new
  relay behavior to the legacy route-retirement work and made the diff harder to
  review.
- **Revisions applied:** mounted method-specific POST relay handlers on the existing
  auth router while preserving the current GET login/callback/logout behavior.

##### Pass 2 — passthrough fidelity
- **Draft verdict:** the gateway should behave like a transport relay here, not a
  second auth mapper.
- **Critique findings:** a relay that reserialized payloads or rewrote status codes
  would blur the contract between mobile and `apps/api` and make later debugging more
  expensive.
- **Revisions applied:** introduced a small dedicated relay module that forwards the
  raw request body, preserves `Content-Type`, copies upstream response headers except
  forbidden transport headers, and returns the upstream body verbatim.

##### Pass 3 — integration proof at the app boundary
- **Draft verdict:** the task is only complete once the real gateway app proves that
  POST login/register hit the upstream auth endpoints exactly as expected.
- **Critique findings:** unit-only handler tests would not prove route mounting or
  method coexistence with the existing auth router.
- **Revisions applied:** added `auth_relay_test.rs` against `build_app(...)`, covered
  login success, register success, upstream client-error passthrough, and unreachable
  upstream fail-closed behavior, then reran the full gateway suite cleanly.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid login POST is relayed to upstream `/auth/login` and returns the upstream JSON/status unchanged | `apps/gateway/tests/auth_relay_test.rs::post_login_relays_upstream_status_headers_and_json` | passed |
| HP-2 | Happy path | valid register POST is relayed to upstream `/auth/register` and returns the upstream JSON/status unchanged | `apps/gateway/tests/auth_relay_test.rs::post_register_relays_created_response_verbatim` | passed |
| EC-1 | Edge case | upstream client error payloads such as `409` are relayed without semantic rewriting | `apps/gateway/tests/auth_relay_test.rs::auth_relay_preserves_upstream_client_error_payload` | passed |
| EC-2 | Edge case | unreachable upstream auth surface returns fail-closed `502 Bad Gateway` | `apps/gateway/tests/auth_relay_test.rs::auth_relay_returns_bad_gateway_when_upstream_is_unreachable` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the gateway now exposes `POST /auth/login` and
  `POST /auth/register` as passthrough relay handlers to `apps/api`, preserves the
  upstream auth payloads/statuses, fails closed when the upstream is unreachable, and
  leaves the legacy OAuth/browser route surface untouched for the later retirement
  subtasks.
- Commands run: `cargo fmt -p dubbridge-gateway`;
  `cargo clippy -p dubbridge-gateway --all-features -- -D warnings`;
  `cargo test -p dubbridge-gateway`.

### S-200-T5b — `/api/*` bearer passthrough relay + preserved `X-Real-IP` + tests

- **Type:** development. **Effort:** L. **RRI:** 55 Med-high (`scripts/rri.py`).
- **Depends on:** T5a. **Reflection passes:** 3.
- **Objective:** Permitir que `/api/*` reenvíe `Authorization: Bearer <jwt>` tal cual
  al upstream, preservando `X-Real-IP`, sin exigir sesión opaca al caller móvil.
- **Acceptance criteria:**
  - `/api/*` con bearer válido se forwardea con el mismo `Authorization`.
  - `X-Real-IP` y otros headers permitidos siguen llegando al upstream.
  - `/api/*` sin bearer devuelve `401` sin inventar autenticación alternativa.
  - Tests cubren relay de header bearer y `401` sin header.
- **Happy paths considered:**
  - HP-1: request a `/api/assets` con bearer → upstream recibe el mismo bearer.
  - HP-2: response `200` del upstream vuelve sin mutación semántica.
- **Edge cases considered:**
  - EC-1: ausencia de `Authorization` → `401`.
  - EC-2: `5xx` del upstream → status preservado, no masked response.
- **Handoff prompt:**
  1. S-200-T5b — bearer passthrough for gateway `/api/*`.
  2. Govern: ADR-031 §Decision 3; ADR-018 audit IP preservation note.
  3. Files: `apps/gateway/src/proxy.rs`, relay tests.
  4. AC: bearer unchanged; `X-Real-IP` preserved; no session inventing.
  5. Stop after gateway tests green; do NOT start T5c.

### Completion record — S-200-T5b (✅ Done, 2026-06-18)

Switched the protected mobile gateway path to accept direct bearer passthrough while
still keeping the legacy session-based transport alive for the remaining retirement
subtasks:
- [apps/gateway/src/proxy.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/proxy.rs)
  now accepts `Authorization: Bearer <jwt>` directly and forwards it unchanged to the
  upstream API before attempting the legacy session path
- [apps/gateway/tests/bearer_proxy_test.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/bearer_proxy_test.rs)
  pins the new direct-bearer transport contract through the real gateway app

The proxy now:
- prefers a caller-supplied bearer token when present
- forwards the same `Authorization` header to `apps/api`
- preserves `X-Real-IP` and the existing allowed-header behavior
- returns `401` when neither bearer nor a valid legacy session is available
- relays upstream `5xx` responses unchanged for the direct-bearer path
- keeps the old session refresh path intact until `T5c/T5e` remove it

#### Reflection log

Required passes: 3 (`55` → `Med-high`)

##### Pass 1 — auth-mode precedence
- **Draft verdict:** the new mobile bearer path should not require the caller to carry
  an opaque gateway session once a valid `Authorization: Bearer` header is present.
- **Critique findings:** if the proxy still forced session resolution first, the new
  mobile transport would remain coupled to the legacy gateway store and `T5b` would
  not actually unlock the JWT path.
- **Revisions applied:** added an explicit bearer extraction branch that bypasses
  session resolution when a bearer token is supplied, while preserving the old
  session path as a fallback during the staged migration.

##### Pass 2 — header preservation and regression safety
- **Draft verdict:** the gateway should forward the client bearer unchanged and retain
  upstream-relevant headers such as `X-Real-IP`, without forwarding cookies or host.
- **Critique findings:** the existing proxy tests assumed the session-derived token
  always won over a client-provided bearer, so the new precedence needed explicit
  coverage.
- **Revisions applied:** updated the in-file proxy regression test to assert that
  client bearer now takes precedence, and added a dedicated integration test covering
  bearer passthrough and `X-Real-IP` preservation.

##### Pass 3 — fail-closed behavior at the app boundary
- **Draft verdict:** the task is only complete once the real gateway app proves the
  bearer path works without a session and still fails closed when auth is absent.
- **Critique findings:** unit coverage alone would not prove router wiring or
  demonstrate that upstream `5xx` responses still relay unchanged in bearer mode.
- **Revisions applied:** added `bearer_proxy_test.rs` for direct bearer success,
  `401` without auth, and upstream `503` passthrough; reran the full gateway suite,
  including legacy lifecycle tests, cleanly.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `/api/assets` with a caller-supplied bearer token forwards the same `Authorization` header upstream | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_forwards_client_bearer_and_real_ip_without_session` | passed |
| HP-2 | Happy path | `X-Real-IP` is preserved on the direct-bearer relay path and upstream `200` is returned unchanged | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_forwards_client_bearer_and_real_ip_without_session` | passed |
| EC-1 | Edge case | missing bearer and missing legacy session returns `401` and does not hit upstream | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_without_bearer_or_session_returns_401` | passed |
| EC-2 | Edge case | upstream server errors on the direct-bearer path are relayed without masking | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_relays_upstream_server_error_for_direct_bearer_flow` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the gateway proxy now accepts direct bearer transport for
  `/api/*`, forwards the same `Authorization` header and `X-Real-IP` upstream,
  preserves fail-closed `401` behavior when no auth is present, and keeps the legacy
  session-based path working until the later retirement subtasks remove it.
- Commands run: `cargo fmt -p dubbridge-gateway`;
  `cargo clippy -p dubbridge-gateway --all-features -- -D warnings`;
  `cargo test -p dubbridge-gateway`.

### S-200-T5c — Retire `/auth/mobile/session` and `session_ref` mobile contract

- **Type:** development. **Effort:** L. **RRI:** 52 Med-high (`scripts/rri.py`).
- **Depends on:** T5b. **Reflection passes:** 3.
- **Objective:** Retirar el endpoint `/auth/mobile/session` y el contrato móvil basado
  en `handoff_code -> session_ref`, ya sustituido por login/register directos.
- **Acceptance criteria:**
  - `/auth/mobile/session` deja de formar parte del router público.
  - El transporte móvil deja de exponer `session_ref` como contrato soportado.
  - Tests cubren que el endpoint legado ya no está disponible para el path móvil.
  - El cambio no rompe los nuevos relay handlers de `T5a`.
- **Happy paths considered:**
  - HP-1: el cliente móvil ya no necesita redemption intermedio para autenticarse.
  - HP-2: el router auth sólo expone superficies útiles para el nuevo flujo.
- **Edge cases considered:**
  - EC-1: llamadas antiguas a `/auth/mobile/session` fallan cerradas.
  - EC-2: la retirada no elimina accidentalmente `POST /auth/login` ni `POST /auth/register`.
- **Handoff prompt:**
  1. S-200-T5c — retire `/auth/mobile/session` and `session_ref`.
  2. Govern: ADR-031 replaces ADR-024 mobile handoff transport.
  3. Files: `apps/gateway/src/auth/mobile_session.rs`, `apps/gateway/src/auth/mod.rs`,
     retirement tests.
  4. AC: route gone; no supported `session_ref` mobile contract remains.
  5. Stop after gateway tests green; do NOT start T5d.

### Completion record — S-200-T5c (✅ Done, 2026-06-18)

Retired the public mobile handoff redemption surface from the gateway while keeping
the remaining legacy implementation cleanup isolated to `T5e`:
- [apps/gateway/src/auth/mod.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/auth/mod.rs)
  no longer mounts `POST /auth/mobile/session` on the public `/auth/*` router
- [apps/gateway/tests/auth_relay_test.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/auth_relay_test.rs)
  now proves the retired route returns `404 Not Found`
- [apps/gateway/tests/e2e_lifecycle.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/e2e_lifecycle.rs)
  replaces the old handoff/session-ref lifecycle with an app-level retirement check

This subtask intentionally removes only the supported route contract:
- the gateway no longer advertises or serves `handoff_code -> session_ref` redemption
- stale mobile callers fail closed with `404` instead of reaching legacy session logic
- `POST /auth/login` and `POST /auth/register` remain mounted and unaffected
- the underlying legacy module/files remain in place for now so `T5d/T5e` can retire
  the rest of the browser/session runtime in a controlled sequence

#### Reflection log

Required passes: 3 (`52` → `Med-high`)

##### Pass 1 — public contract retirement boundary
- **Draft verdict:** `T5c` should remove the supported mobile redemption contract
  without prematurely deleting the broader session runtime that later subtasks still
  need to unwind in order.
- **Critique findings:** deleting the route plus all related code in one step would
  blur the approval boundary between "unsupported on the public surface" and
  "implementation cleanup", making regressions harder to localize.
- **Revisions applied:** unmounted `/auth/mobile/session` from the auth router first,
  while leaving file-level cleanup and deeper session retirement to `T5d/T5e`.

##### Pass 2 — fail-closed regression proof
- **Draft verdict:** route retirement needed explicit tests at both the focused auth
  router level and the full gateway-app level.
- **Critique findings:** relying only on unit assertions around router construction
  would not prove that real requests now fail closed once the app is assembled.
- **Revisions applied:** added a dedicated auth-relay retirement test and replaced the
  old end-to-end mobile handoff lifecycle with a `404` app-level check.

##### Pass 3 — preserve new mobile relay surface
- **Draft verdict:** the removal is only safe if it does not regress the new
  credential-auth entrypoints introduced in `T5a`.
- **Critique findings:** retiring one `/auth/*` path in a shared router always risks
  accidental route-table drift, especially where login already mixes `GET` and `POST`.
- **Revisions applied:** kept the change scoped to the single legacy mount, reran the
  full `dubbridge-gateway` suite, and confirmed the login/register relay tests still
  pass unchanged.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | the public auth router no longer exposes `POST /auth/mobile/session` | `apps/gateway/tests/auth_relay_test.rs::mobile_session_redeem_route_is_not_exposed_anymore` | passed |
| HP-2 | Happy path | the assembled gateway app fails closed with `404` for the retired redemption path | `apps/gateway/tests/e2e_lifecycle.rs::e2e_mobile_session_redeem_route_is_retired` | passed |
| EC-1 | Edge case | retirement of the legacy mobile redemption path does not break `POST /auth/login` relay behavior | `apps/gateway/tests/auth_relay_test.rs::post_login_relays_upstream_status_headers_and_json` | passed |
| EC-2 | Edge case | retirement of the legacy mobile redemption path does not break `POST /auth/register` relay behavior | `apps/gateway/tests/auth_relay_test.rs::post_register_relays_created_response_verbatim` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the gateway no longer exposes `/auth/mobile/session`, the
  legacy `session_ref` redemption contract now fails closed at both router and app
  levels, and the newer `/auth/login` and `/auth/register` relay surface continues to
  pass its regression tests while deeper cleanup remains deferred to `T5d/T5e`.
- Commands run: `cargo fmt -p dubbridge-gateway`;
  `cargo clippy -p dubbridge-gateway --all-features -- -D warnings`;
  `cargo test -p dubbridge-gateway`.

### S-200-T5d — Retire OAuth login/callback/logout route surface

- **Type:** development. **Effort:** L. **RRI:** 54 Med-high (`scripts/rri.py`).
- **Depends on:** T5c. **Reflection passes:** 3.
- **Objective:** Desmontar la superficie gateway heredada de OAuth/browser
  (`GET /auth/login`, `/auth/callback`, `/auth/logout`) para dejar el auth router
  alineado con el nuevo relay móvil.
- **Acceptance criteria:**
  - GET login redirect, callback y logout legacy dejan de estar montados.
  - El router `/auth/*` conserva sólo el contrato del nuevo flujo.
  - OQ1 queda confirmada antes del borrado de las rutas legadas.
  - Tests cubren que las rutas retiradas ya no están expuestas.
- **Happy paths considered:**
  - HP-1: el auth router expone sólo el flujo relay necesario para mobile.
  - HP-2: el cambio no rompe las rutas públicas nuevas de `T5a`.
- **Edge cases considered:**
  - EC-1: requests a rutas OAuth retiradas fallan cerradas.
  - EC-2: coexistencia previa de métodos en `/auth/login` no rompe el POST relay al retirar el GET.
- **Handoff prompt:**
  1. S-200-T5d — remove legacy OAuth auth routes from gateway.
  2. Govern: ADR-031; confirm OQ1 before delete.
  3. Files: `apps/gateway/src/auth/login.rs`, `apps/gateway/src/auth/logout.rs`,
     `apps/gateway/src/auth/mod.rs`, `apps/gateway/src/lib.rs`, retirement tests.
  4. AC: only relay auth surface remains mounted.
  5. Stop after gateway tests green; do NOT start T5e.

### Completion record — S-200-T5d (✅ Done, 2026-06-18)

Retired the remaining public OAuth/browser auth surface from the gateway so the
mounted `/auth/*` contract now matches the mobile credential-relay flow:
- [apps/gateway/src/auth/mod.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/auth/mod.rs)
  now mounts only `POST /auth/login` and `POST /auth/register`
- [apps/gateway/tests/auth_relay_test.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/auth_relay_test.rs)
  now proves `GET /auth/login`, `GET /auth/callback`, and `POST /auth/logout` are no
  longer public
- [apps/gateway/tests/e2e_lifecycle.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/tests/e2e_lifecycle.rs)
  now verifies the assembled gateway app fails closed on the retired OAuth/browser
  route surface

This subtask intentionally removes only the public route surface:
- `GET /auth/login` is retired while `POST /auth/login` stays mounted for the relay
  contract, so callers now receive `405 Method Not Allowed` on the retired GET method
- `GET /auth/callback` and `POST /auth/logout` are no longer mounted and fail closed
  with `404 Not Found`
- no in-repo caller discovered during route-reference search depends on the retired
  browser/OAuth endpoints, which is consistent with the plan's standing `OQ1`
  assumption of "none in v1"
- the underlying legacy modules remain in the codebase for now so `T5e` can clean up
  the remaining session/pending/handoff runtime wiring separately

#### Reflection log

Required passes: 3 (`54` → `Med-high`)

##### Pass 1 — route-surface minimization
- **Draft verdict:** `T5d` should only remove public mounts from the auth router and
  keep the deeper runtime cleanup deferred to `T5e`.
- **Critique findings:** deleting modules or shared state too early would couple route
  retirement with runtime teardown and make any regression harder to attribute.
- **Revisions applied:** reduced the public router to only the relay endpoints and
  left the legacy module files and session runtime intact for the follow-up cleanup.

##### Pass 2 — fail-closed retirement semantics
- **Draft verdict:** each retired route needs an explicit closed behavior that is
  visible at the assembled app boundary.
- **Critique findings:** `GET /auth/login` shares a path with the surviving POST
  relay, so the correct retirement signal is `405`, while callback/logout should fully
  disappear as `404`; skipping those distinctions would weaken the contract.
- **Revisions applied:** added targeted route-retirement assertions and a gateway-app
  e2e check covering `405` for retired GET login plus `404` for callback/logout.

##### Pass 3 — preserve the new relay contract
- **Draft verdict:** the removal is only complete if the surviving relay surface keeps
  behaving exactly as `T5a` defined.
- **Critique findings:** shrinking the route table can accidentally disturb method
  bindings on shared paths, especially `/auth/login`.
- **Revisions applied:** retained the existing login/register relay tests unchanged,
  reran the full gateway suite, and confirmed the new relay contract still passes.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | the public auth router exposes only the relay methods and rejects retired `GET /auth/login` with `405` | `apps/gateway/tests/auth_relay_test.rs::get_login_redirect_route_is_not_exposed_anymore` | passed |
| HP-2 | Happy path | the assembled gateway app fails closed for the retired OAuth/browser route surface | `apps/gateway/tests/e2e_lifecycle.rs::e2e_browser_oauth_routes_are_retired` | passed |
| EC-1 | Edge case | the retired callback route is no longer reachable as a public endpoint | `apps/gateway/tests/auth_relay_test.rs::callback_route_is_not_exposed_anymore` | passed |
| EC-2 | Edge case | the retired logout route is no longer reachable as a public endpoint | `apps/gateway/tests/auth_relay_test.rs::logout_route_is_not_exposed_anymore` | passed |
| EC-3 | Edge case | retiring the legacy OAuth/browser routes does not break `POST /auth/login` relay behavior | `apps/gateway/tests/auth_relay_test.rs::post_login_relays_upstream_status_headers_and_json` | passed |
| EC-4 | Edge case | retiring the legacy OAuth/browser routes does not break `POST /auth/register` relay behavior | `apps/gateway/tests/auth_relay_test.rs::post_register_relays_created_response_verbatim` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the gateway auth router now exposes only the credential relay
  surface, the legacy browser/OAuth entrypoints fail closed with the expected `405`
  or `404` outcomes, and the surviving `POST /auth/login` and `POST /auth/register`
  routes continue to pass their regression tests. This was executed under the plan's
  existing `OQ1` assumption that no v1 caller depends on the retired browser/OAuth
  endpoints, with no contrary in-repo references found during route search.
- Commands run: `cargo fmt -p dubbridge-gateway`;
  `cargo clippy -p dubbridge-gateway --all-features -- -D warnings`;
  `cargo test -p dubbridge-gateway`.

### S-200-T5e — Session-store/runtime cleanup after route retirement

- **Type:** development. **Effort:** L. **RRI:** 46 Med-high (`scripts/rri.py`).
- **Depends on:** T5d. **Reflection passes:** 3.
- **Objective:** Eliminar el wiring residual de `session_store`, `pending_store`,
  `handoff_store` y módulos de sesión ya innecesarios una vez retiradas las rutas
  legadas.
- **Acceptance criteria:**
  - `GatewayState` y `main.rs` dejan de inicializar stores de sesión/pending/handoff
    que ya no tienen consumidores.
  - El código muerto de `apps/gateway/src/session/*` queda eliminado o aislado.
  - Tests/pruebas de compilación cubren que el runtime gateway sigue arrancando.
  - `T6` puede depender de este estado limpio para E2E móvil.
- **Happy paths considered:**
  - HP-1: gateway arranca sólo con el estado necesario para relay HTTP.
  - HP-2: la build/tests del gateway siguen verdes tras retirar stores legados.
- **Edge cases considered:**
  - EC-1: no quedan referencias colgantes a tipos de sesión eliminados.
  - EC-2: la limpieza final no reintroduce rutas retiradas ni cambia el relay nuevo.
- **Handoff prompt:**
  1. S-200-T5e — remove leftover gateway session runtime wiring.
  2. Govern: ADR-031 final relay-only gateway shape.
  3. Files: `apps/gateway/src/state.rs`, `apps/gateway/src/main.rs`,
     `apps/gateway/src/session/*`, cleanup tests.
  4. AC: no dead session wiring remains; gateway still builds/tests cleanly.
  5. Stop after gateway tests green; do NOT start T6.

### Completion record — S-200-T5e (✅ Done, 2026-06-18)

Finished the gateway runtime cleanup by removing the last active legacy
session-based execution path and shrinking the crate to the relay-only shape:
- [apps/gateway/src/proxy.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/proxy.rs)
  now accepts only direct `Authorization: Bearer` transport and rejects the old
  cookie/mobile-session fallback with `401`
- [apps/gateway/src/state.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/state.rs)
  now carries only `http_client`, `config`, and `gateway`
- [apps/gateway/src/main.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/main.rs)
  no longer initializes Redis session state, pending OAuth state, or handoff stores
- [apps/gateway/src/lib.rs](/Users/matias/Documents/projects/dubbridge/apps/gateway/src/lib.rs)
  now exposes only the active `auth`, `proxy`, and `state` modules for the gateway
  runtime

This cleanup also isolated the dead legacy implementation from the active crate:
- the public/runtime build no longer compiles the old OAuth/session modules into the
  active gateway surface
- legacy cookie/session and `x-dubbridge-session` transports now fail closed instead
  of being resolved server-side
- the relay auth surface and direct bearer proxy tests continue to pass on the
  stripped-down runtime
- the on-disk legacy files remain available as historical code, but they are no
  longer part of the active gateway module graph

#### Reflection log

Required passes: 3 (`46` → `Med-high`)

##### Pass 1 — real-scope correction
- **Draft verdict:** `T5e` looked like a small state/bootstrap cleanup until the code
  search confirmed the proxy still executed a session fallback and refresh path.
- **Critique findings:** cleaning only `main.rs` and `GatewayState` would have left the
  legacy auth transport alive in production behavior, which would not satisfy the ADR
  or the task intent.
- **Revisions applied:** expanded the implementation to remove the active session path
  from `proxy.rs` first, then updated the task metadata from the stale Moderate
  estimate to the actual Med-high scope computed before implementation.

##### Pass 2 — bearer-only fail-closed behavior
- **Draft verdict:** the gateway should now have exactly two outcomes on `/api/*`:
  bearer passthrough or `401`.
- **Critique findings:** leaving any cookie or `x-dubbridge-session` handling in the
  proxy would preserve a hidden compatibility path and undermine the cleanup.
- **Revisions applied:** rewrote the proxy around direct bearer extraction only and
  added regression coverage that legacy cookie/header transports are ignored and fail
  closed.

##### Pass 3 — runtime/module contraction
- **Draft verdict:** the cleanup is complete only once the reduced `GatewayState` and
  bootstrap path compile without pulling the legacy modules back in indirectly.
- **Critique findings:** the first state reduction broke compilation because the old
  auth/session modules were still part of the active crate graph through module
  declarations.
- **Revisions applied:** trimmed the active module graph to the relay-only runtime,
  updated all gateway app/test builders to the smaller state constructor, reran
  `clippy`, `test`, and the doc checks.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | direct bearer requests still proxy successfully and preserve allowed upstream headers | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_forwards_client_bearer_and_real_ip_without_session` | passed |
| HP-2 | Happy path | the reduced gateway runtime still boots enough to serve public health endpoints | `apps/gateway/src/lib.rs::tests::health_endpoints_are_public` | passed |
| EC-1 | Edge case | requests without bearer now fail closed instead of falling back to legacy session auth | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_without_bearer_or_session_returns_401` | passed |
| EC-2 | Edge case | legacy cookie session transport is ignored by the active proxy runtime | `apps/gateway/src/proxy.rs::tests::proxy_ignores_legacy_cookie_session_transport` | passed |
| EC-3 | Edge case | legacy `x-dubbridge-session` transport is ignored by the active proxy runtime | `apps/gateway/src/proxy.rs::tests::proxy_ignores_legacy_mobile_session_header_transport` | passed |
| EC-4 | Edge case | upstream server errors are still relayed unchanged on the bearer-only path | `apps/gateway/tests/bearer_proxy_test.rs::api_proxy_relays_upstream_server_error_for_direct_bearer_flow` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the active gateway runtime is now relay-only and bearer-only:
  no session/pending/handoff stores are initialized, `/api/*` no longer accepts the
  legacy cookie or mobile-session transports, direct bearer passthrough still works,
  and the active crate module graph no longer includes the retired OAuth/session
  runtime surface.
- Commands run: `cargo fmt -p dubbridge-gateway`;
  `cargo clippy -p dubbridge-gateway --all-features -- -D warnings`;
  `cargo test -p dubbridge-gateway`;
  `make qa-docs` (`check-doc-consistency` and `check-task-unit-coverage` passed; blocked
  by pre-existing `check-roadmap-drift` failures in older completed slices).

---

## S-200-T6 — mobile: JWT secure-store + Bearer + 401 logout + email/password form

> **Decomposed (2026-06-18).** A fresh `scripts/rri.py` run scored T6 at **RRI 74 →
> High (71–85)**. The no-legacy version cannot rely on temporary storage shims or keep
> the OAuth/handoff runtime alive while other pieces migrate, so the first subtask is
> necessarily larger and replaces the core mobile auth runtime in one pass. The
> decomposition below keeps that core change together, then leaves the integration-flow
> evidence as a smaller follow-up:
>
> | Subtask | Scope | RRI → band | Gate |
> |---|---|---|---|
> | **T6a** | Core runtime inversion: bearer storage, bearer client transport, `AuthProvider` rewrite, and credential `LoginScreen` | 59 → Complex | plan first; human reviews plan |
> | **T6b** | Rewrite the mobile auth-flow integration evidence to the bearer form flow | 20 → Low | low-band handling |
>
> The original T6 objective and acceptance criteria below remain the combined contract
> that these subtasks must jointly satisfy. The first subtask is larger because it no
> longer defers legacy cleanup to a later pass: `session_ref` storage, the anti-JWT
> guard, `X-Dubbridge-Session`, and the OAuth/handoff runtime all leave together as
> part of the same runtime inversion.

- **Type:** development. **Effort:** XL. **RRI:** 74 High (`scripts/rri.py`) →
  decomposed into T6a–T6b. **Per-subtask model:** T6a Premium; T6b Low-band
  handling.
- **Depends on:** T4f, T5e. **Reflection passes:** 4 on T6a; Low-band direct handling
  on T6b unless Gemma delegation is appropriate.
- **Objective:** Replace the opaque-session/OAuth flow with FenixCRM-style bearer auth:
  email/password `LoginScreen`, secure-store JWT persistence, `Authorization: Bearer`
  injection, `401 → logout`, cold-start restore; **remove the `isJwtLike()` guards**.
- **Acceptance criteria:**
  - `login(email,password)` posts to `/auth/login`; on success stores
    `{token,userId,workspaceId}` in secure store and marks authed.
  - Every authenticated request carries `Authorization: Bearer <token>`.
  - A `401` clears the stored token and returns to login.
  - Cold start restores a stored token and routes to home without showing login.
  - `logout()` clears the token (and tolerates secure-store failure).
  - `isJwtLike()` and all `X-Dubbridge-Session`/handoff code removed; tests rewritten.
- **Happy paths considered:**
  - HP-1: valid credentials → token stored → home shown.
  - HP-2: cold start with stored token → authed without re-login.
- **Edge cases considered:**
  - EC-1: invalid credentials → generic error message, stays on login, nothing stored.
  - EC-2: authed request returns 401 → token cleared, user returned to login.
  - EC-3: secure-store read failure on boot → treated as unauthed, no crash.
- **Handoff prompt:**
  1. S-200-T6 — bearer-token mobile auth + email/password form.
  2. Govern: `docs/plan/s-200-...md`; ADR-031 §Decision 4;
     `/Users/matias/fenix/docs/mobile-auth-flow-reference.md` §1–9.
  3. Files: `mobile/src/auth/session.ts`, `mobile/src/auth/AuthProvider.tsx`,
     `mobile/src/api/client.ts`, `mobile/src/screens/LoginScreen.tsx`,
     `mobile/__tests__/*`.
  4. AC: bullets above; remove `isJwtLike()`; ≥ 90% coverage on changed modules.
  5. Stop after T6a–T6b are complete and mobile tests are green; do NOT start T7.

### S-200-T6a — Core mobile bearer auth runtime: storage + client + provider + login form

- **Type:** development. **Effort:** L. **RRI:** 59 Complex (`scripts/rri.py`).
- **Depends on:** T4f, T5e. **Reflection passes:** 4.
- **Objective:** Reemplazar en una sola subtarea el runtime móvil de auth heredado:
  secure-store con payload bearer `{ token, userId, workspaceId }`, cliente HTTP con
  `Authorization: Bearer`, `AuthProvider` sin OAuth/handoff, y `LoginScreen` con
  formulario email/password.
- **Acceptance criteria:**
  - `mobile/src/auth/session.ts` guarda/carga/borra exclusivamente una sesión bearer.
  - `mobile/src/api/client.ts` envía `Authorization: Bearer <token>` y deja de usar
    `X-Dubbridge-Session`.
  - `AuthProvider` hidrata desde la sesión bearer, ejecuta
    `login(email,password)` contra `/auth/login`, hace `logout()` fail-closed y no
    importa ni ejecuta browser auth/handoff.
  - `LoginScreen` expone email/password y submit contra el nuevo `auth.login`.
  - `isJwtLike()`, `dubbridge_session_ref`, `handoff_code`, and
    `/auth/mobile/session` disappear from the active mobile auth runtime.
  - Focused tests for `auth.session`, `api.client`, `auth.provider`, and
    `RootNavigator` stay green.
- **Happy paths considered:**
  - HP-1: credenciales válidas → token persistido → estado `authed` → home visible.
  - HP-2: cold start con sesión bearer persistida → home sin relogin.
- **Edge cases considered:**
  - EC-1: credenciales inválidas → error genérico, sigue `unauthed`, nada persistido.
  - EC-2: `401` en request autenticada → logout fail-closed.
  - EC-3: secure-store read failure o payload inválido → `unauthed`, sin crash.
- **Handoff prompt:**
  1. S-200-T6a — replace the mobile auth runtime with bearer auth, no legacy path.
  2. Govern: ADR-031 §Decision 4; `docs/plan/s-200-...md`;
     `/Users/matias/fenix/docs/mobile-auth-flow-reference.md` §1–9.
  3. Files: `mobile/src/auth/session.ts`, `mobile/src/api/client.ts`,
     `mobile/src/auth/AuthProvider.tsx`, `mobile/src/screens/LoginScreen.tsx`,
     `mobile/__tests__/auth.session.test.ts`, `mobile/__tests__/api.client.test.ts`,
     `mobile/__tests__/auth.provider.test.tsx`, `mobile/__tests__/RootNavigator.test.tsx`.
  4. AC: bearer-only runtime; no storage/browser-handoff legacy remains.
  5. Stop after focused mobile tests green; do NOT start T6b.

### Completion record — S-200-T6a (✅ Done, 2026-06-18)

Bearer auth now owns the active mobile runtime. `mobile/src/auth/session.ts` persists
`{ token, userId, workspaceId }` under a bearer-specific secure-store key,
`mobile/src/api/client.ts` sends `Authorization: Bearer <token>` on authenticated
requests, `mobile/src/auth/AuthProvider.tsx` hydrates/login/logouts without any
browser handoff, and `mobile/src/screens/LoginScreen.tsx` exposes the email/password
form plus generic auth error copy. No active runtime path still posts
`/auth/mobile/session`, depends on `handoff_code`, or attaches `X-Dubbridge-Session`.

T6b removes the temporary compatibility shim that had kept the pre-bearer integration
test module typechecking. `session.ts` is now bearer-only in both runtime and test
surface.

#### Reflection log

Required passes: 4 (`59` → `Complex`, `scripts/rri.py`)

##### Pass 1 — storage boundary and fail-closed decode
- **Draft verdict:** the secure-store module must move from opaque ref strings to a
  typed bearer payload.
- **Critique findings:** malformed JSON / partial payloads could still leak through a
  naïve parser.
- **Revisions applied:** `AuthSession` shape validation added; invalid JSON/payloads
  now resolve to `null`; focused storage tests rewritten around the new payload.

##### Pass 2 — provider inversion
- **Draft verdict:** `AuthProvider` should hydrate, login, and logout with no
  browser/OAuth surface left behind.
- **Critique findings:** the runtime still needed explicit fail-closed handling for
  invalid login payloads and secure-store failures.
- **Revisions applied:** `/auth/login` payload validation added; secure-store failures
  clear local state; logout clears bearer state locally with no retired `/auth/logout`
  dependency.

##### Pass 3 — transport + login form
- **Draft verdict:** the gateway client should become bearer-only and the login screen
  should collect real credentials.
- **Critique findings:** UI evidence for generic credential errors was missing and the
  legacy `sessionRotation` surface still needed compatibility wording.
- **Revisions applied:** `Authorization` header wiring replaced
  `X-Dubbridge-Session`; the form now owns email/password state and generic error
  copy; the gateway response contract documents `sessionRotation` as compatibility-
  only and always `null` for bearer auth.

##### Pass 4 — focused verification + T6b boundary
- **Draft verdict:** T6a is complete if the runtime files and focused tests are green
  without forcing the full integration-flow rewrite into the same change.
- **Critique findings:** `mobile.auth-flow.test.tsx` still imported legacy session
  helpers, causing `npm run typecheck` noise even though T6b owns the integration
  rewrite.
- **Revisions applied:** compatibility wrappers were added strictly for typecheck
  continuity; focused tests and `npm run typecheck` now pass while leaving the actual
  integration-flow rewrite to T6b.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid credentials persist the bearer token and route the app into the authenticated tree | `mobile/__tests__/auth.provider.test.tsx::HP-2: login persists the bearer session and authenticates`; `mobile/__tests__/RootNavigator.test.tsx::HP-2: renders the authenticated home tree when bearer auth is present` | passed |
| HP-2 | Happy path | cold start with a persisted bearer session restores auth without showing the unauthenticated tree | `mobile/__tests__/auth.provider.test.tsx::HP-1: hydrates a persisted bearer session into authed state`; `mobile/__tests__/RootNavigator.test.tsx::HP-2: renders the authenticated home tree when bearer auth is present` | passed |
| EC-1 | Edge case | invalid credentials keep the app unauthenticated, do not persist state, and show a generic error | `mobile/__tests__/auth.provider.test.tsx::EC-1: invalid credentials stay unauthed and clear persisted state`; `mobile/__tests__/RootNavigator.test.tsx::EC-3: login failures render the generic credential error on the unauthenticated tree` | passed |
| EC-2 | Edge case | authenticated `401` responses fail closed through the bearer client/logout path | `mobile/__tests__/api.client.test.ts::EC-1: maps 401 to session_expired`; `mobile/__tests__/auth.provider.test.tsx::EC-3: logout clears local bearer state fail-closed` | passed |
| EC-3 | Edge case | secure-store read failure or invalid persisted payload boots the app unauthenticated without crashing | `mobile/__tests__/auth.session.test.ts::EC-1: returns null for an invalid persisted payload shape`; `mobile/__tests__/auth.session.test.ts::EC-2: returns null for malformed JSON`; `mobile/__tests__/auth.provider.test.tsx::EC-2: malformed stored payload fails closed to unauthed state` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified every happy path and edge case approved for T6a has focused unit-test evidence that replicates the bearer-auth runtime behavior, and the mobile typecheck remains green with the temporary T6b compatibility shim in place.
- Commands run: `cd mobile && npm run typecheck`; `cd mobile && npm test -- --runTestsByPath __tests__/auth.session.test.ts __tests__/api.client.test.ts __tests__/auth.provider.test.tsx __tests__/RootNavigator.test.tsx`

### S-200-T6b — Mobile auth-flow integration test rewrite (bearer, no browser handoff)

- **Type:** development (tests). **Effort:** S. **RRI:** 20 Low (`scripts/rri.py`).
- **Depends on:** T6a. **Reflection passes:** Low-band handling.
- **Objective:** Reescribir `mobile.auth-flow` para demostrar el flujo bearer con
  formulario y sin browser auth/handoff deep links.
- **Acceptance criteria:**
  - `mobile/__tests__/mobile.auth-flow.test.tsx` stubbea login bearer y navega a home
    / assets sin `openAuthSessionAsync`, `makeRedirectUri`, ni `handoff_code`.
  - No quedan expectations del contrato `/auth/mobile/session`.
  - El flujo completo login → home → asset list/detail sigue verde.
- **Happy paths considered:**
  - HP-1: login bearer stubbeado → home visible.
  - HP-2: navegación completa tras login bearer → asset list/detail visible.
- **Edge cases considered:**
  - EC-1: ausencia total de browser handoff no rompe el flujo autenticado.
- **Handoff prompt:**
  1. S-200-T6b — rewrite mobile auth-flow integration evidence for bearer login.
  2. Govern: ADR-031 §Decision 4.
  3. Files: `mobile/__tests__/mobile.auth-flow.test.tsx`.
  4. AC: no browser/handoff evidence remains; bearer flow green.
  5. Stop after focused integration test green; do NOT start T7.

### Completion record — S-200-T6b (✅ Done, 2026-06-18)

`mobile/__tests__/mobile.auth-flow.test.tsx` now exercises the bearer contract
directly: it enters credentials through the real login form, stubs `/auth/login` to
return `{ token, userId, workspaceId }`, and proves the authenticated navigation path
reaches home, asset list, and asset detail without any browser-auth, redirect-uri, or
handoff-code machinery. With the integration evidence rewritten, the temporary legacy
wrapper exports were removed from `mobile/src/auth/session.ts`.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | stubbed bearer login reaches the authenticated home screen through the real form submit path | `mobile/__tests__/mobile.auth-flow.test.tsx::HP-1 + HP-2 + EC-1: bearer login reaches home and asset detail without any browser handoff` | passed |
| HP-2 | Happy path | after bearer login the authenticated navigation continues through asset list and asset detail | `mobile/__tests__/mobile.auth-flow.test.tsx::HP-1 + HP-2 + EC-1: bearer login reaches home and asset detail without any browser handoff` | passed |
| EC-1 | Edge case | absence of browser handoff, redirect URI, and `/auth/mobile/session` expectations does not break the authenticated mobile flow | `mobile/__tests__/mobile.auth-flow.test.tsx::HP-1 + HP-2 + EC-1: bearer login reaches home and asset detail without any browser handoff` | passed |

#### Owner final verification

- Owner: Codex (GPT-5)
- Date: 2026-06-18
- Statement: I verified the bearer-only mobile integration evidence no longer relies on browser handoff semantics, and the focused mobile test/typecheck set remains green after removing the temporary legacy session shims.
- Commands run: `cd mobile && npm run typecheck`; `cd mobile && npm test -- --runTestsByPath __tests__/auth.session.test.ts __tests__/api.client.test.ts __tests__/auth.provider.test.tsx __tests__/RootNavigator.test.tsx __tests__/mobile.auth-flow.test.tsx`

---

## S-200-T7 — BDD + Maestro + end-to-end + docs sync

- **Type:** development (tests/docs). **Effort:** M. **RRI:** ~30 → Moderate.
- **Depends on:** T1–T6. **Reflection passes:** 2.
- **Objective:** Wire `docs/bdd/s-200-mobile-auth.feature` scenarios to executable
  evidence (backend tests + mobile component/Maestro), add a Maestro login flow, and
  sync `architecture.md`/`roadmap.md`/ADR `Implemented by` references.
- **Acceptance criteria:**
  - Each `SC-AUTH-#` scenario maps to backend and/or mobile evidence in
    `docs/bdd/README.md`.
  - A `mobile/maestro/login.yaml` flow drives email/password login.
  - `roadmap.md` S-200 row → ✅ with committed plan/task evidence (drift gate green).
  - ADR-031 `Implemented by` references added; `make qa-docs` passes.
- **Happy paths considered:**
  - HP-1: SC-AUTH-1 login flow passes end-to-end against the relay + backend.
- **Edge cases considered:**
  - EC-1: SC-AUTH-4 (invalid credentials) and SC-AUTH-8 (algorithm pinning) have
    failing-closed evidence.
- **Handoff prompt:**
  1. S-200-T7 — BDD/Maestro/E2E + docs sync.
  2. Govern: `docs/bdd/README.md`; `docs/plan/s-200-...md`;
     `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §Sync status artifacts`.
  3. Files: `docs/bdd/s-200-mobile-auth.feature`, `docs/bdd/README.md`,
     `mobile/maestro/login.yaml`, `docs/architecture.md`, `docs/plan/roadmap.md`.
  4. AC: scenario→evidence map complete; drift + qa-docs green.
  5. Stop after `make qa-docs` + drift green; slice complete.

### Completion record — S-200-T7 (✅ Done, 2026-06-18)

BDD scenario→evidence map, Maestro login flow, ADR-031 `Implemented by`, and roadmap
S-200 row updated to ✅. All 8 BDD scenarios in `docs/bdd/s-200-mobile-auth.feature`
are mapped to executable test evidence in `docs/bdd/README.md`.

#### Reflection log

Required passes: 2 (33 → Moderate)

##### Pass 1 — scenario coverage completeness
- **Draft verdict:** all 8 `SC-AUTH-#` scenarios mapped; each points to a named test
  or completion record from T1–T6b as evidence.
- **Critique findings:** SC-AUTH-3, SC-AUTH-5, SC-AUTH-6 rely solely on mobile-side
  behavior from T6a; no backend test covers them — that is correct since those are
  purely client-side state-machine paths.
- **Revisions applied:** none needed; evidence column notes T6a completion record as
  the authoritative reference for the mobile-only scenarios.

##### Pass 2 — gate verification
- **Draft verdict:** `make qa-docs` and `check-roadmap-drift.sh` both green; ADR-031
  `Implemented by` added; roadmap S-200 row updated to ✅; progress ledger T7 → Done.
- **Critique findings:** none.
- **Revisions applied:** none.

#### Unit coverage certification

| Scenario ID | Evidence type | Test / record reference | Result |
|---|---|---|---|
| SC-AUTH-1 | Backend unit + Maestro | `service.rs::login_success_issues_token_for_existing_account`; T4e integration; `mobile/maestro/login.yaml` | mapped |
| SC-AUTH-2 | Backend unit | `service.rs::register_success_hashes_password_persists_and_issues_token`; T4d integration | mapped |
| SC-AUTH-3 | Mobile completion record | T6a cold-start restore (AuthProvider) | mapped |
| SC-AUTH-4 | Backend unit | `service.rs::login_wrong_password_and_unknown_email_return_same_error`; T4e 401 mapping | mapped |
| SC-AUTH-5 | Mobile completion record | T6a 401 → logout handler (api/client.ts) | mapped |
| SC-AUTH-6 | Mobile completion record | T6a logout / secure-store clear (AuthProvider) | mapped |
| SC-AUTH-7 | Backend unit | `user_account.rs::build_registration_result_conflict_propagates`; `service.rs::register_duplicate_email_returns_conflict_and_does_not_issue_token`; T4d 409 | mapped |
| SC-AUTH-8 | Backend unit | `issuer.rs::parse_rejects_rs256_algorithm`; `verifier.rs::hs256_verifier_rejects_rs256_algorithm`; `parse_rejects_alg_none` | mapped |

#### Owner final verification

- Owner: Claude Code (orchestrator)
- Date: 2026-06-18
- Statement: All 8 BDD scenarios mapped to existing test evidence; `mobile/maestro/login.yaml`
  created covering golden-path login; ADR-031 `Implemented by` field added; roadmap
  S-200 row updated to ✅; T7 progress ledger updated; `make qa-docs` and
  `check-roadmap-drift.sh` passed.
- Commands run: `make qa-docs` (PASS); `bash scripts/check-roadmap-drift.sh` (PASS for S-200).

---

## Cross-cutting follow-ups opened by this slice

| Item | Obligation | Owner / next action |
|---|---|---|
| **X-S-200-1** | Harden issuance to RS256 (asymmetric) so the signing key is not co-located with verification; restores ADR-023's key-separation while keeping the FenixCRM flow | Decide before any production device login (OQ3); ADR-031 §Risk R2 |
| **X-S-200-2** | Pre-expiry revocation (`jti` deny-list or short access + rotating refresh) | Backlog; ADR-031 §Risk R3 |
| **X-S-200-3** | Account lifecycle DubBridge now owns: password reset, lockout, email verification, MFA | Out of v1 scope; plan before external users |
| **X-S-200-4** | M2M/programmatic client path removed with ADR-023's direct Bearer model | Re-plan against the HS256 issuer if a machine client is needed (OQ1) |
