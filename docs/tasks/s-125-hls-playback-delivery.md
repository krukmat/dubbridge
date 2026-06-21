---
type: TaskList
title: "S-125 HLS Playback Delivery"
status: planned
slice: S-125
plan: docs/plan/s-125-hls-playback-delivery.md
---
# S-125 HLS Playback Delivery

> **Status:** Planned 2026-06-21. No task has started. Every code task **except T3a**
> is RRI 26+ and requires the full task-presentation + explicit human approval before
> implementation (`AGENT_WORKFLOW_GUIDE.md` §4). **T3a is RRI 21 (Low band)** and is
> delegated to local Gemma via Ollama with no full-approval presentation; the primary
> agent stays orchestrator/reviewer of record. RRI figures below were measured with
> `scripts/rri.py`, not estimated.
> **Plan:** `docs/plan/s-125-hls-playback-delivery.md`.
> **Governing ADR:** `docs/adr/ADR-032-hls-playback-delivery-boundary.md`.
> **Behavioral coverage contract:** unit-v1.

## Slice RRI summary

Scored as a single task, the slice is **RRI 100 (Very high)** — auth + audit +
migration + a new public delivery surface. That mandates decomposition before
implementation, which this ledger provides. Per-task measured RRI:

| Task | RRI | Band | Decomposition obligation |
|---|---|---|---|
| T0 | — | planning/docs | none (no code) |
| T1 | 42 | Med-high | none |
| T2 | 76 | High | triggered (T≥4, P≥4) — split or justify at approval |
| T3a | 21 | Low | **local Gemma-eligible** (pure manifest rewriter) |
| T3b | 49 | Med-high | none (storage scoped-reference helper — retained by primary agent) |
| T4 | 88 | Very high | triggered — split or justify at approval |
| T5 | 76 | High | triggered — split or justify at approval |

Tasks whose RRI triggers the decomposition gate (T2/T4/T5) must, at their own
approval checkpoint, either be split into approved subtasks or carry an explicit,
approved justification for proceeding whole (RRI policy §decomposition).

**T3 split (Gemma eligibility analysis, 2026-06-21).** T3 was split because exactly
one S-125 sub-part qualifies for local Gemma delegation. Only the **pure manifest
rewriter** clears RRI ≤ 25 with legitimately low D/K/P — it is a pure, IO-free,
single-file function with deterministic fixtures (best-fit per
`LOW_RRI_LOCAL_MODEL_HANDOFF.md`). Every other S-125 sub-part is floored above the
band by the anchor rubric: any `infra/migrations/**` / `crates/audit` touch carries
D4/P5/K4 + the authn/authz penalty (a trivial 1-line migration already scores RRI 42),
and the API/authz/grant work (T4/T5) is security-critical by construction. The
storage scoped-reference helper (T3b) sits on the no-raw-key boundary in
`crates/storage` (anchor floor 3/3/3) and is therefore retained by the primary agent,
not delegated. See `scripts/rri.py` runs recorded in this slice's planning history.

---

## S-125-T0: Plan, task ledger, BDD, and roadmap/ADR sync
**Effort:** S
**Complexity:** Low (planning/docs)
**Depends on:** None
**Status:** Done (2026-06-21)
**Type:** planning/docs (no code; RRI gate N/A)
**Objective:** Establish the `S-125` plan, this task ledger, the BDD feature skeleton,
and synchronize roadmap/architecture/ADR-032 so downstream tasks have a governing
contract.
**Inputs:**
- ADR-032 (Proposed).
- Roadmap `S-125` row + X25.
- `S-120` prepared-media plan/tasks (readiness + lineage contract this slice consumes).
**Outputs:**
- `docs/plan/s-125-hls-playback-delivery.md`.
- `docs/tasks/s-125-hls-playback-delivery.md` (this file).
- `docs/bdd/s-125-hls-playback-delivery.feature` + BDD README mapping rows.
- Roadmap/architecture references pointing at the new plan/tasks.
**Acceptance criteria:**
- Plan records objective, scope, governing constraints, design decisions, and the
  T0–T5 decomposition strategy.
- This ledger lists T1, T2, T3a, T3b, T4, T5 with measured RRI, dependencies,
  acceptance criteria, and happy/edge examples per development task.
- Roadmap `S-125` row references the new plan + task ledger (no longer "no plan yet").
- `make qa-docs` passes (index parity, dangling refs, OKF frontmatter).
**Related documents:** ADR-032; `docs/plan/roadmap.md`; `docs/architecture.md`;
`docs/plan/s-120-media-preparation.md`.
**Note:** ADR-032 stays `Proposed` until `S-125-T5` delivers the boundary, then flips
to `Accepted` with full propagation (ADR change-propagation contract).

---

## S-125-T1: Playback-grant domain contract + fail-closed state decoding
**Effort:** L (RRI 42 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T0
**Status:** Done (2026-06-21)
**Type:** development
**Objective:** Define the `PlaybackGrant` domain type — its scope, expiry, principal/
org/project binding, status, and denial reasons — with strict, fail-closed decoding of
stored values, in `crates/domain`. No IO, no persistence.
**Inputs:**
- `S-120` `PreparationStatus` / derived-artifact lineage types (`crates/domain/src/artifact.rs`).
- ADR-032 grant requirements (readiness, authorization, scope/expiry).
- Existing fail-closed decode pattern (`UnknownStoredValue`) used by S-120 domain types.
**Outputs:**
- `crates/domain/src/playback.rs`: `PlaybackGrant`, `PlaybackScope`, `GrantStatus`,
  `PlaybackDenial`, and a constructor/validator that rejects malformed grants.
- Unit tests for valid construction, expiry logic, and fail-closed decoding.
**Acceptance criteria:**
- `PlaybackGrant` carries asset id, grant id, scope, principal/org/project ref, issued/
  expiry timestamps, and status; no field allows an implicit "allow".
- `GrantStatus` and `PlaybackScope` decode strictly: an unknown stored token yields a
  typed error, never a default-allow.
- Expiry is evaluated against an injected clock/instant so it is unit-testable.
- ≥90% line coverage for the new module; no clippy warnings.
**Happy paths considered:**
- HP-1: valid grant fields → constructed grant in `Active` status with a future expiry.
- HP-2: grant evaluated before expiry → `valid`; same grant at/after expiry → `Expired`.
**Edge cases considered:**
- EC-1: unknown stored `GrantStatus`/`PlaybackScope` token → typed `UnknownStoredValue`
  error, never an allow.
- EC-2: expiry timestamp before issued timestamp → construction rejected.
**Files expected to change:** `crates/domain/src/playback.rs`; `crates/domain/src/lib.rs` (module export).
**Reflection strategy:** RRI 42 → Med-high → **3 passes**. Pass 1: correctness of grant
construction/expiry against HP-1/HP-2. Pass 2: fail-closed decoding completeness for
every stored enum (EC-1) and ordering invariants (EC-2). Pass 3: coverage-gap sweep and
API ergonomics for the repo/API consumers in T2/T4.
**Agent handoff prompt:** Add a pure `crates/domain` playback-grant contract with
strict fail-closed decoding and clock-injected expiry. No persistence, no API. Stop
after unit tests + coverage are green; do not start T2.

**Happy paths covered:**
- HP-1: `PlaybackGrant::new` with valid timestamps → `GrantStatus::Active` (`valid_grant_is_active`).
- HP-2: `is_valid_at` before expiry → `true`; at/after expiry → `false`
  (`grant_is_valid_before_expiry`, `grant_is_invalid_at_expiry`, `grant_is_invalid_after_expiry`).

**Edge cases covered:**
- EC-1: `"unknown_token".parse::<GrantStatus>()` → `PlaybackError::UnknownGrantStatus`; same for empty string and for `PlaybackScope` (`unknown_*_is_error`, `empty_string_*_is_error`). No branch returns a default-allow.
- EC-2: `expires_at == issued_at` and `expires_at < issued_at` both → `PlaybackError::InvalidExpiry` (`expiry_equal_to_issued_is_rejected`, `expiry_before_issued_is_rejected`).

**Correction to ledger (applied in-place):** the ledger cited `UnknownStoredValue` as a pattern in `crates/domain`; it is actually a `DbError` variant in `crates/db` used by the repo layer. T1 follows the actual domain pattern: `FromStr` returning a domain-level `PlaybackError`, which T2's repo layer will map to `DbError::UnknownStoredValue` (identical to `review.rs:68-77`).

### Reflection log

Required passes: 3 (RRI 42 → Med-high)

#### Pass 1
- **Draft verdict:** grant construction and expiry logic correct against HP-1/HP-2.
- **Critique findings:** `is_valid_at` boundary at exactly `expires_at` needed an explicit test; only before/after were covered.
- **Revisions applied:** added `grant_is_invalid_at_expiry` test.

#### Pass 2
- **Draft verdict:** fail-closed decode solid; `FromStr` never falls through to allow.
- **Critique findings:** empty-string tokens were not tested; only named-unknown tokens were.
- **Revisions applied:** added `empty_string_grant_status_is_error` and `empty_string_playback_scope_is_error`.

#### Pass 3
- **Draft verdict:** coverage complete; ergonomics for T2/T4 verified.
- **Critique findings:** `PlaybackScope::Display` formatting failed `cargo fmt` (multiline `write!`). No logic issues.
- **Revisions applied:** reformatted `Display` impl to pass `cargo fmt`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid fields → `Active` grant | `crates/domain/src/playback.rs::tests::valid_grant_is_active` | passed |
| HP-2 | Happy path | before expiry → valid; at/after → invalid | `::grant_is_valid_before_expiry`, `::grant_is_invalid_at_expiry`, `::grant_is_invalid_after_expiry` | passed |
| EC-1 | Edge case | unknown token → typed error, never allow | `::unknown_grant_status_is_error`, `::unknown_playback_scope_is_error`, `::empty_string_grant_status_is_error`, `::empty_string_playback_scope_is_error` | passed |
| EC-2 | Edge case | expiry ≤ issued → construction rejected | `::expiry_equal_to_issued_is_rejected`, `::expiry_before_issued_is_rejected` | passed |

### Owner final verification

- Owner: Claude Sonnet 4.6
- Date: 2026-06-21
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo test -p dubbridge-domain -- playback` (14 passed), `cargo test -p dubbridge-domain` (102 passed, 0 failed), `cargo clippy -p dubbridge-domain -- -D warnings` (clean), `cargo fmt -p dubbridge-domain -- --check` (clean)

---

## S-125-T2: Playback-grant schema + repository (grant → prepared-HLS lineage)
**Effort:** XL (RRI 76 — High; decomposition gate triggered)
**Recommended model:** Codex `GPT-5.2-Codex` (Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T1
**Status:** Not started
**Type:** development (migration + repository)
**Objective:** Persist playback grants as the system of record and resolve a grant to
the asset's prepared-HLS lineage + readiness, fail-closed.
**Decomposition obligation:** RRI 76 with T≥4 and P≥4 triggers the decomposition gate.
At the approval checkpoint, either split into `T2a` (migration) / `T2b` (repository) or
record an approved justification for proceeding whole. Default proposal: split.
**Inputs:**
- `PlaybackGrant` domain type from T1.
- `S-120` preparation status + `HlsManifest`/`HlsSegment` lineage rows
  (`crates/db/src/preparation_repo.rs`, migration `0019`).
- Migration numbering: next is `infra/migrations/0021_*`.
**Outputs:**
- `infra/migrations/0021_create_playback_grants.sql`.
- `crates/db/src/playback_repo.rs`: `issue_grant`, `get_active_grant`, `expire_grant`,
  and `resolve_grant_target` (grant → prepared HLS manifest lineage + `Ready` check).
- Integration tests in `apps/api/tests/` covering issue/lookup/expire/resolve.
**Acceptance criteria:**
- A grant can be issued, fetched while active, and is not returned once expired.
- `resolve_grant_target` returns the prepared `HlsManifest` lineage only when
  `PreparationStatus::Ready`; `Pending`/`InProgress`/`Failed`/absent → typed denial.
- Stored grant status decodes through the T1 fail-closed contract (no default-allow).
- ≥90% coverage for new repo code; `make qa-docs` green.
**Happy paths considered:**
- HP-1: issue grant for a `Ready` asset → fetchable, and `resolve_grant_target` returns
  its `HlsManifest` lineage.
- HP-2: expire an active grant → subsequent `get_active_grant` returns none.
**Edge cases considered:**
- EC-1: grant for an asset whose `PreparationStatus` is not `Ready` → resolve denies;
  no manifest leaked.
- EC-2: asset has no `HlsManifest` lineage row → resolve fails closed (no fabricated key).
- EC-3: expired grant → not returned by `get_active_grant`; resolve denies.
**Files expected to change:** `infra/migrations/0021_create_playback_grants.sql`;
`crates/db/src/playback_repo.rs`; `crates/db/src/lib.rs`; `apps/api/tests/playback_repo_test.rs`.
**Reflection strategy:** RRI 76 → **4 passes** (High band uses the Complex-band minimum
for safety on a security-anchored path). Pass 1: migration shape + repo CRUD vs HP-1/HP-2.
Pass 2: readiness/lineage fail-closed resolution (EC-1/EC-2). Pass 3: expiry + concurrency
(EC-3, double-issue). Pass 4: coverage + index/docs sync.
**Agent handoff prompt:** Add the `0021` playback-grant migration + `playback_repo`
with fail-closed `resolve_grant_target` gated on `S-120` readiness/lineage. Decompose
into T2a/T2b first unless the approver waives it. Stop after integration tests +
qa-docs green; do not start T3b.

---

## S-125-T3a: Pure manifest rewriter (local Gemma delegation)
**Effort:** S (RRI 21 — Low)
**Recommended model:** Local Gemma via Ollama (`DUBBRIDGE_LOW_RRI_MODEL`, default
`gemma4:12b-it-q4_K_M`). The primary agent remains orchestrator/reviewer of record.
**Depends on:** S-125-T3b (crate scaffold must exist first — Gemma cannot create the
crate/workspace wiring), S-125-T1 (grant-context type for the routed-base input).
**Status:** Not started
**Type:** development (Low-band, local delegation)
**Objective:** Implement one **pure, IO-free** function in the already-scaffolded
`crates/playback` crate that rewrites a prepared `.m3u8` so every media-segment
reference points at a caller-supplied routed base instead of a raw object-store key.
No storage, no network, no auth — fixtures only.
**Why delegable:** measured RRI **21** (`scripts/rri.py --touches crates/playback/src/lib.rs
--cc 6 --D 2 --K 0 --P 2 --T 2 --A 0 --X 1`). It is a single-file pure function with
deterministic fixtures — best-fit per `LOW_RRI_LOCAL_MODEL_HANDOFF.md`. It is the only
S-125 sub-part that clears the band with legitimately low D/K/P.
**Inputs:**
- Scaffolded `crates/playback` (from T3b): crate exists, builds, error type declared.
- A representative prepared `.m3u8` fixture + the malformed-HLS fixture already in the repo.
- The exact function signature, error variants, and rewrite rule, supplied verbatim in
  the delegation packet (Gemma is a mechanical transcriber, not the designer).
**Outputs:**
- `crates/playback/src/lib.rs`: `rewrite_manifest(manifest: &str, routed_base: &str)
  -> Result<String, ManifestRewriteError>` (pure), plus its unit tests.
**Acceptance criteria:**
- Zero IO and zero network calls; fully unit-testable from in-memory fixtures.
- Every media-segment URI in the output is prefixed/replaced by `routed_base`; no raw
  object-store key (no `s3://`, no bare key path) appears in any rewritten manifest.
- Malformed/empty/`#EXTM3U`-missing input returns `ManifestRewriteError`, emitting nothing.
- Order of segments is preserved; non-segment tag lines pass through unchanged.
- ≥90% line coverage for the function; no clippy warnings.
**Happy paths considered:**
- HP-1: valid prepared `.m3u8` + `routed_base` → manifest whose segment URIs are all
  prefixed with `routed_base`, order preserved.
- HP-2: playlist with N segments → all N references rewritten; `#EXTINF`/tag lines intact.
**Edge cases considered:**
- EC-1: malformed manifest (missing `#EXTM3U`) → `ManifestRewriteError`, nothing emitted.
- EC-2: no raw object-store key leaks into the output (assert absence of `s3://`/raw key).
**Files expected to change:** `crates/playback/src/lib.rs` only (single-file packet,
`--mode full-file`).
**Reflection strategy (orchestrator-applied, per `AGENT_WORKFLOW_GUIDE.md`):** Low-band
delegated work has no required pass count, but the orchestrator applies the Reflection
cycle to Gemma's output during the mandatory review: verify scope/format/tagged-block
contract, read the diff line by line, confirm no out-of-scope edits, run
`cargo build --workspace` + `cargo test -p dubbridge-playback` + clippy, and run at most
one bounded repair cycle before escalating. Record the reflection log in the final report,
not inside the delegated task. Gemma must not evaluate its own output.
**Delegation packet (orchestrator builds before sending):**
- Goal (1–2 sentences) + exact allowed file: `crates/playback/src/lib.rs`.
- What must NOT change: the crate's `Cargo.toml`, workspace wiring, error-type name.
- Output contract: tagged blocks with complete file content, `--mode full-file`.
- `## Current file content`: the scaffolded `lib.rs` verbatim.
- The literal function signature, `ManifestRewriteError` variants, and the rewrite rule.
- Both fixtures (valid + malformed) and the exact assertions to write.
- Sent via `scripts/delegate-low-rri.py`; the wrapper builds + checks the diff.
**Stop condition:** after the orchestrator review + verification pass; do not touch
`crates/storage` (that is T3b) and do not wire the API (T4/T5).

---

## S-125-T3b: Crate scaffold + storage scoped-reference helper
**Effort:** L (RRI 49 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T1 (grant context type)
**Status:** Not started
**Type:** development (retained by primary agent — sits on the no-raw-key boundary)
**Objective:** Scaffold the new `crates/playback` crate (workspace member, `Cargo.toml`,
empty `lib.rs` with the declared `ManifestRewriteError` type so T3a can fill the
function), and add the `crates/storage` helper that returns a scoped/expiring reference
for a prepared-HLS key without exposing raw object-store paths.
**Why retained, not delegated:** the storage helper sits on the no-raw-key boundary in
`crates/storage`, whose anchor-rubric floor is D3/P3/K3 (ADR-006/ADR-018). That is above
the Gemma band by construction; it is the security-relevant half of the original T3 and
must stay with the primary agent.
**Inputs:**
- Grant context type from T1.
- `crates/storage` canonical key conventions from `S-080`/`S-120`.
- The `ManifestRewriteError` contract that T3a will implement against.
**Outputs:**
- `crates/playback` scaffold: `Cargo.toml`, workspace member entry, `src/lib.rs` with the
  `ManifestRewriteError` type + `rewrite_manifest` stub signature (unimplemented or
  `todo!()`), so T3a delegates only the body + tests.
- `crates/storage` helper exposing a scoped/expiring reference (or read body) for a
  prepared-HLS key; clients never receive a raw key.
**Acceptance criteria:**
- `crates/playback` compiles as a workspace member with the error type + stub signature.
- The storage helper never returns a raw MinIO/S3 key to a caller; it yields a scoped/
  expiring reference or a streamed body only.
- Key construction stays inside `crates/storage`; no caller hand-rolls a prepared-HLS key.
- ≥90% coverage for the storage helper; no clippy warnings.
**Happy paths considered:**
- HP-1: request a scoped reference for a valid prepared-HLS key → scoped/expiring
  reference returned, raw key not exposed.
**Edge cases considered:**
- EC-1: unknown/absent prepared-HLS key → typed error, no fabricated reference.
- EC-2: the helper output contains no raw object-store path (assert no `s3://`/raw key).
**Files expected to change:** `crates/playback/Cargo.toml`; `crates/playback/src/lib.rs`
(scaffold + error type + stub); `Cargo.toml` (workspace member); `crates/storage/src/lib.rs`.
**Reflection strategy:** RRI 49 → Med-high → **3 passes**. Pass 1: scaffold compiles +
storage helper happy path (HP-1). Pass 2: no-raw-key non-leakage + unknown-key fail-closed
(EC-1/EC-2). Pass 3: key-ownership boundary (no key construction outside `crates/storage`)
+ coverage sweep.
**Agent handoff prompt:** Scaffold `crates/playback` (workspace member + `ManifestRewriteError`
+ `rewrite_manifest` stub) and add the `crates/storage` scoped-reference helper that never
exposes a raw key. Stop after the scaffold compiles and the storage helper tests are green;
this unblocks T3a (Gemma fills the rewriter body). Do not implement the rewriter body or
wire the API (T4/T5).

---

## S-125-T4: Grant issuance API + review authorization + durable grant audit
**Effort:** XL (RRI 88 — Very high; decomposition gate triggered)
**Recommended model:** Codex `GPT-5.2-Codex` (Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T2, S-125-T3a, S-125-T3b
**Status:** Not started
**Type:** development (public API surface, auth, audit)
**Objective:** Expose a grant-issuance endpoint behind the verified principal and the
`S-100`/`S-160` org/project authorization guard, issuing a scoped/expiring grant for a
`Ready` asset and emitting a durable, traceable grant audit row (ADR-018).
**Decomposition obligation:** RRI 88 (Very high) with T≥4, P≥4 → decomposition mandatory.
At approval, split into `T4a` (authorized issuance handler) / `T4b` (durable grant
audit + denial observability), or record an approved justification for proceeding whole.
Apply the Complex-band minimum of 4 Reflection passes to any 56+ subtask that proceeds.
**Inputs:**
- `playback_repo` (T2), pure rewriter (T3a) + storage helper (T3b).
- Verified principal (ADR-023) + `S-100`/`S-160` membership guard (ADR-027).
- Existing durable-audit emission seam (`crates/audit`/observability, ADR-018).
**Outputs:**
- `apps/api/src/playback_service.rs` + route: `POST /assets/{id}/playback-grants`
  (or equivalent), returning a grant reference, not a raw key.
- Authorization: deny unauthenticated/unauthorized callers fail-closed before issuance.
- A durable audit row for grant create and grant refusal, correlated by trace id.
**Acceptance criteria:**
- Authenticated + org/project-authorized caller for a `Ready` asset → grant issued; a
  durable audit row is written.
- Unauthenticated, cross-org, or unauthorized caller → denied before any grant/audit-as-
  success; the refusal is observable.
- Not-`Ready`/missing-lineage asset → issuance denied fail-closed (reuses T2 resolve).
- An audience-policy hook (F3) is present so `S-180` can attach the ADR-030 decision
  without changing the grant contract.
- ≥90% coverage for new handler logic; integration tests cover allow + each denial.
**Happy paths considered:**
- HP-1: authorized reviewer + `Ready` asset → grant issued + durable audit row written.
**Edge cases considered:**
- EC-1: unauthenticated caller → 401-class denial before issuance; no grant row.
- EC-2: authenticated but non-member of the asset's org/project → denied; refusal audited.
- EC-3: asset not `Ready` / no `HlsManifest` lineage → issuance denied fail-closed.
**Files expected to change:** `apps/api/src/playback_service.rs`; `apps/api/src/routes.rs`
(or router module); audit/observability emission; `apps/api/tests/playback_grant_test.rs`.
**Reflection strategy:** RRI 88 → **4 passes** (Complex-band minimum). Pass 1: authorized
happy path + grant issuance (HP-1). Pass 2: every denial path fails closed before side
effects (EC-1/EC-2/EC-3). Pass 3: durable audit + trace correlation on create and refuse
(ADR-018). Pass 4: audience-policy hook seam (F3) + coverage sweep. Decomposition into
T4a/T4b is the default before these passes.
**Agent handoff prompt:** Behind the verified principal + `S-100`/`S-160` guard, add the
grant-issuance handler that issues a scoped grant for a `Ready` asset and writes a durable
grant audit row. Decompose into T4a/T4b first unless waived. Stop after integration tests
green; do not implement segment delivery (T5).

---

## S-125-T5: Manifest + segment delivery, segment re-authorization, and docs/ADR sync
**Effort:** XL (RRI 76 — High; decomposition gate triggered)
**Recommended model:** Codex `GPT-5.2-Codex` (Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T4
**Status:** Not started
**Type:** development (delivery surface) + docs sync
**Objective:** Serve the rewritten `.m3u8` and its segments through the playback grant —
re-validating the grant (and readiness/expiry) on every segment request — with segment
trace/metrics (not per-segment durable audit), then flip ADR-032 to `Accepted` and sync
architecture/roadmap.
**Decomposition obligation:** RRI 76 (T≥4, P≥4) triggers the gate. At approval, split into
`T5a` (manifest+segment handlers) / `T5b` (docs/ADR-032-Accepted propagation) or justify
proceeding whole. Default proposal: split.
**Inputs:**
- Grant issuance + audit (T4), pure rewriter (T3a) + storage helper (T3b), `playback_repo` (T2).
**Outputs:**
- `GET` manifest handler: validates grant → reads prepared `.m3u8` via storage helper →
  returns the T3-rewritten manifest.
- `GET` segment handler: re-validates the same grant (and expiry/readiness) → streams or
  signs the segment; never serves a raw key.
- Segment-level trace/correlation + metrics; no per-segment durable audit row (ADR-032).
- ADR-032 → `Accepted` with full change-propagation; `docs/architecture.md` and roadmap
  `S-125` row updated to "done"; BDD mapping rows certified.
**Acceptance criteria:**
- Valid grant → manifest returned with backend-routed segment URLs only.
- Valid grant → each authorized segment served; expired/revoked/unauthorized grant →
  segment denied fail-closed even if the manifest was fetched earlier.
- Segment requests carry trace/correlation + metrics; no durable audit row per segment.
- ADR-032 status is `Accepted` in frontmatter + prose + index; architecture/roadmap and
  BDD mapping are consistent; `make qa-docs` green.
- ≥90% coverage for new handler logic.
**Happy paths considered:**
- HP-1: valid grant → manifest fetch returns rewritten `.m3u8`; each referenced segment
  fetch with the same grant succeeds and is traced.
**Edge cases considered:**
- EC-1: segment fetched with an expired grant (manifest fetched earlier) → denied
  fail-closed.
- EC-2: segment fetched with a grant scoped to a different asset/segment → denied; no
  raw key served.
- EC-3: asset transitioned out of `Ready` after grant issuance → segment delivery denies
  fail-closed.
**Files expected to change:** `apps/api/src/playback_service.rs`; `apps/api/src/routes.rs`;
`crates/storage/src/lib.rs` (stream/sign segment); `apps/api/tests/playback_delivery_test.rs`;
`docs/adr/ADR-032-hls-playback-delivery-boundary.md`; `docs/architecture.md`;
`docs/plan/roadmap.md`.
**Reflection strategy:** RRI 76 → **4 passes**. Pass 1: manifest+segment happy path (HP-1).
Pass 2: per-segment re-authorization fail-closed (EC-1/EC-2/EC-3) — a manifest is not a
durable permission. Pass 3: observability split correctness (trace/metrics vs durable
audit, ADR-018/032). Pass 4: ADR-032 propagation + architecture/roadmap/BDD sync + coverage.
**Agent handoff prompt:** Add manifest + segment delivery that re-validates the grant on
every segment, with trace/metrics only for segments. Then flip ADR-032 to Accepted and
sync architecture/roadmap/BDD. Decompose into T5a/T5b first unless waived. Stop after
integration tests + qa-docs green; this is the last S-125 task.

---

## Dependency graph

```text
T0 (plan/tasks/BDD/sync)
  └─> T1 (domain grant contract)
        ├─> T2 (migration + repo: grant -> readiness/lineage)        [decompose]
        └─> T3b (crate scaffold + storage scoped ref) ── primary agent
              └─> T3a (pure manifest rewriter) ── local Gemma delegation
                    T2, T3a, T3b ─> T4 (grant issuance API + authz + audit)  [decompose]
                                          └─> T5 (manifest+segment delivery + ADR/docs) [decompose]
```

T3b precedes T3a: Gemma fills only the rewriter body + tests in an already-scaffolded
crate; the primary agent owns the crate/workspace wiring and the storage boundary.

## Behavioral coverage contract

This ledger declares `Behavioral coverage contract: unit-v1`. Each development task
(`T1`, `T2`, `T3a`, `T3b`, `T4`, `T5`) must, at closure, map every `HP-#`/`EC-#` to a
unit/integration test reference and record an `Owner final verification`, per
`AGENT_WORKFLOW_GUIDE.md`. For T3a (local Gemma delegation), the orchestrator records
the reflection log and verification evidence in the final report; Gemma does not certify
its own work.
