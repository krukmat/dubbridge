---
type: TaskList
title: "S-230 POC v1 Deployment (Digital Ocean)"
status: planned
slice: S-230
plan: docs/plan/s-230-poc-v1-digitalocean.md
---
# S-230 — POC v1 Deployment (Digital Ocean)

Ordered task ledger for the 10-day POC deployment window. Rationale, verified gap
analysis, architecture, calendar, and risks live in
`docs/plan/s-230-poc-v1-digitalocean.md`; this file is the crash-safe progress
ledger.

## Slice execution contract

- Product scope: subtitles + governed review/publication, plus (owner scope
  amendment, 2026-08-16, second pass) text-only cross-language subtitle
  translation through the `S-150-T2c-v -> T2c-vi-a -> T2c-vi-b -> T3a -> T3b ->
  T3c` chain, tracked as `S-230-T3b`. This reverses the original freeze after
  the owner reviewed and explicitly overrode the plan's own recorded
  recommendation against it (`docs/plan/s-230-poc-v1-digitalocean.md` § "The
  market-audience gap, examined"). TTS/dubbed audio (`S-150-T4` through `T7`)
  remains out of scope and blocked on ADR-028; it is not reopened by any task
  here. `S-150-T2c-v` also carries its own separate, still-unresolved
  "Redis-topic decision" parking note (`docs/plan/s-150-translation-dubbing.md`
  line 31) — this scope amendment authorizes presenting/implementing the chain
  under S-230, it does not itself resolve that separate parking condition;
  confirm it with the owner before `S-230-T3b`'s first child task starts.
- No new application technology. PostgreSQL, Redis, S3-compatible storage,
  ffmpeg, and Python/faster-whisper are all pre-existing dependencies.
- Every RRI must be computed with `scripts/rri.py` before the task is presented;
  the provisional efforts below are planning estimates, not scores.
- T1, T1b, T2, T3, T7b, T7c and T8 are development tasks and carry the full
  closure checklist (band-routed review, Reflection where the band requires it,
  unit coverage certification, owner verification). T4, T5, T6 are
  config/ops-shaped; their exemption status is decided per task at presentation
  time, not assumed here. T3b is a non-executable parent — each of its six
  children carries its own full closure checklist under the S-150 ledger; T3b
  itself is never implemented or marked Done directly.
- Tasks touching `mobile/` (T7, T7b, T7c) must read the root `DESIGN.md` before
  planning or implementation, per the workflow guide's Analyze step.
- Task order is a dependency order, not a suggestion. T4 must not start while any
  of T1–T3 is open. **T3b is the one exception to strict table-position
  ordering:** despite sitting between T3 and T4 in the index below, it is an
  independent parallel track (`Depends on: T0` only) and is explicitly excluded
  from T4's gate — T4 does not wait on it. T4/T5/T6 each carry a conditional
  (non-blocking) scope addition if T3b's children finish before they run; see
  each task's own card and `S-230-T3b` §"Downstream coupling."

## Task index

| ID | Title | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| T0 | Slice plan, ledger, and roadmap entry | docs-only | S | — | [x] Done |
| T1 | S3/Spaces credential and region wiring | development | M | T0 | [x] Done |
| T1b | API preparation queue bound to Redis | development | M (recomputed, RRI 35 Moderate) | T0 | [x] Done |
| T2 | Migration runner in the production path | development | M (recomputed, RRI 28 Moderate) | T0 | [x] Done |
| T3 | Real readiness probes for api and gateway | development | M | T0 | [x] Done 2026-08-17 |
| T3b | Cross-language subtitle translation pipeline (S-150 reopening) | development parent | XL | T0 | [ ] Planned — approval pending per child |
| T4 | Production container images (non-executable parent) | development parent | 17 Low/S children | T1, T1b, T2, T3 | [ ] Decomposed — execute T4a–T4q |
| T4a | Production-image test harness | development/test | S (RRI 15 Low) | T3 | [x] Done |
| T4b | API production image | development/config | S (RRI 18 Low) | T4a | [x] Done (structural cert) |
| T4c | API image contract tests | development/test | S (RRI 17 Low) | T4b | [x] Done |
| T4d | Gateway production image | development/config | S (RRI 18 Low) | T4a | [x] Done — 2026-08-17 |
| T4e | Gateway image contract tests | development/test | S (RRI 21 Low) | T4d | [x] Done |
| T4f | Migration production image | development/config | S (RRI 13 Low) | T4a, T2 | [x] Done — 2026-08-20 |
| T4g | Migration image contract tests | development/test | S (RRI 20 Low) | T4f | [x] Done — 2026-08-20 |
| T4h | Exact ASR dependency lock | development/config | S (RRI 7 Low) | T4a | [x] Done — 2026-08-20 |
| T4i | Worker native-runtime image | development/config | S (RRI 13 Low) | T4a, T4h | [x] Done — 2026-08-20 |
| T4j | Worker native-runtime contract tests | development/test | S (RRI 13 Low) | T4i | [x] Done — 2026-08-20 |
| T4k | Worker ASR-bundle image | development/config | S (RRI 13 Low) | T4j | [x] Done — 2026-08-21 |
| T4l | Worker ASR-bundle contract tests | development/test | S (RRI 24 Low) | T4k | [ ] Planned |
| T4m | Translation-bundle image (conditional) | development/config | S (RRI 21 Low) | T4l, T3b | [ ] Conditional |
| T4n | Translation-bundle contract tests (conditional) | development/test | S (RRI 24 Low) | T4m | [ ] Conditional |
| T4o | Full local image-pipeline contract | development/test | S (RRI 25 Low) | T4c, T4e, T4g, T4l; T4n if executed (contract-verified images) | [ ] Planned |
| T4p | Execute and record local image evidence | operational/evidence | S (RRI 19 Low) | T4o | [ ] Planned |
| T4q | T4 parent closeout and status sync | docs-only | S (RRI 10 Low) | T4p; T4n if executed | [ ] Planned |
| T5 | Production deployment descriptor and secret boundary | config-only | M | T4q | [ ] Planned |
| T6 | First deploy and end-to-end smoke on Digital Ocean | operational | L | T5 | [ ] Planned |
| T7 | Mobile POC build against the deployed backend | development/ops | M | T6 | [ ] Planned |
| T7b | Mobile registration screen | development | M | T7 | [ ] Planned — droppable (first) |
| T7c | Session lifetime and expiry behavior | development/config | S | T7 | [ ] Planned |
| T8 | Subtitle visible in the review surface (optional) | development | M | T6 | [ ] Planned — droppable (second) |
| T8b | Translated subtitle visible in the review surface | development | M | T3b, T8 | [ ] Planned — double-conditional |
| T9 | Status, README, and debt-register closeout | docs-only | S | T7, plus each of T7b / T7c / T8 / T8b / T3b that was executed | [ ] Planned |

---

## S-230-T0: Slice plan, ledger, and roadmap entry

**Type:** docs-only
**Effort:** S
**Depends on:** —
**Status:** [x] Done 2026-08-16

**Acceptance criteria:**

- [x] `docs/plan/s-230-poc-v1-digitalocean.md` records objective, frozen scope,
      the verified gap analysis with file-level evidence, target architecture,
      ten-day sequence, risks, and explicit out-of-scope boundary.
- [x] This ledger exists with ordered tasks, dependencies, per-task acceptance
      criteria, and HP/EC cases for every development task.
- [x] `docs/plan/roadmap.md` carries an `S-230` row and records that `S-150` is
      parked for the POC window.

**Evidence to emit:** the two documents; `make qa-docs` (passed 2026-08-16:
doc consistency, task coverage, roadmap drift, OKF frontmatter all green).

**Status artifacts affected:** `docs/plan/roadmap.md`.

---

## S-230-T1: S3/Spaces credential and region wiring

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [x] Done 2026-08-16

**Task-analysis review:** gemma `docs/audit/gemma-evidence/s-230-t1-phase1.json` - PASS
**Code-solution review:** gemma `docs/audit/gemma-evidence/s-230-t1-cloud-tramo.json` - FINDINGS (3 rejected as false positive, 1 accepted-follow-up; see evidence below)
(note: ensure `AppConfig::validate()` checks schema validity of `endpoint`/`region`,
not only presence, to fully satisfy EC-1 — carried into the cloud-tramo handoff
prompt below.)

**Problem (plan G1):** `crates/storage/src/s3.rs:17` uses
`AmazonS3Builder::new()`, which reads no environment. Credentials fall through to
AWS instance-metadata providers that do not exist on Digital Ocean, and the
region silently defaults to `us-east-1`.

**Happy paths considered:**

- **HP-1:** With `storage.backend = "s3"`, a Spaces endpoint, an explicit region,
  and injected credentials, the adapter is built with a static credential
  provider and a `put` followed by `get` against a real Spaces bucket round-trips
  identical bytes.
- **HP-2:** The existing local MinIO path keeps working unchanged with the same
  configuration shape.

**Edge cases considered:**

- **EC-1:** `backend = "s3"` in a production-like environment with missing
  credentials or missing region fails `AppConfig::validate()` at startup, before
  any request is served — not on the first write.
- **EC-2:** Credentials never appear in configuration files, logs, or traces;
  only injected `DUBBRIDGE_*` environment variables carry them (ADR-026
  Decision 4, ADR-018 redaction).

**Acceptance criteria:**

- `StorageSettings` carries region and credential references; secrets are
  env-only and absent from `config/*.toml`.
- `S3Adapter::new` sets bucket, endpoint, region, and static credentials
  explicitly.
- Production validation rejects an `s3` backend that is missing endpoint, region,
  or credentials.
- A real round-trip against an S3-compatible endpoint is executed and recorded —
  a unit test alone does not satisfy HP-1.

**Files expected to change:** `crates/config/src/lib.rs`,
`crates/storage/src/config.rs`, `crates/storage/src/s3.rs`,
`crates/storage/src/lib.rs`, `config/production.toml`, `config/staging.toml`.
Recompute the exact list before presentation.

**RRI:** 43 (Med-high, 41–55). `--auto-cc` measured CC=1 across all four
touched `.rs` files (zero `clippy::cognitive_complexity` warnings), which
in isolation would put `C`'s contribution at a Moderate-range 37. Owner
directive: the anchor-rubric floors on `D`/`K`/`P` (ADR-006, ADR-018 —
`crates/storage` touches immutable-artifact and durable-audit invariants)
are kept at 3 regardless of the low measured CC, so the task stays Med-high.
Do not silently re-derive the band from `--auto-cc` alone without an
explicit re-approval — the floor, not the CC measurement, is what governs
here.

### Module-split routing evidence (ADR-040) — authoritative

**Owner directive, 2026-08-16: split adopted as the authoritative
implementation routing for this task, superseding whole-task Med-high
cloud-only routing.**

- **Trigger check:** `allowed_paths` spans 6 files (≥2, satisfied).
  Heterogeneity: `config/production.toml` and `config/staging.toml` carry no
  Rust logic (not clippy-measurable, effectively C≤1 by inspection — pure
  key/value additions); `crates/storage/src/s3.rs`,
  `crates/storage/src/config.rs`, `crates/storage/src/lib.rs`, and
  `crates/config/src/lib.rs` are the ADR-006/ADR-018 anchor-rubric-floored
  Rust surface. Treated as heterogeneous by file kind and domain floor, not
  by `--auto-cc` score (which read C=1 uniformly — see RRI note above).
- **Hard domain exclusion (ADR-038 §6):** all four `.rs` files touch
  credential handling, endpoint/region wiring, and fail-closed production
  validation — auth/credential and governance-invariant surface. Cloud-only
  regardless of measured CC.
- **Disjoint partition:**
  - **Local tramo (Low-band, RRI 0–25):** `config/production.toml`,
    `config/staging.toml`. Scope: add `endpoint` and `region` keys under
    `[storage]` in both files (non-secret values only — no credentials, per
    ADR-026 Decision 4). Route: `scripts/delegate-low-rri.py --mode
    before-after`, one delegation packet per file. Repair budget: 1 bounded
    Qwen Developer repair cycle per § Local delegation (RRI 0–25); escalate to the
    orchestrator only under the documented tooling-failure exception.
  - **Cloud tramo (Med-high, ADR-038):** `crates/config/src/lib.rs`,
    `crates/storage/src/config.rs`, `crates/storage/src/s3.rs`,
    `crates/storage/src/lib.rs`. Scope: `StorageSettings` region/credential
    fields, `S3Adapter::new` explicit endpoint/region/static-credential
    wiring, `AppConfig::validate()` fail-closed rejection of an incomplete
    `s3` backend. Route: Muse Glimmer advisory refinement →
    `med_high_gate.py` hash-bound receipt → cloud-takeover model per the
    approved task card (full ADR-038 §5 evidence bundle).
- **Interface freeze:** the cloud tramo owns the `endpoint_url`/region field
  names and shapes on `StorageSettings`/`StorageConfig`; the local tramo's
  `.toml` keys (`endpoint`, `region` under `[storage]`) must match those
  names exactly. Freeze the field names before dispatching either tramo —
  the local tramo's packet must state them literally, not infer them.
- **Integration gate:** run T1's full acceptance criteria (round-trip
  against a real S3-compatible endpoint, production validation test) against
  the merged diff before Reflection. A `.toml`-tramo failure repairs within
  its own 1-attempt Low-band budget; an `.rs`-tramo failure repairs within
  Med-high's normal (zero-whole-task-repair) ADR-038 gate; an interface
  mismatch (field name drift) abandons the split and re-routes the whole
  task through whole-task Med-high cloud-only implementation.
- **Review/approval unaffected:** phase-1/phase-2 Gemma review, 3 Reflection
  passes, and the RRI 41+ human approval gate below all evaluate the final
  merged diff as one task, per ADR-040.

### Cloud-tramo ADR-038 refinement — resolved 2026-08-16

- **Muse Glimmer advisory refinement:** `route_recommendation: CLOUD_REQUIRED`.
  Model `muse-glimmer:30b-q4_K_M`, digest
  `de878ce33ad81d060001db1469a02eebe4d86f0ad58cfe52dc062fdcbe4464c1`. Rationale:
  all four `.rs` files match the ADR-038 §6 hard domain exclusion (credential
  handling, endpoint/region wiring, fail-closed production validation) —
  cloud-only regardless of measured complexity. Two `unknowns` flagged for the
  cloud implementer to resolve before/during implementation: (1) exact
  `DUBBRIDGE_*` env var names for the S3 credentials are not yet fixed; (2) the
  precise schema-validity rules for endpoint/region/credentials (beyond mere
  presence) are not yet fully defined — the cloud implementer must define both
  as part of this task, consistent with Gemma's phase-1 note.
  Artifact: `docs/audit/med-high/s-230-t1-refinement-artifact.json`.
- **Primary route receipt:** `claude-sonnet-5-orchestrator` independently
  concurs `CLOUD_REQUIRED` (a receipt may only downgrade GO_LOCAL to cloud,
  never upgrade CLOUD_REQUIRED to local — moot here since both sides already
  agree). Artifact: `docs/audit/med-high/s-230-t1-primary-receipt.json`.
- **Gate decision (`med_high_gate.py`):** `route: CLOUD_REQUIRED` — both
  inputs validated (hash-bound to packet `295c0488671a4f0ce9f5dd386c7ef882153f4e09aefe1aa48d5d2420005da678`),
  RRI 43 confirmed in-band. Artifact:
  `docs/audit/med-high/s-230-t1-gate-decision.json`. Task packet:
  `docs/audit/med-high/s-230-t1-packet.json`.
- **Refined scope/steps/acceptance tests/stop conditions from the artifact**
  are the binding cloud-tramo implementation contract, superseding the
  shorter "Handoff prompt (cloud tramo)" bullet below where more specific:
  extend `StorageConfig`/`StorageSettings` with region + credential
  references; update `S3Adapter::new` to set bucket/endpoint/region/static
  credentials explicitly; strengthen `AppConfig::validate()` for both
  presence and schema validity; add and record a real round-trip
  put-then-get test against an S3-compatible endpoint; do not touch
  `config/production.toml` or `config/staging.toml`, upload path, key
  layout, or any task beyond S-230-T1.
- **Local tramo unaffected:** this refinement covers only the cloud tramo.
  The local tramo (`config/production.toml`, `config/staging.toml`) remains
  routed as Low-band delegation per the split above and has not yet been
  dispatched.

### Local-tramo delegation evidence — completed 2026-08-16

- **`config/production.toml`, attempt 1 — REJECTED (out-of-scope diff):**
  `qwen3.8:27b-mlx` via `delegate-low-rri.py --mode before-after` correctly
  added `endpoint`/`region` but also silently rewrote `base_path` (`""` →
  `"/data"`) and `bucket` (`"dubbridge-production"` → `"prod-assets"`),
  violating the explicit "do not change" constraint. Applied diff was
  reverted with `git checkout -- config/production.toml` before any commit.
  Artifact: `docs/audit/low-rri/s-230-t1-prod-attempt1-failed.json`.
- **`config/production.toml`, attempt 2 — APPLIED (repair, 1/1 budget
  used):** re-delegated with a stricter packet showing the exact required
  AFTER block verbatim and calling out the prior failure explicitly. Result:
  `base_path`/`bucket` preserved exactly, only `endpoint =
  "https://nyc3.digitaloceanspaces.com"` and `region = "nyc3"` added.
  Verified in scope via `git diff`. Artifact:
  `docs/audit/low-rri/s-230-t1-prod-attempt2-applied.json`.
- **`config/staging.toml`, attempt 1 — APPLIED:** delegated directly with the
  same strict verbatim-AFTER-block packet style (informed by the production
  repair). Correct on the first attempt: `backend`/`base_path`/`bucket`
  preserved, `endpoint`/`region` added with the same placeholder values.
  Artifact: `docs/audit/low-rri/s-230-t1-staging-attempt1-applied.json`.
- **Values used (non-secret placeholders, both files):** `endpoint =
  "https://nyc3.digitaloceanspaces.com"`, `region = "nyc3"`. These are
  DigitalOcean Spaces placeholder values, not yet confirmed against the
  actual POC infrastructure region — the cloud-tramo implementer and/or T4-T6
  operational tasks must confirm or replace them with the real target region
  before production use.
- **No credentials were added to either file** (EC-2 preserved).
- **Field names match the cloud-tramo interface freeze exactly**
  (`endpoint`, `region`), confirmed against
  `docs/audit/med-high/s-230-t1-refinement-artifact.json`.

**Evidence to emit:** RRI report, this module-split routing block, phase-1
and phase-2 review artifacts, round-trip evidence against a real endpoint,
Reflection log, unit coverage certification, owner verification.

**Status artifacts affected:** this ledger; roadmap X9 wording if the storage
contract changes materially.

**Handoff prompt (cloud tramo):** Wire explicit S3 credentials and region
through config into `S3Adapter`, fail closed in production when any are
absent, and prove one real round-trip. Do not touch the upload path or key
layout. Consume `endpoint`/`region` key names exactly as frozen in the
interface-freeze note above — the `.toml` tramo is dispatched separately.
Per Gemma's phase-1 note: `AppConfig::validate()` must reject not only a
missing `endpoint`/`region`, but also a malformed/empty value (schema
validity, not just presence), to fully satisfy EC-1.

**Handoff prompt (local tramo):** Add `endpoint` and `region` keys under
`[storage]` in `config/production.toml` and `config/staging.toml`, using the
non-secret placeholder values appropriate to each environment. No
credentials. Field names are frozen — do not rename.

**Stop condition:** Stop after the round-trip evidence on the merged diff.
Do not start T2.

### Cloud-tramo implementation evidence — completed 2026-08-16

Implemented per the ADR-038 refinement artifact's binding contract
(`docs/audit/med-high/s-230-t1-refinement-artifact.json`).

**Files changed:**
- `crates/config/src/lib.rs` — `StorageSettings` gains `region`,
  `access_key_id`, `secret_access_key` (`Option<String>`, env-only);
  `#[serde(alias = "endpoint")]` on the pre-existing `endpoint_url` field to
  reconcile the frozen TOML key name with the actual Rust field name (see
  interface-mismatch fix below); `AppConfig::validate()` calls
  `StorageSettings::validate_s3_production()` when `backend == S3`, itself
  gated by the pre-existing `production_like` early-return; `from_env()`
  reads the 3 new fields from single-underscore env vars (legacy reader,
  unaffected production path). `Default` derived on `StorageBackend`
  (`#[default] LocalFs`) and `StorageSettings`.
- `crates/storage/src/config.rs` — `StorageConfig` mirrors the same 3 fields
  and `Default`; `From<&StorageSettings>` copies them.
- `crates/storage/src/s3.rs` — `S3Adapter::new` wires `region`,
  `access_key_id`, `secret_access_key` explicitly into `AmazonS3Builder`
  alongside the pre-existing `endpoint`/`allow_http`.
- `crates/storage/src/lib.rs` — test-fixture updates only (no production
  logic; this crate doesn't construct `StorageSettings`).

**Interface-mismatch fix (not anticipated in the refinement artifact):** the
frozen contract named the TOML key `endpoint` and implied a matching Rust
field name, but the pre-existing public field was `endpoint_url` (consumed
by `S3Adapter::new` and other callers). Resolved via `#[serde(alias =
"endpoint")]` rather than reopening the already-evidence-recorded local
tramo or renaming a public field mid-task. Verified via the two tests that
load the real `config/staging.toml`/`production.toml`
(`app_config_load_staging_profile_reads_staging_toml_values`,
`app_config_validate_production_profile_with_representative_secrets_passes`).

**Maintainability fix (owner feedback mid-task):** the first implementation
pass repeated the full 7-field `StorageSettings`/`StorageConfig` struct
literal across ~13 test-fixture call sites in `apps/api`, `apps/gateway`,
and 3 gateway integration-test files. Flagged as copy-paste by the task
owner; resolved by deriving `Default` on `StorageBackend`/`StorageSettings`/
`StorageConfig` and reducing every fixture to its materially-relevant fields
plus `..Default::default()`.

**HP-1 real round-trip:** `crates/storage/src/s3.rs::s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint`
(`#[ignore]`-gated, following the `crates/jobs` Redis-integration pattern),
run explicitly against local MinIO
(`infra/local/docker-compose.yml`, service `minio`):

```
DUBBRIDGE_STORAGE_TEST_ENDPOINT="http://localhost:9000" \
DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID="dubbridge" \
DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY="dubbridge123" \
DUBBRIDGE_STORAGE_TEST_BUCKET="dubbridge-local" \
cargo test -p dubbridge-storage --lib s3::tests::s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint -- --ignored --nocapture
```

Result: `ok` — `put` then `get` round-tripped identical bytes against a real
S3-compatible endpoint, then cleanup `delete`. Re-confirmed against the
final merged diff (both tramos applied) during the integration gate below,
same result.

### ADR-040 integration gate — passed 2026-08-16

Run against the merged diff of both tramos (local `.toml` + cloud `.rs`),
per the module-split contract's mandatory integration-gate step, before
Reflection:

| Check | Command | Result |
|---|---|---|
| Workspace build | `cargo build --workspace --all-features` | clean |
| Full test suite | `cargo test --workspace --all-features` | 0 failed across all crates (all `test result: ok`) |
| Real S3 round-trip (HP-1) | see command above | `ok`, 1 passed |
| Format | `cargo fmt --check` | clean, exit 0 |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean, exit 0 (only pre-existing unrelated `apalis-redis` future-incompat warning) |

No tramo-attributable or interface-attributable failure occurred — the split
is not abandoned and the whole task did not re-route to whole-task
cloud-only implementation.

### Reflection log

Required passes: 3 (`43` → `Med-high`)

#### Pass 1 — Contract correctness

- **Draft verdict:** region + static credentials wired into `S3Adapter::new`;
  `AppConfig::validate()` fail-closed for `backend = s3`.
- **Critique findings:** EC-1 needs schema validity, not just presence
  (Gemma phase-1 note); interface-freeze `endpoint`/`endpoint_url` mismatch
  against the completed local tramo; HP-1 requires a real round-trip, not a
  unit test alone.
- **Revisions applied:** added `validate_s3_production()` (blank/malformed
  rejection on all 4 fields); added `#[serde(alias = "endpoint")]`; added and
  ran the `#[ignore]`-gated real MinIO round-trip test.

#### Pass 2 — Failure boundaries and side effects

- **Draft verdict:** S3 validation branch is additive and gated; `LocalFs`
  path untouched; no credential values in `config/*.toml`.
- **Critique findings:** confirmed via grep that no credential value exists
  in `config/production.toml`/`config/staging.toml` (EC-2); confirmed the
  3-field widening of `StorageSettings`/`StorageConfig` didn't silently break
  ~13 downstream test-fixture sites (caught via iterative
  `cargo build`/`cargo test --workspace` compile-error cycles, all resolved).
- **Revisions applied:** none beyond Pass 1 — the `Default`-based fixture
  pattern from the maintainability fix already removes the main
  silent-staleness risk in copy-pasted literals.

#### Pass 3 — Coverage and maintainability

- **Draft verdict:** 8 new `AppConfig::validate()` unit tests (one per
  rejection branch plus the accept case), 3 new `StorageConfig::from`
  tests, 1 real round-trip integration test.
- **Critique findings:** verified every branch of
  `validate_s3_production()` has a dedicated test; confirmed the
  copy-paste pattern is fully remediated workspace-wide (`Default` derive +
  `..Default::default()` at all 13 touched call sites).
- **Revisions applied:** none — satisfied by Pass 1/Pass 2 work.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `REVIEW_PATHS="crates/config/src/lib.rs crates/storage/src/config.rs crates/storage/src/s3.rs crates/storage/src/lib.rs" GEMMA_REVIEW_TASK_ID="s-230-t1-cloud-tramo" make qa-gemma-review`
- Artifact: `docs/audit/gemma-evidence/s-230-t1-cloud-tramo.json`
- Verdict: `FINDINGS` (2/3 passes usable; aggregate `status: findings`)
- Findings: 4 — all verified against current source before disposition:
  1. `crates/config/src/lib.rs:212` major — "`validate_s3_production` runs
     without checking `production_like`." **Rejected (false positive):**
     `validate()` line 188-190 has an unconditional early-return on
     `!is_production_like()` before line 212 is reachable.
  2. `crates/config/src/lib.rs:262` major — "`from_env` env var names
     mismatch Figment's `__` convention." **Rejected (false positive):**
     `from_env()` is the pre-existing, explicitly-documented legacy reader
     ("do not add new callers — use load() instead"); `load()` is the real
     production path and correctly uses the `__` convention.
  3. `crates/config/src/lib.rs:78` minor — "`StorageSettings` derives
     `Debug` and holds `secret_access_key`; leak risk via `{:?}`."
     **Accepted-follow-up:** no current call site logs it (verified by
     grep), and it matches the pre-existing unredacted-secret pattern
     already on `AuthSettings::jwt_secret`/`GatewayOAuthSettings::client_secret`
     in the same file — a redaction pass is cross-cutting, deferred to the
     T9 debt register rather than fixed in this task's scope.
  4. `crates/config/src/lib.rs:262` minor — "`from_env` omits explicit
     `backend`/`base_path`, relies on `Default`." **Rejected (false
     positive):** direct read shows `from_env()` sets both explicitly
     (lines 259-263); it does not use `..Default::default()`.
- Muse Glimmer fallback: not triggered — reason: Gemma produced a usable
  aggregate (2/3 parseable passes)
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a — reason: n/a
- disposition_divergence: `none`
- Primary-agent disposition: 3 rejected as false positives (cited against
  current source line-by-line), 1 accepted as a follow-up debt item
  (recorded in the T9 debt register above)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | explicit endpoint/region/credentials build a static-credential S3 adapter; real put+get round-trips identical bytes against an S3-compatible endpoint | `crates/storage/src/s3.rs::s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint` (run explicitly, `--ignored`) | passed |
| HP-2 | Happy path | local MinIO path keeps working unchanged with the same config shape | `crates/storage/src/lib.rs::build_adapter_s3_returns_s3_adapter` (existing S3-shape construction test, unaffected by the field additions) | passed |
| EC-1 | Edge case | `backend = s3` in production with missing/blank/malformed endpoint, region, or credentials fails `AppConfig::validate()` before any request | `crates/config/src/lib.rs::app_config_validate_rejects_s3_backend_missing_endpoint_in_production`, `::_malformed_endpoint_in_production`, `::_blank_endpoint_in_production`, `::_missing_region_in_production`, `::_blank_region_in_production`, `::_missing_access_key_id_in_production`, `::_missing_secret_access_key_in_production`, and the accept case `::app_config_validate_accepts_well_formed_s3_backend_in_production` | passed |
| EC-2 | Edge case | credentials never appear in `config/*.toml`; only `DUBBRIDGE_*` env vars carry them | `crates/config/src/lib.rs::app_config_load_staging_profile_reads_staging_toml_values` and `::app_config_validate_production_profile_with_representative_secrets_passes` (both inject credentials only via `DUBBRIDGE_STORAGE__ACCESS_KEY_ID`/`_SECRET_ACCESS_KEY` env vars, never via TOML); confirmed by direct read that `config/production.toml`/`config/staging.toml` carry no `access_key`/`secret` key | passed |

### Owner final verification

- Owner: `matias`
- Date: 2026-08-16
- Statement: I verified every happy path and edge case defined for this task
  has unit or integration test evidence that replicates the expected
  behavior, including a real (non-mocked) round-trip against an
  S3-compatible endpoint for HP-1, and reviewed the Gemma phase-2 findings
  disposition against the current source before accepting it.
- Commands run:
  ```
  cargo build --workspace --all-features
  cargo test --workspace --all-features
  DUBBRIDGE_STORAGE_TEST_ENDPOINT="http://localhost:9000" DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID="dubbridge" DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY="dubbridge123" DUBBRIDGE_STORAGE_TEST_BUCKET="dubbridge-local" cargo test -p dubbridge-storage --lib s3::tests::s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint -- --ignored --nocapture
  cargo fmt --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  REVIEW_PATHS="crates/config/src/lib.rs crates/storage/src/config.rs crates/storage/src/s3.rs crates/storage/src/lib.rs" GEMMA_REVIEW_TASK_ID="s-230-t1-cloud-tramo" make qa-gemma-review
  ```

**Status: [x] Done 2026-08-16**

---

## S-230-T1b: API preparation queue bound to Redis

**Type:** development
**Effort:** M (recomputed and confirmed Moderate — see RRI note below; the
provisional M estimate was briefly superseded by a mis-scored Med-high draft,
then corrected back to Moderate)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

> The `T1b` suffix marks a task inserted on 2026-08-16 by the
> S-070/S-090/S-095/S-150 coverage review, after T0–T9 were already published in
> the roadmap row. The suffix keeps T2–T9 stable rather than renumbering them.

**Problem (plan G10):** `apps/api/src/main.rs:29` builds state with
`AppState::with_auth_service(..)`, which hardcodes
`preparation_queue: Arc::new(InMemoryPreparationJobQueue::default())`
(`apps/api/src/state.rs:31`, `:49`, `:67`). That queue is a
`Mutex<Vec<PreparationJob>>` (`crates/jobs/src/lib.rs:72`–`94`). The
worker-runner consumes from Redis (`apps/worker-runner/src/main.rs:76`–`88`), and
`apps/api` never opens a Redis connection at all.

The result is a **silent** no-op: finalize returns success, the job is pushed onto
an in-process vector, and no preparation, transcription, subtitle or review work
ever happens. Every probe stays green. Without this task the deployed POC accepts
uploads and produces nothing.

`AppState::with_preparation_queue` (`apps/api/src/state.rs:75`) already accepts an
injected queue but is unused and forces `auth_service: None`, so it cannot
currently serve an authenticated API.

**Happy paths considered:**

- **HP-1:** With a reachable `redis_url`, `POST /ingest/{token}/finalize` enqueues
  a `PreparationJob` onto the same Redis namespace the worker-runner's
  `RedisPreparationJobQueue` consumes, and the worker observes and executes it.
- **HP-2:** The API state carries both a real authenticated auth service and an
  injected queue at once — the current constructor split does not force choosing
  one.

**Edge cases considered:**

- **EC-1:** When Redis is unreachable, finalize fails closed exactly as the
  existing handler already specifies: `preparation_status` is written as `Failed`
  with the enqueue detail, the error is logged, and the response is 500
  (`apps/api/src/routes/ingestion.rs:331`–`347`). It must never report success on
  a dropped job.
- **EC-2:** Tests that rely on `InMemoryPreparationJobQueue` for assertions keep
  working — the in-memory implementation stays available as a test double and is
  only removed from the production startup path.
- **EC-3:** The Redis connection is established during startup, so a misconfigured
  `redis_url` surfaces at boot rather than on the first upload.

**Acceptance criteria:**

- `apps/api` constructs a `RedisPreparationJobQueue` from `config.redis_url` at
  startup and injects it into `AppState`.
- A single constructor carries both `auth_service` and an injected
  `preparation_queue`; the production path uses it.
- `InMemoryPreparationJobQueue` is no longer reachable from any binary's startup
  path, and a test proves the production constructor does not select it.
- An integration-level test proves a job enqueued through the API's configured
  queue is visible to a Redis-backed consumer, not only that `enqueue` returned
  `Ok`.

**Files expected to change:** `apps/api/src/state.rs`, `apps/api/src/main.rs`,
and possibly `crates/jobs/src/lib.rs` for a shared constructor seam. Recompute the
exact list before presentation.

**RRI:** 35 (Moderate, 26–40). `--auto-cc` measured CC=1 across both
currently-known touched files (no cognitive-complexity warnings).

An earlier draft of this note scored RRI 41 (Med-high) by raising D and K to
4 using the RRI policy's general scoring-band language for "async
orchestration" / "queues". That was inconsistent with this ledger's own
precedent: `S-230-T1`'s recorded RRI note (above) hit the identical
D/K-floor-vs-general-band ambiguity for `crates/storage` — the anchor
rubric's `crates/jobs`/`crates/storage`/... row literally contains the phrase
"async orchestration" at floor D/K/P=3, while the general D band separately
lists "async orchestration" at D=4 — and explicitly resolved it by **keeping
the floor** ("the floor, not the CC measurement, is what governs here").
Recomputed consistently with that precedent:

- **D=3, K=3, P=3** — anchor-rubric floor for the `crates/jobs`/async-
  orchestration row (ADR-006, ADR-018), not raised, matching T1's approach.
  P is not raised toward 4/5: this task changes an internal constructor
  signature, not a public route, `apps/gateway/src/auth/**`, `crates/auth`,
  or a rights/audit path.
- **T=2** ("partial tests exist"), grounded in actual test evidence rather
  than guessed: `apps/api/tests/ingestion_test.rs:150` already has a working
  harness that constructs `AppState::with_preparation_queue` with an
  observable `InMemoryPreparationJobQueue` and exercises the finalize
  enqueue path end to end. The area is not untested — what's net-new and
  uncovered is specifically the Redis-backed construction and the
  consolidated (`auth_service` + injected queue) constructor. An earlier
  pass under-evidenced this as "T=2 apps/main.rs has unrelated auth tests
  only" without checking `ingestion_test.rs`; checking it changes nothing
  about the final T value but grounds it in evidence instead of a guess.
- **A=1** (task has explicit HP/EC and acceptance criteria in this ledger).
- **X=2** (2 files: `apps/api/src/state.rs` + `apps/api/src/main.rs`).

This reading is not on a band boundary (35 sits mid-Moderate, not adjacent to
41), so it is more stable than the earlier 41 call, which pivoted on a single
un-evidenced point (T=3 vs T=4 would have flipped it either way). Recorded
plainly as a correction, not silently overwritten, since the earlier number
was already shown to you.

**Evidence to emit:** RRI report, phase-1 and phase-2 Gemma review artifacts,
evidence that a job enqueued through the API's configured queue is visible to
a Redis-backed consumer, 2-pass Reflection log, unit coverage certification,
owner verification. (Corrected from an earlier draft of this line that
carried over ADR-038/Muse Glimmer/3-pass language from the mis-scored RRI 41
Med-high draft above — this task's confirmed RRI is 35, Moderate, which uses
the direct local-first route and 2 Reflection passes, not ADR-038.)

**Status artifacts affected:** this ledger; the plan's G10 entry.

**Handoff prompt:** Bind the API's preparation queue to Redis at startup and
allow one `AppState` constructor to carry both the auth service and the injected
queue. Do not change the finalize handler's existing fail-closed enqueue-error
behavior, the job payload, or the queue namespace.

**Stop condition:** Stop once a Redis-backed consumer is shown to receive an
API-enqueued job. Do not start T2 and do not touch the worker-runner's worker
registration.

**Task-analysis review:** gemma `docs/audit/gemma-evidence/s-230-t1b-phase1.json` - PASS

### Implementation routing evidence

**Whole-task route:** RRI 35 (Moderate). Presented and approved in a prior
session; routed through the direct local-first path
(`scripts/local-agent/run_local_task.py`, `DUBBRIDGE_LOCAL_AGENT_MODEL`) per
`docs/policies/HITL_AUTONOMY_POLICY.md § Local-first implementation (RRI
26-40 Moderate)`. Both evidence-backed local repair attempts were exhausted
(`.agent/local-runs/s-230-t1b/attempt1.json`, `attempt2.json`) without a
usable in-scope patch, triggering
`docs/policies/HITL_AUTONOMY_POLICY.md § Post-repair-budget Low-band
decomposition` (owner directive 2026-08-16): decompose the remaining work
into Low-band (RRI 0-25) subtasks rather than escalate to cloud, orchestrator
acting as orchestrator only.

**Low-band subtasks dispatched** (all via `scripts/delegate-low-rri.py`,
personally reviewed by the orchestrator against acceptance criteria before
any patch was applied — build, test, clippy `-D warnings`, and `cargo fmt
--check` re-verified after every apply):

| Subtask | RRI | Scope | Attempts | Outcome |
|---|---|---|---|---|
| S-230-T1b-low-a | 24 | `apps/api/src/state.rs`: `with_auth_service_and_preparation_queue` constructor + 2 unit tests | 6 (see below) | Landed on attempt 6 (`muse-glimmer:30b-q4_K_M`) |
| S-230-T1b-low-b | 24 | `apps/api/src/main.rs`: Redis connect + constructor call site | 2 (`qwen3.8:27b-mlx`) | Landed on attempt 2 |
| S-230-T1b-low-c1 | 4 | `apps/api/Cargo.toml`: `apalis`/`apalis-redis` dev-dependencies | 1 (`qwen3.8:27b-mlx`) | Landed on attempt 1 |
| S-230-T1b-low-c2 | 20 | `apps/api/tests/redis_preparation_queue_test.rs` (new file): Redis-consumer-visibility integration test | 1 (`qwen3.8:27b-mlx`) | Landed on attempt 1 |

**Subtask A attempt detail** (`.agent/local-runs/s-230-t1b/low-band/`):
attempt 3 (`qwen3.8:27b-mlx`) failed to compile — the orchestrator's own
packet specified a nonexistent `InMemoryStorageAdapter` and a fictional
4-string-argument `enqueue`, corrected in place by reading
`crates/storage/src/lib.rs` and `crates/jobs/src/lib.rs` directly rather than
guessing (root cause: orchestrator packet error, not the model). Attempt 4
(`qwen3.8:27b-mlx`, corrected packet) regressed by deleting the pre-existing
`with_preparation_queue` function — root cause: an oversized BEFORE anchor
plus an ambiguous "replace your test module" instruction let the model
over-scope (again an orchestrator packet defect). Attempts 5
(`qwen3.8:27b-mlx`) and 6 (`muse-glimmer:30b-q4_K_M`, escalated per the
documented Low-band reviewer/developer fallback chain after two consecutive
same-class failures) both independently reproduced the identical
`unexpected closing delimiter` defect against a minimal 2-line append-only
anchor — cross-model reproduction of the same defect against the same packet
is what isolated the root cause to the orchestrator's own packet (an anchor
that closed `impl AppState` combined with append content that assumed the
block was still open), not either model. The packet was rewritten to append
a complete, self-contained second `impl AppState { ... }` block, removing
the ambiguity; the rewritten packet landed cleanly on the next dispatch.

**Subtask B attempt detail:** attempt 1 (`qwen3.8:27b-mlx`) correctly
returned `BLOCKED` rather than guessing, because the orchestrator's
first-draft packet described the required edit in prose without embedding
the literal BEFORE anchor text (unlike subtask A's packet) — again an
orchestrator packet defect, not a model failure; the model's refusal to
proceed without seeing real file content was the correct behavior. The
packet was rewritten with an explicit BEFORE/AFTER block (verified
byte-unique against the real file before dispatch) and landed on the next
attempt.

**Acceptance-criteria gap found during closure, not part of the original
decomposition:** the ledger's acceptance criteria required "an
integration-level test proves a job enqueued through the API's configured
queue is visible to a Redis-backed consumer, not only that `enqueue`
returned `Ok`" — subtasks A and B did not cover this (they only added unit
tests using `InMemoryPreparationJobQueue` as a test double). Discovered
during the orchestrator's own closure review (re-reading the ledger's
acceptance criteria against the applied diff), not flagged by any model.
Decomposed into subtasks C1 (dev-dependency wiring) and C2 (the test itself,
mirroring the already-proven `crates/jobs::redis_enqueued_job_is_retrievable_from_its_namespace`
pattern through `AppState`'s own constructor). The Makefile's `qa-test-redis`
target was deliberately NOT extended to cover this new test in this task —
`Makefile` recipe lines are tab-indented and a local-model transcription
error there fails silently (`*** missing separator`) in a way `cargo
build`/`clippy` cannot catch; wiring it in is left as an explicit follow-up.
The new test documents its own manual run command instead
(`cargo test -p dubbridge-api --test redis_preparation_queue_test --
--ignored`, requires `DUBBRIDGE_REDIS_URL`) and was verified for real against
a local Redis instance (`redis://127.0.0.1:6379/15`, matching the db index
already used by the project's own `qa-test-redis` documentation) before and
after delegation.

**Whitespace-only mechanical fixes applied directly by the orchestrator**
(the narrow "mechanical, lint-driven refactor of already-verified logic, no
behavior change" exception in `docs/policies/HITL_AUTONOMY_POLICY.md §
Post-repair-budget Low-band decomposition`, step 7): `cargo fmt` was run
after landing subtasks A, B, and C2, each time correcting only indentation
(an extra space on continuation lines from the local models' text
formatting) — re-verified with `cargo build`, `cargo test`, and `cargo
clippy -D warnings` after each fmt pass. No logic was authored directly by
the orchestrator at any point in this task.

**Net authorship split:** 100% of production and test logic was authored by
local models (`qwen3.8:27b-mlx`, `muse-glimmer:30b-q4_K_M`) across 4
Low-band delegations; the orchestrator's direct contribution was limited to
diagnosis (reading real crate signatures before writing packets), packet
construction/correction across 3 rounds of root-cause analysis, review, and
mechanical `cargo fmt` normalization.

### Reflection log

Required passes: 2 (`35` -> `Moderate`)

#### Pass 1

- **Draft verdict:** all four subtasks (A/B/C1/C2) applied together; build,
  86 non-ignored tests, the ignored Redis integration test (against a real
  local Redis instance), `cargo clippy -D warnings`, and `cargo fmt --check`
  all pass clean.
- **Critique findings:**
  - HP-1's ledger wording ("the worker observes and executes it") is a
    stronger claim than what the new integration test actually exercises:
    the test proves visibility via an independent `apalis_redis::RedisStorage
    ::fetch_by_id` probe in the same namespace a worker polls (mirroring the
    project's own precedent test in `crates/jobs`), not a live worker
    process popping and executing the job end to end. The task's own
    acceptance-criteria bullet ("proves a job... is visible to a
    Redis-backed consumer, not only that `enqueue` returned `Ok`") is the
    narrower, authoritative bar and is fully met; standing up a live
    `apalis` `WorkerBuilder` consumer loop for this task would be
    disproportionate scope. Recorded as an explicit interpretation rather
    than silently claiming full end-to-end HP-1 coverage.
  - EC-1 (fail-closed on enqueue error) is untouched by this task
    (`apps/api/src/routes/ingestion.rs` has zero diff) and already has a
    dedicated pre-existing test
    (`finalize_marks_preparation_failed_when_enqueue_fails`) — confirmed
    still passing, no regression.
  - EC-3 ("misconfigured `redis_url` surfaces at boot") cannot be unit
    tested at the `main()` level (it is the binary entrypoint and blocks on
    `axum::serve`); the underlying fail-closed connect behavior it depends
    on is already unit-tested at `crates/jobs::redis_queue_fails_closed_on_malformed_url`
    / `redis_queue_fails_closed_on_unreachable_server`, and the boot-order
    guarantee itself follows from `?` inside `async fn main() ->
    anyhow::Result<()>` propagating before any later statement (including
    `axum::serve`) runs — a Rust language guarantee, not something that
    needs a bespoke test.
  - No adjacent-module side effects: the six constructor parameters are all
    distinct types, so the compiler would reject any transposed-argument
    regression; already build-verified.
  - `TempDir` lifetime in the new integration test: `storage_dir` is bound
    for the full test function scope, matching the existing working pattern
    in `apps/api/tests/ingestion_test.rs` — no early-drop risk.
- **Revisions applied:** none required; findings were interpretation notes
  to record explicitly, not defects.

#### Pass 2

- **Draft verdict:** stable; incorporating Gemma Reviewer's phase-2 finding
  as input per policy.
- **Critique findings:** Gemma Reviewer (3/3 passes, consensus) flagged that
  `with_auth_service_and_preparation_queue` duplicates
  `workspace_service: pg_workspace_service(pool.clone())` construction
  already present in `new`, `with_auth_service`, `with_workspace_service`,
  and `with_preparation_queue` (`nit` severity). Verified by direct
  inspection (`grep -n pg_workspace_service apps/api/src/state.rs`): this
  duplication is a **pre-existing pattern across all four prior
  constructors**, not a regression introduced by this task — the new
  constructor follows the exact established convention. A builder-pattern
  refactor across all of `AppState`'s constructors is explicitly out of this
  task's scope (its own "do not do this" boundary: "Do NOT modify the
  existing `new`, `with_auth_service`, `with_workspace_service`, or
  `with_preparation_queue` functions or their signatures") and would touch
  every call site in the crate, warranting its own separately-scoped task.
- **Revisions applied:** none in this task's scope; disposition recorded as
  `accepted-follow-up` (valid observation, pre-existing pattern, deferred to
  a future AppState-constructor cleanup task, not this one).

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `REVIEW_PATHS="apps/api/src/state.rs apps/api/src/main.rs apps/api/Cargo.toml apps/api/tests/redis_preparation_queue_test.rs" GEMMA_REVIEW_TASK_ID=s-230-t1b make qa-gemma-review`
- Artifact: `docs/audit/gemma-evidence/s-230-t1b.json`
- Verdict: PASS
- Findings: 1 consensus `nit` (duplicated `workspace_service` construction,
  pre-existing pattern — see Reflection log Pass 2)
- Muse Glimmer fallback: not triggered — reason: n/a (Gemma responded with a
  usable 3/3-pass consolidated result)
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted the finding as valid but out of this
  task's scope (pre-existing duplication pattern across all AppState
  constructors); no code change made in response

Code-solution review: gemma `docs/audit/gemma-evidence/s-230-t1b.json` - PASS

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | job enqueued through the API's configured (Redis) queue is visible to a Redis-backed consumer via the same namespace/fetch mechanism a worker polls | `apps/api/tests/redis_preparation_queue_test.rs::job_enqueued_through_api_configured_queue_is_visible_to_redis_consumer` (requires `DUBBRIDGE_REDIS_URL`; verified against a real local Redis instance) | passed |
| HP-2 | Happy path | `AppState` carries both a real auth service and an injected preparation queue at once | `apps/api/src/state.rs::tests::with_auth_service_and_preparation_queue_sets_auth_service`, `apps/api/src/state.rs::tests::with_auth_service_and_preparation_queue_uses_passed_queue` | passed |
| EC-1 | Edge case | Redis-unreachable enqueue failure fails closed: `preparation_status = Failed`, 500 response | `apps/api/tests/ingestion_test.rs::finalize_marks_preparation_failed_when_enqueue_fails` (pre-existing, unmodified by this task, re-verified passing) | passed |
| EC-2 | Edge case | `InMemoryPreparationJobQueue` remains usable as a test double after this task | `apps/api/src/state.rs::tests::with_auth_service_and_preparation_queue_uses_passed_queue` (uses `InMemoryPreparationJobQueue` directly); full existing `apps/api/tests/ingestion_test.rs` suite (which also uses it) re-verified passing with no regression | passed |
| EC-3 | Edge case | misconfigured `redis_url` fails the process at boot, not on first request | `crates/jobs/src/lib.rs::tests::redis_queue_fails_closed_on_malformed_url`, `crates/jobs/src/lib.rs::tests::redis_queue_fails_closed_on_unreachable_server` (fail-closed connect behavior this task's `main.rs` wiring depends on); boot-ordering itself follows from Rust's `?`-propagation semantics in `apps/api/src/main.rs`'s `async fn main() -> anyhow::Result<()>`, verified by code inspection (not independently unit-testable without spawning the binary as a subprocess) | passed |

### Owner final verification

- Owner: `matias`
- Date: `2026-08-17`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, with HP-1 and
  EC-3 covered by evidence at the narrower/underlying level documented above
  rather than a bespoke end-to-end test, for the reasons stated in the
  Reflection log and certification table.
- Commands run: `cargo build -p dubbridge-api`, `cargo test -p dubbridge-api`,
  `DUBBRIDGE_REDIS_URL=redis://127.0.0.1:6379/15 cargo test -p dubbridge-api
  --test redis_preparation_queue_test -- --ignored --test-threads=1`,
  `cargo clippy -p dubbridge-api --all-targets -- -D warnings`, `cargo fmt -p
  dubbridge-api --check`

**Status:** [x] Done 2026-08-17

---

## S-230-T2: Migration runner in the production path

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

**Problem (plan G2):** `sqlx::migrate!` is used only in test modules. Nothing
applies the 29 files in `infra/migrations/` to a real database, and `apps/cli` is
a skeleton.

**Happy paths considered:**

- **HP-1:** Against an empty database, the runner applies all 29 migrations in
  order and populates `_sqlx_migrations`.
- **HP-2:** A second run against the same database is a no-op and exits zero
  (safe to run on every deploy).

**Edge cases considered:**

- **EC-1:** An unreachable database or a failing migration exits non-zero with
  the failing version identified; api and worker startup must not proceed.
- **EC-2:** A database whose applied-migration checksum diverges from the
  embedded set fails closed rather than applying a partial or reordered set.

**Acceptance criteria:**

- `apps/cli` exposes an explicit `migrate` entry point that embeds
  `infra/migrations` and is runnable as a one-shot container command.
- Configuration is loaded through the same fail-closed `AppConfig::load()` path
  as api and worker; no separate connection-string handling.
- Exit codes are meaningful enough for a Compose dependency condition to gate
  application startup on migration success.

**Files expected to change:** `apps/cli/src/main.rs`, `apps/cli/Cargo.toml`,
and focused tests. Recompute before presentation.

**Evidence to emit:** RRI report, phase reviews, real-PostgreSQL run evidence
(empty database and re-run), Reflection log if required, unit coverage
certification, owner verification.

**Status artifacts affected:** this ledger.

**Handoff prompt:** Add a `migrate` command to `apps/cli` that applies
`infra/migrations` through the existing config loader, idempotently, with
non-zero exit on failure. Do not change any migration file.

**Stop condition:** Stop after migration-run evidence. Do not wire Compose yet.

### Implementation routing evidence

- **Whole-task local-agent route:** `scripts/local-agent/run_local_task.py`
  with `qwen3.8:27b-mlx` in a disposable worktree, per the RRI 28 Moderate
  band. Both attempts (1/2 and 2/2) aborted identically with
  `status: aborted, reason: malformed_tool_call_repeated` — the model
  correctly diagnosed the required `Cargo.toml`/`main.rs` content each time
  (visible in the transcript's malformed payloads) but emitted double-escaped
  JSON (`\\n` instead of `\n` inside the tool-call JSON string) that the
  wrapper's parser rejected on every turn, exhausting `MAX_MALFORMED_BOUNCES`
  (3) within each invocation. Transcripts:
  `transcript-attempt1.json`, `transcript-attempt2.json` (session scratch,
  not persisted to the repo). This is a reproducible wrapper/model
  tool-calling-format failure, not a scope, comprehension, or capability gap.
- **Post-repair-budget decomposition (owner directive 2026-08-16):** per
  `docs/policies/HITL_AUTONOMY_POLICY.md § Post-repair-budget Low-band
  decomposition`, the remaining work was decomposed into three Low-band
  (RRI 0–25) subtasks rather than escalating to cloud, diagnosing exact
  repository signatures first (workspace `sqlx` feature gap, `dubbridge-db`
  path convention, `AppConfig::load()` vs `from_env()`, the exact
  `sqlx::migrate!("../../infra/migrations")` invocation already used
  identically in ~20 files under `apps/api/tests/`) so each packet was
  self-contained and low-risk:
  - `S-230-T2-a` (RRI 14) — add `sqlx` (`migrate` feature) + `dubbridge-db`
    to `apps/cli/Cargo.toml`. Delegated via `scripts/delegate-low-rri.py`
    (`--mode full-file`, tagged-block contract). Model output matched the
    packet spec verbatim; applied as-is; `cargo check -p dubbridge-cli`
    verified clean.
  - `S-230-T2-b` (RRI 19) — replace `apps/cli/src/main.rs` skeleton with the
    migration runner. Delegated the same way. Model output matched the
    packet spec verbatim; applied as-is; `cargo build -p dubbridge-cli`
    verified clean.
  - `S-230-T2-c` (RRI 14) — new `apps/cli/tests/migrate_test.rs` integration
    test. Delegated the same way. Model output matched the packet spec
    verbatim; applied as-is (directory created); full acceptance suite
    verified clean (see Unit coverage certification below).
  - All three patches were reviewed by the orchestrator against their
    packets' acceptance criteria before being applied directly to the
    primary checkout (no tooling-failure exception or mechanical-refactor
    exception was invoked — every line originated from local-model output).
- **Net authorship split:** 100% of production and test logic was authored
  by `qwen3.8:27b-mlx` across 3 Low-band delegations after the whole-task
  route's budget was exhausted. The orchestrator's direct contribution was
  limited to diagnosis (reading real crate/macro signatures before writing
  packets — including sourcing `sqlx-macros-core-0.8.6`'s actual
  `resolve_path` implementation to ground packet B's design), packet
  construction, and applying the three validated patches verbatim (no
  hand-authored logic).

### Reflection log

Required passes: 2 (`28` -> `Moderate`)

#### Pass 1

- **Draft verdict:** all three subtasks (a/b/c) applied; build, clippy
  (`-D warnings`), `cargo fmt --check`, and the real-Postgres integration
  test all pass clean; `cargo build --workspace` and `cargo clippy
  --workspace --all-targets -- -D warnings` show no regressions elsewhere.
- **Critique findings:**
  - HP-1 ("against an empty database...") was verified against the shared
    local `dubbridge` Postgres database (already migrated by other test
    suites), not a literally empty one — the same convention every existing
    file under `apps/api/tests/` already uses. The `_sqlx_migrations` row
    count assertion (`== 29`) is equivalent evidence regardless of whether
    the database was empty or already migrated before this test ran,
    because `sqlx`'s migrator inserts exactly one row per applied version,
    not per invocation. Recorded as an explicit interpretation, matching the
    precedent set by S-230-T1b's own Pass 1 note on HP-1/EC-3.
  - EC-1 (unreachable DB / failing migration exits non-zero) has no
    CLI-binary-level test; `apps/cli/tests/migrate_test.rs` intentionally
    does not spawn the compiled binary (this was an explicit stop condition
    in packet C, matching how no other integration test in this workspace
    shells out to a compiled binary either). The underlying guarantee —
    that `create_pool` and `sqlx::migrate!().run()` return `Result`s `?`
    propagates before `Ok(())` is reached — is a Rust language guarantee
    inside `async fn main() -> anyhow::Result<()>`, identical in kind to how
    S-230-T1b treated its own EC-3.
  - EC-2 (checksum divergence fails closed) is internal `sqlx::migrate!`
    behavior, not logic written by this task; not independently tested,
    consistent with treating third-party library guarantees as out of this
    task's unit-test scope.
  - No `database_url` value appears in any log line — verified by direct
    inspection of the applied `main.rs`.
  - No adjacent-module side effects: `apps/cli` is a leaf binary crate (no
    other workspace crate depends on it); `cargo build --workspace` after
    the change confirms no regression.
- **Revisions applied:** none required; findings were interpretation notes
  to record explicitly, not defects.

#### Pass 2

- **Draft verdict:** stable; incorporating Gemma Reviewer's phase-2 findings
  as input per policy, both independently re-verified against primary
  sources rather than accepted or dismissed on read.
- **Critique findings:** Gemma Reviewer (3/3 passes) returned `FINDINGS`
  with two `major`-severity items. Both were independently verified against
  primary evidence (not taken at face value in either direction) and found
  to be false positives:
  - **Finding 1** (consensus): claimed `sqlx::migrate!("../../infra/migrations")`
    "is resolved at compile time relative to the source file location" and
    "resolves outside the workspace" when built from a different working
    directory. Verified false by reading the actual macro implementation
    shipped in this workspace's locked dependency,
    `~/.cargo/registry/.../sqlx-macros-core-0.8.6/src/common.rs:28`
    (`resolve_path`), which resolves the path against
    `env::var("CARGO_MANIFEST_DIR")` — the crate's own directory, fixed at
    compile time — not the process's runtime working directory. Empirically
    re-confirmed by running `cargo build -p dubbridge-cli
    --manifest-path /Users/matias/dubbridge/Cargo.toml` from `/tmp` (a
    different cwd), which compiled clean. `apps/cli` sits at the same
    nesting depth (`apps/<crate>/src/`) as `apps/api` and
    `apps/worker-runner`, both of which already use the identical
    `"../../infra/migrations"` string without incident. The finding's own
    suggested fix ("anchor via CARGO_MANIFEST_DIR") is, in fact, already
    what the macro does internally.
  - **Finding 2** (pass-specific): claimed switching `AppConfig::from_env()`
    to `AppConfig::load()?` is "a breaking change for existing CLI
    invocations that relied on environment variables only." Verified: (a)
    `AppConfig::load()` (`crates/config/src/lib.rs:167`) still merges
    `Env::prefixed("DUBBRIDGE_").split("__")`, so existing `DUBBRIDGE_*`
    env-var-only invocations keep working; (b) no Compose file, Dockerfile,
    or script anywhere in the repository invokes `dubbridge-cli` yet (grep
    confirmed) — S-230-T4/T5 have not wired it in, so there is no existing
    invocation to break; (c) `docs/tasks/s-030-t2-layered-loader.md:165`
    already documents `from_env()` as deprecated/legacy with planned
    removal, and line 136 names `apps/cli/src/main.rs` specifically as a
    file that needed this exact migration; (d) this task's own acceptance
    criteria required this change verbatim ("Configuration is loaded
    through the same fail-closed `AppConfig::load()` path as api and
    worker"), and the phase-1 review had already flagged the pre-change
    skeleton's use of `from_env()` as the thing to fix. The change is
    intentional, required, and pre-documented, not an undocumented
    regression.
- **Revisions applied:** none — both findings dispositioned as false
  positives with primary-source evidence (dependency source code, grep
  across the repository, and the pre-existing S-030 migration-debt record),
  not merely re-asserted from initial judgment.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `REVIEW_PATHS="apps/cli/src/main.rs apps/cli/Cargo.toml apps/cli/tests/migrate_test.rs" GEMMA_REVIEW_TASK_ID=s-230-t2 make qa-gemma-review`
- Artifact: `docs/audit/gemma-evidence/s-230-t2.json`
- Verdict: FINDINGS (aggregate receipt records `FINDINGS-ACKED` after disposition)
- Findings: 2 `major` (1 consensus, 1 pass-specific) — both independently
  re-verified against primary sources (sqlx macro source, grep across repo,
  S-030 task ledger) and dispositioned as false positives; see Reflection
  log Pass 2 for full evidence chain
- Muse Glimmer fallback: not triggered — reason: n/a (Gemma responded with a
  usable 3/3-pass consolidated result)
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a
- disposition_divergence: none
- Primary-agent disposition: both findings rejected as false positives with
  cited primary-source evidence (not merely asserted); no code change made
  in response

Code-solution review: gemma `docs/audit/gemma-evidence/s-230-t2.json` - FINDINGS (both dispositioned false-positive)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | migration runner applies all 29 `infra/migrations` files and populates `_sqlx_migrations` | `apps/cli/tests/migrate_test.rs::migrations_apply_and_are_idempotent_on_second_run` (requires `DUBBRIDGE_DATABASE_URL`; verified against a real local Postgres instance; asserts row count == 29) | passed |
| HP-2 | Happy path | a second run against the same database is a no-op and succeeds | `apps/cli/tests/migrate_test.rs::migrations_apply_and_are_idempotent_on_second_run` (second `migrate!().run(&pool).await` call in the same test, asserted `Ok`) | passed |
| EC-1 | Edge case | unreachable DB / failing migration exits non-zero, does not proceed | not independently unit-tested (binary-entrypoint behavior); guaranteed by `?`-propagation inside `async fn main() -> anyhow::Result<()>` (`apps/cli/src/main.rs`) — a Rust language guarantee, verified by code inspection, same treatment as S-230-T1b's EC-3 | passed (by inspection) |
| EC-2 | Edge case | checksum-divergent migration set fails closed rather than partially applying | not independently unit-tested; internal `sqlx::migrate!` behavior (third-party library guarantee), out of this task's unit-test scope, same treatment as S-230-T1b's EC-1/EC-3 | passed (by library contract) |

### Owner final verification

- Owner: `matias`
- Date: `2026-08-17`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, with EC-1 and
  EC-2 covered by inspection/library-contract evidence rather than a bespoke
  unit test, for the reasons stated in the Reflection log and certification
  table, and I have reviewed the disposition of both Gemma Reviewer findings
  as false positives.
- Commands run (available for independent re-verification): `cargo build -p
  dubbridge-cli`, `cargo clippy -p dubbridge-cli --all-targets -- -D
  warnings`, `cargo fmt -p dubbridge-cli --check`,
  `DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@localhost:5432/dubbridge
  cargo test -p dubbridge-cli --test migrate_test -- --test-threads=1`,
  `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`

**Status:** [x] Done 2026-08-17

---

## S-230-T3: Real readiness probes for api and gateway

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [x] Done — owner-directed closure 2026-08-17

**Problem (plan G3):** `apps/api/src/lib.rs:49` returns `"ready"`
unconditionally, never touching PostgreSQL, Redis, or storage. The gateway has
the same stub pair.

**Happy paths considered:**

- **HP-1:** With PostgreSQL, Redis, and storage reachable, `GET /health/ready`
  returns 200 and names each checked component.
- **HP-2:** `GET /health/live` stays a cheap process-liveness answer and does not
  depend on any external system.

**Edge cases considered:**

- **EC-1:** With PostgreSQL unreachable, readiness returns 503 and identifies the
  failing component; the process does not crash.
- **EC-2:** A hung dependency is bounded by a timeout so the probe cannot itself
  hang the health check.
- **EC-3:** Gateway readiness reflects its upstream API reachability rather than
  reporting ready while the API is down.

**Acceptance criteria:**

- Readiness performs a bounded check per dependency and aggregates to 200 or 503.
- Liveness and readiness stay semantically distinct.
- No credential or connection string appears in the response body.

**Files expected to change:** `apps/api/src/lib.rs`, `apps/gateway/src/lib.rs`,
supporting state/helpers, and focused tests. Recompute before presentation.

**Evidence to emit:** RRI report, phase reviews, tests covering healthy and
degraded dependency states, Reflection log if required, unit coverage
certification, owner verification.

**Status artifacts affected:** this ledger.

**Handoff prompt:** Make `/health/ready` actually probe PostgreSQL, Redis, and
storage with bounded timeouts in api, and upstream reachability in gateway.
Leave `/health/live` cheap. No new endpoints.

**Stop condition:** Stop after probe tests. Do not build images.

### Execution evidence (2026-08-17)


- Code-solution review: `gemma4:26b-a4b-it-qat` via local Ollama `/api/chat` — PASS; no findings.
- Verification: `cargo test -p dubbridge-api --lib --no-fail-fast` — 89 passed; `cargo test -p dubbridge-gateway --lib --no-fail-fast` — 13 passed.
- Scope verified: API readiness probes PostgreSQL, Redis, and storage; gateway readiness probes its upstream API; liveness routes remain independent.
- Formatting note: `git diff --check` reports trailing whitespace in `apps/gateway/src/lib.rs`; the owner explicitly accepted leaving it unchanged.
- **Owner-directed closure (2026-08-17):** Matias declared S-230-T3 closed. This
  resolves T3 as a dependency for downstream tasks. The missing retrospective
  RRI, phase-1 artifact, Reflection log, unit-coverage certification, and
  structured owner-verification record remain an evidence debt; they are not
  represented as having been reconstructed or passed retroactively.

---

## S-230-T3b: Cross-language subtitle translation pipeline (S-150 reopening)

**Type:** development parent (not executable as written)
**Effort:** XL (aggregate of 5 Med-high + 1 Moderate children; each child scores and
is approved independently — see child RRI figures below)
**Depends on:** S-230-T0 (independent of T1/T1b/T2/T3/T4/T5 — parallel track, not
on the deployment critical path)
**Status:** [ ] Planned — not executable directly; execute children in order

> **Owner scope amendment, 2026-08-16 (second pass).** The original S-230 scope
> froze `S-150` out entirely (`docs/plan/s-230-poc-v1-digitalocean.md` §"Scope
> decision"). A same-day follow-up review ("The market-audience gap, examined")
> computed the exact cost of reopening it and **explicitly recommended against
> doing so inside this window** ("Recommendation: C for this POC... Do not let
> the desire for a translated-clip moment pull S-150 scope back into S-230
> piecemeal"). The owner reviewed that recommendation and its cost table and
> explicitly confirmed reopening anyway, to make the product's actual
> differentiator (cross-language output, per `README.md:3`) demonstrable rather
> than asserted. This task is the resulting scope amendment record and
> integration tracker. It does not re-litigate that decision; it exists so the
> calendar and governance cost is visible rather than absorbed silently into
> "T2c-v."

**Objective:** Produce one real (non-mocked) translated-subtitle artifact for a
target language, through the already-reviewed S-150 delivery pipeline, so the
POC can demonstrate genuine cross-language output — text-only, no dubbed audio.

**Why this is a parent, not a single task:** the six child tasks below are
already fully defined with their own HP/EC, RRI, and acceptance criteria in
`docs/tasks/s-150-translation-dubbing.md`. This task does not redefine them; it
sequences them under S-230 and adds the cross-slice integration/demo
acceptance check. Duplicating their content here would create two sources of
truth for the same requirements — the S-150 ledger stays authoritative for
each child's own definition, RRI, review, Reflection, and closure.

**Ordered children (execute in this order; each still needs its own RRI
presentation + explicit approval before implementation — this parent's
approval does not pre-approve them):**

| Order | Task ID | What it does | RRI | Band |
|---|---|---|---|---|
| 1 | `S-150-T2c-v` | Redis translation-queue adapter | 50 | Med-high |
| 2 | `S-150-T2c-vi-a` | Wire `fan_out_localization` into the subtitle runtime, replacing `prepare_review_post_ready` | 51 | Med-high |
| 3 | `S-150-T2c-vi-b` | Delete the dead legacy review module, sync S-140 BDD | 31 | Moderate |
| 4 | `S-150-T3a` | Typed translation provider/subprocess contract | 42 | Med-high |
| 5 | `S-150-T3b` | Functional Python translation worker | 44 | Med-high |
| 6 | `S-150-T3c` | Rust translation runtime persistence + readiness transition | 53 | Med-high |

Every Med-high child routes cloud-only under ADR-038 (Muse Glimmer refinement
-> primary receipt -> cloud takeover, no local repair attempts); `T2c-vi-b` is
Moderate and may use the local-first path. Each carries its own band-resolved
phase-1/phase-2 review, Reflection passes, unit coverage certification, and
owner verification — this parent adds no exemption from any of that.

**Explicit boundary (unchanged from the original freeze):** `S-150-T4`
(ADR-028 amendment) through `S-150-T7` — i.e. TTS/dubbed audio — remain out of
scope for S-230. This chain produces translated subtitle text only. Do not
implement T4/T5/T6/T7 under this task or this slice.

**Blocking precondition not resolved by this amendment:** `S-150-T2c-v`
carries its own, separate "parked pending a Redis-topic decision" note
(`docs/plan/s-150-translation-dubbing.md:31`, `:322`) that predates and is
independent of the S-230 scope freeze. This task authorizes presenting and
implementing the chain under S-230; it does not itself reopen that Redis-topic
decision. Confirm that separately with the owner before starting child 1.

**Happy paths considered:**

- **HP-1:** For one real S-140 subtitle artifact and one configured target
  language, the full chain (queue -> runtime fan-out -> provider -> worker ->
  persistence) produces a persisted `TranslatedSubtitle` artifact that a
  Redis-backed consumer actually processed — not just "enqueued."

**Edge cases considered:**

- **EC-1:** If any child in the chain is not complete by the time T6 or T9
  runs, the POC's e2e smoke and closure report state that plainly ("translation
  chain partial: children N/6 done") rather than silently describing the POC
  as showing translated output it does not yet produce. This mirrors the
  project's own "assert on observed downstream state, never a 2xx alone"
  principle (T6 acceptance criteria) applied to this task's own status
  reporting.

**Acceptance criteria:**

- All six children above are `[x] Done` with their own full closure records in
  `docs/tasks/s-150-translation-dubbing.md`.
- A real test asset's source-language subtitle produces a persisted
  translated-subtitle artifact for at least one target language, verified
  against real Redis/PostgreSQL/storage (not mocks) — this is the integration
  check this parent owns on top of each child's own acceptance criteria.
  **This proves the artifact exists in storage; it does not make it visible
  to a human reviewer — that is `S-230-T8b`'s separate scope.**
- `S-150-T2c-v`'s separate Redis-topic parking note is confirmed resolved by
  the owner before child 1 starts (see blocking precondition above).
- No `S-150-T4`–`T7` (TTS/dubbing) code is touched.

**Calendar note (owner-acknowledged cost):** the plan's own cost analysis
(`docs/plan/s-230-poc-v1-digitalocean.md` §"D, scoped") found this chain does
not fit inside the original 10-day window alongside a first deployment. This
task therefore runs as a **parallel track**, not a hard gate on `S-230-T6`
(first deploy + smoke): T6 may complete and demonstrate the pre-existing
subtitle/review/publication path on schedule even if this chain is still in
progress. If complete before `S-230-T9` closeout, the translated-subtitle
capability is folded into the demo narrative and the debt register entry for
"no cross-language capability" is removed; if not, T9 records the exact
child-completion state instead of a blanket "done" or "not applicable."

**Downstream coupling with S-230 deployment tasks (conditional, not
blocking).** This task's own acceptance criteria (above) only requires the
chain to run against real local/dev-infra Redis/PostgreSQL/storage, matching
how every other S-150 task has been verified — that alone does **not** put
translation on the deployed Digital Ocean droplet. `S-150-T3b` ("Functional
translation worker") is structurally identical to the existing ASR worker
(`workers/asr-worker-py`): a Python subprocess `apps/worker-runner` shells out
to, per its own acceptance criteria ("Implement the Python stdin/stdout
worker... keep model credentials in injected environment only"). For the
translated-subtitle capability to actually appear on the deployed POC, not
only in this chain's own closure evidence, three sibling tasks need
conditional scope additions **if and when this chain is done before they
execute** — none of these are hard blocking dependencies added to those
tasks' `Depends on` fields, since `T3b` may still be in progress when they run:

- **`S-230-T4` (production images):** the worker-runner image must also bundle
  `workers/translation-worker-py` and its dependencies, mirroring the existing
  ffmpeg/Python/faster-whisper bundling for ASR, with whatever
  path/interpreter env vars `S-150-T3c` defines for it (mirroring
  `DUBBRIDGE_ASR_WORKER_PATH`/`DUBBRIDGE_ASR_WORKER_PYTHON`). If `T3b`'s
  children are not yet done when `T4` builds its image, `T4` proceeds without
  the translation worker and a follow-up image rebuild is needed once they
  close — this is now recorded as a known follow-up, not a silent gap.
- **`S-230-T5` (descriptor + secret boundary):** the environment template must
  add whichever `DUBBRIDGE_*` variable(s) `S-150-T3b`'s configurable provider
  needs for real (non-fake) credentials, the same way T5 already carries
  `S-230-T7c`'s JWT-expiry decision without owning it. T5 owns only carrying
  the variable; `T3b`'s children define its name and requirement.
- **`S-230-T6` (deploy + smoke):** if the translation worker is present in the
  deployed image by the time of the smoke run, the runbook additionally
  asserts a translated-subtitle artifact on observed downstream state (same
  "never a 2xx alone" standard T6 already applies to every other stage); if
  not present, T6 proceeds exactly as originally scoped and T9 records the gap.
- **`S-230-T8b` (translated subtitle visible in review, added 2026-08-16):**
  producing and persisting a translated artifact (this task's own acceptance
  criteria) is **not** the same as a human being able to see it. `T8b` is the
  task that actually closes that gap — it is a separate, double-conditional
  task (depends on both this task and `T8`), not a bullet on an existing task,
  because it needs its own read endpoint and mobile rendering work. Without
  `T8b`, a fully-closed `T3b` still leaves the translated artifact invisible
  to any reviewer, which undercuts this task's own stated objective.

Whoever executes `S-150-T3c` (the last child, which defines the actual
worker-path env var names) must update this section and the tasks above with
the concrete variable names once they are fixed — this section names the
coupling now so it isn't discovered late, but the exact names are `T3c`'s to
define, not this task's.

**Technical-scope diagram — child sequence and downstream coupling:**

```mermaid
flowchart TD
    T2CV["1 . S-150-T2c-v<br/>Redis adapter . RRI 50"] --> T2CVIA["2 . S-150-T2c-vi-a<br/>Runtime fan-out . RRI 51"]
    T2CVIA --> T2CVIB["3 . S-150-T2c-vi-b<br/>Cleanup + BDD sync . RRI 31"]
    T2CVIB --> T3A["4 . S-150-T3a<br/>Provider contract . RRI 42"]
    T3A --> T3Bc["5 . S-150-T3b<br/>Python worker . RRI 44"]
    T3Bc --> T3C["6 . S-150-T3c<br/>Persistence + readiness . RRI 53"]

    T3C -. "if done: bundle worker" .-> T4["S-230-T4<br/>production images"]
    T3C -. "if done: add credential var" .-> T5["S-230-T5<br/>descriptor + secrets"]
    T3C -. "if done: assert translated subtitle" .-> T6["S-230-T6<br/>deploy + smoke"]
    T3C --> T8b["S-230-T8b<br/>visible in review"]
    T8["S-230-T8<br/>subtitle in review"] --> T8b

    classDef child fill:#faf0dc,stroke:#8a5f0b,color:#1f2420;
    classDef ext fill:#ffffff,stroke:#ddd7c9,color:#1f2420;
    class T2CV,T2CVIA,T2CVIB,T3A,T3Bc,T3C child;
    class T4,T5,T6,T8,T8b ext;
```

Solid edges are real dependencies (the ordered child chain; `T8b` depending on
both `T3C` and `T8`). Dotted edges into `T4`/`T5`/`T6` are conditional —
those tasks never wait on this chain; each proceeds as originally scoped if
`T3c` is not yet done when they run.

**Evidence to emit:** each child's own full evidence bundle (RRI, phase
reviews, Reflection log, unit coverage certification, owner verification) per
`docs/tasks/s-150-translation-dubbing.md`; plus this parent's own integration
evidence (real chain run against a real test asset) and the Redis-topic
resolution confirmation.

**Status artifacts affected:** this ledger; `docs/tasks/s-150-translation-dubbing.md`
(child status, and the top-of-file/roadmap "parked for S-230" framing);
`docs/plan/s-150-translation-dubbing.md` (same); `docs/plan/roadmap.md` (S-150
and S-230 rows); `docs/plan/s-230-poc-v1-digitalocean.md` (scope decision,
demo-quality review, out-of-scope section, module-dependency diagram).

**Agent handoff prompt:** Do not implement this parent directly. Present and
execute `S-150-T2c-v` first, in its own RRI 26+ approval cycle, after
confirming the separate Redis-topic decision with the owner. Continue through
the ordered children only after each prior child closes. Stop before any
`S-150-T4`+ (TTS/dubbing) work.

**Stop condition:** Stop once all six children are closed and the integration
check passes, or report the exact partial state at T9 if the window closes
first. Do not start TTS/dubbing work under this task.

---

## S-230-T4: Production container images (non-executable parent)

**Type:** development parent (not executable as written)
**Effort:** aggregate of 17 independently-scored Low/S children
**Depends on:** S-230-T1, S-230-T1b, S-230-T2, S-230-T3
**Status:** [ ] Decomposed 2026-08-17 — execute `S-230-T4a` through `T4q`

**Historical whole-task RRI:** 47 — Med-high (41–55), no penalties. Evidence:
`docs/audit/s-230-t4-rri.md`. This route is superseded before implementation by
the owner-directed Low-band decomposition below because cloud implementation
tokens are unavailable. The parent preserves the aggregate contract but may not
be delegated or implemented directly. Child scores and exact commands:
`docs/audit/s-230-t4-low-rri-decomposition.md`.

**Task-analysis review:** Gemma's initial review
(`docs/audit/gemma-evidence/s-230-t4-phase1.json`) was **BLOCKED** by the then
unresolved T3 record. The owner-directed closure recorded in S-230-T3 resolves
that dependency for T4; its retrospective evidence debt remains tracked but is
not a T4 presentation blocker. The revised review
(`docs/audit/gemma-evidence/s-230-t4-phase1-rerun.json`) passed against this
synchronized record; the final task-definition review
(`docs/audit/gemma-evidence/s-230-t4-phase1-final.json`) is **PASS**. The
frozen T4 scope below already incorporates the initial review's reproducibility
and ASR-resource findings.

### Low-band decomposition contract (owner direction, 2026-08-17)

Every development child is an independent simple patch with one writable path,
`Effort: S`, and RRI 0–25. **Sequencing correction (2026-08-17):** the
original decomposition sequenced each contract-test child before the image
child it was meant to validate — Muse Glimmer's phase-1 review on the
original T4b (contract tests) returned `BLOCKED` because there was no image
yet to test against. The orchestrator confirmed this was a genuine gap, not a
false positive, and the owner directed inverting the order across all six
image/contract-test pairs (T4b/T4c, T4d/T4e, T4f/T4g, T4i/T4j, T4k/T4l,
T4m/T4n): each image-authoring child now precedes its own contract-test
child, so every Dockerfile is validated by a contract test written and run
against that already-built image, not the reverse. The table below reflects
the corrected order. The orchestrator supplies verified immutable OCI digests
in the delegation packet; local models never choose or refresh a base-image
version.

Implementation uses `qwen3.8:27b-mlx` through
`scripts/delegate-low-rri.py`. Independent phase-1 and phase-2 review uses the
Low-band chain `muse-glimmer:30b-q4_K_M` ->
`gemma4:26b-a4b-it-qat` -> D14. Each child is its own mandatory Ollama restart
boundary. No child inherits the parent's prior Gemma review: it receives its own
Muse Glimmer phase-1 review before delegation. Low-band children require no full
approval card.

| Child | Result | Writable path | Purpose |
|---|---:|---|---|
| T4a | RRI 15 | `scripts/test-production-images.sh` | bounded test harness |
| T4b | RRI 18 | `apps/api/Dockerfile` | API image |
| T4c | RRI 17 | same as T4a | API contract tests (against T4b image) |
| T4d | RRI 18 | `apps/gateway/Dockerfile` | gateway image |
| T4e | RRI 16 | test script | gateway contract tests (against T4d image) |
| T4f | RRI 17 | `apps/cli/Dockerfile` | migration image |
| T4g | RRI 20 | test script | migration contract tests (against T4f image) |
| T4h | RRI 14 | `workers/asr-worker-py/requirements.txt` | exact ASR dependency lock |
| T4i | RRI 21 | `apps/worker-runner/Dockerfile` | Rust + ffmpeg worker image |
| T4j | RRI 13 | test script | worker native-runtime tests (against T4i image) |
| T4k | RRI 21 | worker Dockerfile | Python + ASR bundle image |
| T4l | RRI 24 | test script | ASR bundle tests (against T4k image) |
| T4m | RRI 21 | worker Dockerfile | conditional translation bundle image |
| T4n | RRI 24 | test script | conditional translation tests (against T4m image) |
| T4o | RRI 25 | test script | full local image-pipeline contract |
| T4p | RRI 19 | `docs/audit/s-230-t4-local-image-evidence.md` | execute and record evidence |
| T4q | RRI 10 | task/plan/roadmap docs | aggregate closure and status sync |

`T4m`/`T4n` execute only if `S-150-T3b` and `S-150-T3c` are done before
worker-image integration. Otherwise `T4p` records the follow-up image rebuild as
debt and `T4q` may close the parent without those conditional children.

**Problem (plan G4):** Dockerfiles exist only for the Python workers. The
worker-runner image is the hard one: it shells out to `ffprobe`/`ffmpeg` and
spawns `python3 workers/asr-worker-py/main.py` as a subprocess.

**Acceptance criteria:**

- Multi-stage images for `dubbridge-api` and `dubbridge-gateway` producing a slim
  runtime layer with no build toolchain.
- A worker-runner image containing the Rust binary, ffmpeg/ffprobe, Python, and
  faster-whisper, with `DUBBRIDGE_ASR_WORKER_PATH` and
  `DUBBRIDGE_ASR_WORKER_PYTHON` resolving inside the image.
- Base images are pinned to explicit OCI digests; the image evidence records the
  resolved OS package versions and `pip freeze --all` output. The ASR dependency
  input is exact-version pinned in `workers/asr-worker-py/requirements.txt`; no
  floating Python package constraint is permitted in the production image path.
- **Conditional on `S-230-T3b`:** if `S-150-T3b`/`T3c` (functional translation
  worker + its Rust consumer) are `[x] Done` at the time this task executes,
  the worker-runner image also bundles `workers/translation-worker-py` and its
  dependencies, with whichever path/interpreter env vars `T3c` defines for it
  (see `S-230-T3b` §"Downstream coupling"). If not done yet, this task
  proceeds without it and records a follow-up image rebuild as debt rather
  than silently shipping an image that cannot run translation.
- `ASR_MODEL_SIZE` is an explicit build/run parameter; the POC value is `small`,
  not the `large-v3` default (plan G7). The worker startup/run evidence must show
  an incompatible or unavailable selected model fails the job/start path loudly;
  it must not silently fall back to `large-v3` or report readiness for work it
  cannot perform.
- A migration image or entry point derived from T2 that Compose can run as a
  one-shot job.
- Every image starts against local infrastructure and passes its own
  `/health/ready` where applicable.

**Behavioral examples:**

- **HP-1:** With the documented local infrastructure available, the built API,
  gateway, and worker-runner images start using their production entry points;
  API and gateway become ready and a media-preparation-to-ASR flow succeeds
  without invoking `cargo run`.
- **HP-2:** The one-shot migration image applies the current migration set to an
  empty local database and exits successfully.
- **EC-1:** If PostgreSQL, Redis, S3-compatible storage, or the gateway's API
  upstream is unavailable, the corresponding readiness endpoint reports not
  ready while its liveness endpoint stays cheap and independent.
- **EC-2:** If the selected `ASR_MODEL_SIZE=small` cannot be loaded in the image,
  the ASR job/start path fails visibly, never falls back to `large-v3`, and
  never reports false readiness.

**Evidence to emit:** Dockerfiles and exact ASR dependency input; local build and
run transcripts; OCI base-image digests, OS/Python package inventories, image
sizes; a successful local pipeline run using the built images rather than `cargo
run`; RRI evidence; phase reviews; 3-pass Reflection log; unit-coverage
certification; owner verification.

**Status artifacts affected:** this ledger; `DEVELOPMENT_REFERENCE.md` if the
documented local run path changes.

**Handoff prompt:** Author production Dockerfiles for api, gateway, and
worker-runner (with ffmpeg + Python + faster-whisper), plus a migration entry
point. Prove each starts and reports ready locally.

**Stop condition:** Stop after local image verification. Do not provision cloud
resources.

**Sequencing correction (2026-08-17):** the original T4-child decomposition
ordered each pair as contract-test-then-image (e.g. former T4b "API contract
tests" before former T4c "API image"). When T4b was presented for phase-1
review, `muse-glimmer:30b-q4_K_M` returned `BLOCKED`: the contract case had no
concrete port/binary/dependency-simulation detail to test against because no
image existed yet to inspect (`docs/adr` gives no such detail either — it's
implementation-specific to each Dockerfile). Investigated against
`config/default.toml` (`api_port = 8080`) and `config/local.toml`/
`config/staging.toml` (`port = 8081` for gateway) confirmed the finding was a
genuine specification gap, not a false positive to disprove (contrast the
T4a phase-1 correction above, which *was* a disproved false positive). Owner
decision: invert every image/contract-test pair so the image is built first
(with its own manual HP-1/EC-1 evidence) and the harness case then codifies
that evidence as a repeatable regression test against the real image. Applied
to all six pairs: T4b/T4c (API), T4d/T4e (gateway), T4f/T4g (migration),
T4i/T4j (worker native-runtime), T4k/T4l (worker ASR-bundle), T4m/T4n
(translation-bundle, conditional). T4o's dependency on T4c/T4e/T4g/T4l/T4n
(by ID) is unaffected — it already correctly required the contract-test
children, which now correctly land after their images.

### S-230-T4a: Production-image test harness

**Type:** development/test infrastructure
**Effort:** S — RRI 15 Low
**Depends on:** S-230-T3
**Status:** [x] Done — 2026-08-17
**Writable path:** `scripts/test-production-images.sh`

Create a bounded Bash harness with named `contract` and `run` modes, strict
error handling, deterministic cleanup, and an unknown-case failure. Keep it
below 500 lines. **HP-1:** listing and executing a registered contract returns
zero. **EC-1:** an unknown case or missing required argument returns non-zero
without launching Docker. Evidence: RRI artifact; Muse phase reviews; `bash -n`;
named harness self-tests; HP/EC unit certification; owner verification. Status
artifact: this ledger. Handoff: create only the harness foundation; stop before
adding any service-specific image contract.

**RRI:** 15 (Low). Source: `docs/audit/s-230-t4-low-rri-decomposition.md`
(`python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 4
--D 1 --K 2 --P 0 --T 2 --A 0 --X 1`).

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` via
`scripts/delegate-low-rri.py --mode full-file`. Attempt 1 used
`declare -A CASES=( [self-check]=1 )`, which fails on the actual deployment
shell (`GNU bash, version 3.2.57(1)-release (arm64-apple-darwin25)` — macOS
stock `/bin/bash` predates bash 4's associative arrays: `declare: -A: invalid
option`). Root-caused via a minimal repro (`bash -c 'declare -A CASES=(
[self-check]=1 )'` on this shell reproduces the identical failure) before any
re-delegation. The orchestrator revised the frozen interface contract to
mandate a bash-3.2-safe, space-delimited case registry (`CASE_LIST` + `for`
loop membership test) instead, matching every other script already in
`scripts/`, none of which uses `declare -A`. Attempt 2 against the revised
packet produced a working script; this is the accepted implementation (one
bounded repair, within the Low-band's 1-repair budget).

**Phase-1 review correction:** the first phase-1 pass on the revised packet
returned `BLOCKED`, flagging that `contract_self-check()` (a hyphenated bash
function name) is a syntax error on bash 3.2. The orchestrator tested this
claim directly against the target shell before accepting or rejecting it:
`bash -n` and execution both succeed for a hyphenated function name defined,
looked up via `declare -F`, and invoked indirectly via `"$func_name"` — bash
restricts hyphens in *variable* names, not function names. This counter-
evidence was returned to the reviewer, which then reversed to `PASS` and
confirmed the rest of the contract (bash-3.2 constraints, HP-1/EC-1, stop
condition) was otherwise unambiguous. Both the original `BLOCKED` and the
corrected `PASS` are preserved in the review artifacts below rather than
overwritten.

**Post-delegation defect:** the delegation wrapper's tagged-block response
leaked a stray trailing `--- CONTENT ---` marker into the returned file
content (a wrapper/template artifact, not model-authored logic). Stripped
before the file was written to the real repo path; confirmed by line-count
and `bash -n` before and after.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Phase 1 (task-analysis, pre-delegation):
  - Pass 1 (original packet, pre-bash-3.2-fix): `PASS` —
    `t4a-phase1-response-v2.json` (scratchpad, superseded by the packet
    revision below; not applicable to the delegated packet).
  - Pass 2 (revised bash-3.2-safe packet): `BLOCKED` — hyphenated function
    name finding — `t4a-phase1-response-v3.json` (scratchpad).
  - Pass 3 (same packet + orchestrator counter-evidence): `PASS`, findings
    explicitly note the prior claim was falsified by reproducible test —
    `t4a-phase1-response-v4.json` (scratchpad).
- Phase 2 (code-solution, post-implementation): `PASS`, 0 findings —
  `t4a-phase2-response.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass, not the N-pass
  consolidated-aggregate mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at both phases.
- disposition_divergence: `none`
- Primary-agent disposition: phase-1 `BLOCKED` finding investigated and
  disproved with a direct, reproducible bash-3.2 test before re-submission;
  phase-2 raised no findings to disposition.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `contract self-check` exits 0 | `bash scripts/test-production-images.sh contract self-check` → exit 0, printed `Bash version: 3.2.57(1)-release` | passed |
| HP-1 | Happy path | `run self-check` exits 0 | `bash scripts/test-production-images.sh run self-check` → exit 0 | passed |
| EC-1 | Edge case | 0 args fails closed, no docker call | `bash scripts/test-production-images.sh` → exit 1, usage to stderr; `docker` only referenced inside `contract_self-check`/`run_self-check` (lines 47-48, 63-64), unreachable before validation | passed |
| EC-1 | Edge case | 1 arg fails closed | `bash scripts/test-production-images.sh contract` → exit 1 | passed |
| EC-1 | Edge case | invalid mode fails closed | `bash scripts/test-production-images.sh bogus-mode self-check` → exit 1 | passed |
| EC-1 | Edge case | unknown case fails closed | `bash scripts/test-production-images.sh contract nonexistent-case` → exit 1 | passed |

`bash -n scripts/test-production-images.sh` → no syntax errors. File is 106
lines (well under the 500-line ceiling).

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-17`
- Statement: I verified every HP-1 and EC-1 case defined for this task by
  executing the exact invocations above against the real repository file
  `scripts/test-production-images.sh` on the actual deployment shell (bash
  3.2.57), confirmed matching exit codes, confirmed `docker` is never
  reachable before the three-step validation completes, and confirmed the
  phase-1 `BLOCKED` finding was investigated to a reproducible conclusion
  rather than accepted or dismissed without evidence.
- Commands run: `bash -n scripts/test-production-images.sh`; the six HP-1/EC-1
  invocations listed in the coverage table above; `wc -l
  scripts/test-production-images.sh`.

Reviewability budget: not evaluated — this is a single new ~106-line file,
trivially within any derived Low-band review budget; no margin question.

### S-230-T4b: API production image

**Type:** development/config
**Effort:** S — RRI 18 Low
**Depends on:** S-230-T4a
**Status:** [x] Done (structural certification; live runtime verification deferred to T4c)
**Writable path:** `apps/api/Dockerfile`

Create a digest-pinned multi-stage image that builds only `dubbridge-api`
(binary path `/app/dubbridge-api`, bound to `api_port = 8080` per
`config/default.toml`) and contains no Rust toolchain in its runtime stage.
**HP-1:** the image starts against local Compose infrastructure
(`infra/local/docker-compose.yml`) and reaches ready on `/health/ready` (with
`/health/live` also reachable) within a bounded timeout. **EC-1:** stopping a
required local dependency (e.g. Postgres) makes `/health/ready` report
non-200 while `/health/live` remains 200. Evidence: RRI artifact; Muse phase
reviews; exact build/run command; image size and base digest; manual
start/readiness/degraded-dependency transcript; HP/EC certification; owner
verification. Status artifact: this ledger. Stop without changing application
source or other images.

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` (Low-band
developer per `DUBBRIDGE_LOW_RRI_MODEL` default — the ambient shell's
`DUBBRIDGE_LOW_RRI_MODEL=gemma4:26b-a4b-it-qat` override was bypassed via an
explicit `--model qwen3.8:27b-mlx` flag for this delegation only, per owner
instruction; the shell-level override was left untouched for the owner to
address separately) via `scripts/delegate-low-rri.py --mode full-file`.
Attempt 1 at default `--num-predict` (4096) failed with "missing file end
marker" (response truncated before completion, no output file written).
Attempt 2 at `--num-predict 8192` (same packet, no repair — this is the same
delegation call retried at a larger token budget, not a content revision)
produced a complete, valid tagged response.

**Post-delegation defect:** the tagged-block response leaked a trailing
`--- CONTENT ---` marker into the returned file content (a wrapper/template
artifact, matching the T4a precedent — not model-authored logic). Stripped
before the file was written to the real repo path; confirmed by line-count
(39 → 38 lines) before/after.

**No live Docker validation:** the packet explicitly scoped this delegation
to producing a structurally correct Dockerfile only ("You are NOT responsible
for verifying these at runtime — no Docker available in this delegation").
`docker build` was attempted as a best-effort syntax check but hung
indefinitely resolving the placeholder digest against the registry (expected:
`sha256:PLACEHOLDER` is not a resolvable digest) and was killed rather than
left running. HP-1/EC-1 are therefore certified as *structural achievability*
(the Dockerfile's instructions make the criteria achievable given the
already-implemented health endpoints) — not as an executed runtime transcript.
Live start/readiness/degraded-dependency execution against a real resolved
digest remains owner follow-up work before this image is deployed, and is the
literal subject of T4c (which runs the harness `run` mode against this image).

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Phase 1 (task-analysis, pre-delegation):
  - Pass 1 (v1 packet): `PASS`, 1 finding — config-file delivery inside the
    runtime stage was underspecified (how `DUBBRIDGE_CONFIG_DIR` resolves in
    a container) — `t4b_packet_phase1_result.json` (scratchpad).
  - Pass 2 (v2 packet, Requirement 5 added to resolve the finding): `PASS`,
    0 findings — `t4b_packet_v2_phase1_result.json` (scratchpad). Per the
    revised-packet re-review rule, this is a distinct phase-1 event from
    Pass 1, not an overwrite of it.
- Phase 2 (code-solution, post-implementation, against the delegated and
  marker-stripped Dockerfile): `PASS`, 0 findings —
  `t4b_phase2_result.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass, not the N-pass
  consolidated-aggregate mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at both phases.
- disposition_divergence: `none`
- Primary-agent disposition: phase-1 Pass 1 finding resolved by revising the
  packet (added explicit config-directory-copy requirement) and re-submitting
  for its own fresh phase-1 pass, which returned 0 findings; phase-2 raised
  no findings to disposition.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | image structurally reaches `/health/ready` + `/health/live` given Compose infra | Structural review of `apps/api/Dockerfile`: `ENTRYPOINT ["/app/dubbridge-api"]` runs the real `dubbridge-api` binary (unmodified Rust source, health endpoints already implemented and out of this task's scope) with `DUBBRIDGE_CONFIG_DIR=/app/config` pointing at the copied `config/` tree so `api_port = 8080` resolves per `config/default.toml`; no code path in the image alters or bypasses the existing health-check logic. Not an executed runtime transcript — see "No live Docker validation" above. | structurally certified (not executed) |
| EC-1 | Edge case | liveness independent of downstream deps; readiness dependent | Same structural basis: the image does not implement or wrap health-check logic itself (`ENTRYPOINT` invokes the unmodified binary directly), so the existing HP/EC-1 behavior certified for the binary in prior tasks is preserved unchanged by this image's construction. Not independently executed against a running container in this delegation. | structurally certified (not executed) |

**Certification caveat (explicit, not a pass/fail evasion):** both cases above
are certified as *structural achievability* only, per the packet's own scope
("You are NOT responsible for verifying these at runtime — no Docker
available in this delegation"). No `docker run` against a resolved image was
executed — `docker build` was attempted and hung on the intentionally
unresolvable placeholder digest, then was killed. This differs from T4a's
unit coverage certification (a real `bash` script executed with real exit
codes) and from typical Rust unit tests: there is no lower-level test harness
that can exercise "does this image reach ready" without an actual container
runtime and a resolved base-image digest. Live execution of both cases is the
explicit, named subject of S-230-T4c (`run` mode against this image) and
remains owner follow-up before deployment.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-17`
- Statement: I verified the delegated `apps/api/Dockerfile` against every
  packet requirement by direct file inspection (multi-stage structure,
  placeholder-digest TODO comments present on both `FROM` lines, no Rust
  toolchain copied into the runtime stage, `WORKDIR /app` +
  `COPY --from=builder .../dubbridge-api /app/dubbridge-api` +
  `ENTRYPOINT ["/app/dubbridge-api"]`, `config/` copied and
  `DUBBRIDGE_CONFIG_DIR` set, `DUBBRIDGE_ENV` left unset, `EXPOSE 8080`
  matching `config/default.toml`). I confirmed the stripped marker artifact
  did not alter file line-count beyond the marker removal itself (39 → 38
  lines). I confirmed HP-1/EC-1 cannot be certified as executed in this
  environment (no live Docker validation was possible against the
  intentionally placeholder digest) and recorded that gap explicitly rather
  than certifying an untested runtime claim. I confirmed no other file in the
  writable-path boundary was touched (`git status` scoped to `apps/api/`).
- Commands run: `python3 -c "..."` (marker-strip + line-count check, ad hoc);
  `docker build -t dubbridge-api-t4b:test apps/api/` (killed — hung resolving
  placeholder digest, expected and non-blocking per packet scope); `git
  status apps/api/`.

Reviewability budget: not evaluated — this is a single new 38-line file,
trivially within any derived Low-band review budget; no margin question.

### S-230-T4c: API image contract tests

**Type:** development/test
**Effort:** S — RRI 17 Low
**Depends on:** S-230-T4b
**Status:** [x] Done — 2026-08-17
**Writable path:** `scripts/test-production-images.sh` (plus two
owner-authored mechanical fixes outside that path — see "Scope note" below)

Add named API contract/runtime cases that codify T4b's manual evidence as a
repeatable harness case, run against the image built in T4b. **HP-1:**
contract mode verifies the expected `dubbridge-api` binary path
(`/app/dubbridge-api`), bound port (`8080`), and that `/health/live`
and `/health/ready` both return 200 against the running T4b image with local
dependencies up. **EC-1:** runtime mode rejects an image with no executable at
the expected path, or detects that stopping a required dependency (matching
T4b's EC-1 transcript) makes `/health/ready` non-200 while liveness stays
available — and fails the harness case if readiness stays 200 instead.
Evidence: RRI artifact; Muse phase reviews; harness tests executed against the
real T4b image; HP/EC certification; owner verification. Status artifact: this
ledger. Stop before modifying `apps/api/Dockerfile` or any other image.

**RRI:** 17 (Low). `python3 scripts/rri.py --touches
scripts/test-production-images.sh --cc 5 --D 1 --K 2 --P 1 --T 2 --A 0 --X 1`
(ledger's original planning estimate was 16; re-run at implementation time
scored 17 — same band, no route change).

**Scope note (why this task also touched two files outside its declared
writable path):** T4b's own certification was explicitly *structural, not
executed* — its Dockerfile shipped with `sha256:PLACEHOLDER` digests that
made `docker build` unresolvable, and T4b named this task as the place where
real runtime evidence would be produced. Making that possible required, in
order: (1) resolving real OCI digests for `apps/api/Dockerfile`'s two `FROM`
lines (owner-authored mechanical edit, single known-value substitution per
line, no delegation — verified via `docker manifest inspect
rust:1-bookworm`/`debian:bookworm-slim` for `linux/arm64`); (2) discovering
and fixing a previously-unknown repo defect: no `.dockerignore` existed
anywhere in the repo, so `COPY . /usr/src/app` in `apps/api/Dockerfile`
shipped the entire working tree (98GB, dominated by `target/` 51GB and
`.agent/` 33GB) as Docker build context, hanging the build. Added
`/Users/matias/dubbridge/.dockerignore` (owner-authored, mechanical,
new file — excludes `target`, `mobile`, `.agent`, `.antares-runtime`,
`.venv-antares-t1`, `.serena`, `node_modules`, `.git`, `logs`, and other
non-source directories). Both are recorded as the "documented
tooling-failure exception" / "mechanical lint-driven refactor" carve-outs in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
decomposition` — root-caused with direct, reproducible evidence
(`docker manifest inspect`, `du -sh`, `Sending build context to Docker
daemon` log lines) before either edit, not authored as new application
logic.

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` via
`scripts/delegate-low-rri.py --mode before-after`, two separate packets
against the same target file (each anchored on a small, verified-unique
BEFORE block, per the file's existing 106-line size — well under the
500-line target-file gate):

- **Packet A** — add `"api"` to `CASE_LIST`. Phase-1 `PASS` on first
  submission, 0 findings. Delegated and applied without repair.
- **Packet B** — add `contract_api()`/`run_api()`. Phase-1 v1 returned
  `BLOCKED` (5 findings: unverified/hardcoded network-name guess, missing
  `--network` enforcement, insufficient cleanup guarantees, missing bash 3.2
  polling constraints, no dependency-up precondition). All 5 were
  investigated and resolved with verified facts before resubmission — not
  disproved, unlike the T4a phase-1 correction precedent; here the reviewer
  was correct and the packet was revised. Notably, the packet's own
  assumed network name (`infra_local_default`) was checked directly against
  the running compose stack and found wrong (actual: `local_default`,
  confirmed via `docker inspect local-postgres-1`), so the fix could not
  have been a guess-and-hope resubmission. Packet B v2 phase-1: `PASS`,
  0 findings. Delegated and applied without repair (1/1 attempt each
  packet, within the Low-band 1-repair budget with no repair consumed).

**Post-delegation defects found and fixed (mechanical, root-caused before
fixing, both inside the declared writable path):**

1. A minor whitespace diff on two *unchanged* comment lines inside the
   verbatim-preserved `run_self-check()` block (the model reproduced them
   with one extra leading space). Stripped before applying, confirmed by
   direct string comparison against the original file.
2. **Pre-existing IFS bug exposed by this task, not introduced by it:** the
   file's line 3 sets `IFS=$'\n\t'` globally (inherited unchanged from T4a).
   `case_exists()`'s `for c in $CASE_LIST` relies on IFS word-splitting on
   space; with only one case (`"self-check"`) in T4a this never manifested.
   Adding a second case (`"self-check api"`) exposed it: `case_exists api`
   always returned false, so `run scripts/test-production-images.sh contract
   api` fell through to `usage()`/exit 1 instead of dispatching. Root-caused
   with `bash -x` before fixing (trace showed `for c in '$CASE_LIST'`
   iterating the whole string as one token). Fix: save/restore `IFS` locally
   inside `case_exists()`, setting `IFS=' '` only for its own `for` loop —
   1 function, 4 added lines, no change to global `IFS` or any other
   function's behavior. Regression-verified: T4a's `self-check` HP-1/EC-1
   cases re-ran unchanged and still pass.
3. **False-positive exit code under `set -e` + `trap ... RETURN`:** first
   full `run api` execution printed `"Run check passed for api"` (correct)
   but the script's actual exit code was `1` (wrong — would have made this
   harness case silently report success while actually failing any
   automated caller checking `$?`, a fail-open bug in a fail-closed test
   harness). Root-caused by isolated repro outside the repo (confirmed the
   pattern in 3 minimal `bash -c` scripts before touching the real file):
   `trap cleanup_api RETURN` runs `cleanup_api()` on every return from
   `run_api()`; under `set -e`, if any command inside that cleanup fails
   (here: the second `docker stop` on the `--rm` api container, which had
   already self-removed after the first successful `docker stop`), `set -e`
   aborts the whole script, and that abort's exit code overwrites
   `run_api()`'s real `return 0`. Fix: `|| true` on both commands inside
   `cleanup_api()` — cleanup best-effort, never lets a harmless cleanup
   failure clobber the case function's real result. Re-verified end-to-end:
   `run api` now exits `0` on success, confirmed over 2 consecutive runs
   (idempotent), no leaked `dubbridge-api-contract-test` container, and
   `local-postgres-1` correctly restored to `Up` after each run.

**Live execution evidence (real image, real infra, not structural-only):**
built `dubbridge-api-t4c:test` from the now-digest-pinned
`apps/api/Dockerfile` (`docker build -f apps/api/Dockerfile -t
dubbridge-api-t4c:test .` — succeeded after the `.dockerignore` fix,
14.74MB context, image ID `697a5a13a6d0`, `ENTRYPOINT
["/app/dubbridge-api"]`, `ExposedPorts 8080/tcp` confirmed via `docker
inspect`). Ran against the real local Compose stack
(`local-postgres-1`/`local-redis-1`/`local-minio-1`, network
`local_default`, resolved dynamically at runtime — not hardcoded). EC-1's
negative path (readiness must degrade, not silently stay 200) was also
independently verified outside the harness with a manual container +
`docker stop local-postgres-1` + `curl -w '%{http_code}'`: `200` before
stop, `503` after stop on `/health/ready`, `200` throughout on
`/health/live`.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Phase 1 (task-analysis, pre-delegation):
  - Packet A, Pass 1: `PASS`, 0 findings — `t4c-packetA-review-resp.json`
    (scratchpad).
  - Packet B v1: `BLOCKED`, 5 findings (network-name guess unverified,
    missing `--network` enforcement, insufficient cleanup guarantees,
    missing bash 3.2 polling constraints, no dependency-up precondition) —
    `t4c-packetB-review-resp3.json` (scratchpad). First attempt at this
    review (against the pre-fix packet) returned 0 bytes twice under
    concurrent load from an unrelated `docker build` consuming host
    memory/CPU — treated as a capacity symptom per the local
    resource-recovery protocol (checked `vm_stat`: ~60MB free), not a
    content failure; the usable `BLOCKED` verdict was obtained only after
    the competing build was killed.
  - Packet B v2 (all 5 findings resolved with verified facts, not
    reassertion): `PASS`, 0 findings — `t4c-packetB-v2-review-resp.json`
    (scratchpad). Distinct phase-1 event from v1 per the revised-packet
    re-review rule, not an overwrite.
- Phase 2 (code-solution, post-implementation, against the full diff
  including the two mechanical fixes and the applied delegation output):
  `PASS`, 1 low-severity finding (`contract_api` doesn't statically check
  for health-route declarations in the Dockerfile; accepted as
  non-blocking — health routes are Rust application code outside the
  Dockerfile's own text, and `run_api`'s live HP-1/EC-1 checks already
  exercise them at runtime) plus 3 `info`-level notes confirming the IFS
  fix, the `cleanup_api` `|| true` fix, and the digest-pin edit were each
  correctly scoped — `t4c-phase2-resp.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass, not the N-pass
  consolidated-aggregate mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  (after one capacity-related retry on Packet B v1) produced usable
  verdicts at every phase.
- disposition_divergence: `none`
- Primary-agent disposition: Packet B v1's 5 findings independently
  verified against the real compose stack/host before the v2 rewrite (the
  network-name claim was checked with `docker inspect`, not just
  reasserted); phase-2's low finding accepted as non-blocking with stated
  rationale; the 3 info notes required no action.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts at
  every attempt.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `contract api` verifies binary path/port from Dockerfile text | `bash scripts/test-production-images.sh contract api` → exit 0, printed "Contract check passed for api" (grep-verified `ENTRYPOINT ["/app/dubbridge-api"]` and `EXPOSE 8080` present) | passed |
| HP-1 | Happy path | `run api` reaches `/health/live` and `/health/ready` = 200 with real T4b image + local deps up | `bash scripts/test-production-images.sh run api` against `dubbridge-api-t4c:test` (built from the digest-pinned `apps/api/Dockerfile`) on network `local_default` → exit 0, printed "Run check passed for api"; re-run twice for idempotency, both exit 0, no leaked container, `local-postgres-1` restored `Up` after each run | passed |
| EC-1 | Edge case | stopping `local-postgres-1` makes `/health/ready` non-200 while `/health/live` stays available; harness fails the case if readiness stays 200 | Exercised inside `run_api()`'s own EC-1 block (same execution as the HP-1 row above — the function returns non-zero if readiness doesn't degrade or liveness doesn't survive). Independently cross-checked outside the harness: manual container on `local_default` + `docker stop local-postgres-1` + `curl -s -o /dev/null -w '%{http_code}'` → `/health/ready` `200`→`503`, `/health/live` `200` throughout | passed |

`bash -n scripts/test-production-images.sh` → no syntax errors. File is 236
lines (well under the 500-line ceiling). Full regression re-run: T4a's
`contract self-check`/`run self-check` HP-1/EC-1 cases (0 args, 1 arg,
invalid mode, unknown case → all exit 1; `self-check` cases → exit 0) all
still pass unchanged after this task's edits.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies;
  scope-affecting decisions — digest resolution ownership, `.dockerignore`
  addition, and both mechanical bash fixes — were each confirmed with the
  human operator before being applied, per this session's exchange)
- Date: `2026-08-17`
- Statement: I verified every HP-1 and EC-1 case defined for this task by
  executing the exact invocations above against the real repository file
  `scripts/test-production-images.sh` on the actual deployment shell (bash
  3.2.57), against the real image built from `apps/api/Dockerfile`, against
  the real local Compose infrastructure — not a structural/static
  certification. I confirmed both post-delegation mechanical defects (IFS
  word-splitting bug, `set -e`/`RETURN`-trap false-positive exit code) with
  isolated, reproducible before-fix repros, then confirmed the fix resolved
  each without re-breaking any T4a regression case. I confirmed no test
  container or state was leaked (`docker ps -a` clean) and the local
  dependency stack was left running in its original state. I confirmed the
  writable-path deviation (touching `apps/api/Dockerfile` and adding
  `.dockerignore`) was authorized by the human operator before being made,
  each edit was mechanical (known-value substitution / new ignore-file,
  not new application logic), and both are recorded here with root-cause
  evidence rather than asserted.
- Commands run: `bash -n scripts/test-production-images.sh`; the full
  regression + HP-1/EC-1 invocations listed in the coverage table above;
  `docker manifest inspect rust:1-bookworm` / `debian:bookworm-slim`;
  `docker build -f apps/api/Dockerfile -t dubbridge-api-t4c:test .`;
  `docker inspect dubbridge-api-t4c:test`; `docker inspect
  local-postgres-1`; the manual EC-1 cross-check (`docker run` +
  `docker stop local-postgres-1` + `curl -w '%{http_code}'`); `docker ps
  -a` (leak check) before and after.

Reviewability budget: not evaluated — this task's total diff (harness +
digest-pin edit + new `.dockerignore`) is a few hundred lines across three
files, well within any derived Low-band review budget; no margin question.

### S-230-T4d: Gateway production image

**Type:** development/config
**Effort:** S — RRI 18 Low
**Depends on:** S-230-T4a
**Status:** [x] Done — 2026-08-17
**Writable path:** `apps/gateway/Dockerfile`

Create a digest-pinned multi-stage image that builds only `dubbridge-gateway`
(binary path `/usr/local/bin/dubbridge-gateway`, bound to `port = 8081` per
`config/local.toml`/`config/staging.toml`) and has no build toolchain in the
runtime stage. **HP-1:** the image starts against a healthy T4b API instance
and reaches ready. **EC-1:** stopping the API upstream changes gateway
readiness to non-ready while gateway liveness remains available (liveness must
not depend on the upstream). Evidence: RRI artifact; Muse phase reviews;
build/run command; image size/base digest; start/readiness/degraded-upstream
transcript; HP/EC certification; owner verification. Status artifact: this
ledger. Stop without changing gateway source or the production descriptor.

**Binary path correction:** the actual binary path is `/app/dubbridge-gateway`
(matching `/app/dubbridge-api` in the already-approved `apps/api/Dockerfile`
pattern, `WORKDIR /app`), not `/usr/local/bin/dubbridge-gateway` as the
original task text stated — the same descriptive imprecision T4b's actual
implementation corrected for the API image. `EXPOSE 8081` and the
`config/local.toml`/`config/staging.toml` `[gateway] port = 8081` binding are
otherwise exactly as specified.

**RRI:** 9 (Low). `python3 scripts/rri.py --touches apps/gateway/Dockerfile
--cc 1 --D 1 --K 1 --P 1 --T 0 --A 0 --X 1` (ledger's original planning
estimate was 18; re-run at implementation time scored 9 — same band, no route
change; this task is a closer mechanical mirror of the already-proven T4b
pattern than T4b itself was, hence the lower score).

**Local-stack precheck (workflow Step 0):** restarted Ollama before this
task's first local-model call (old PID `97741` terminated, new PID `74630`
confirmed via `pgrep -fl ollama` and a fresh `lsof -iTCP:11434 -sTCP:LISTEN`
listener). Warmed and confirmed both `muse-glimmer:30b-q4_K_M` (Low-band
phase-1/phase-2 reviewer) and `qwen3.8:27b-mlx` (Low-band developer) with a
production-parameter JSON-only probe (`think=false`, `num_predict=4096`,
`num_ctx=65536`): both returned `done_reason: "stop"` with valid non-empty
content.

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` via
`scripts/delegate-low-rri.py --mode full-file` (new file — before-after mode
does not apply). Packet built `apps/api/Dockerfile`'s full content as the
proven reference pattern plus itemized gateway-specific substitutions
(package name, binary path, port, two comment lines), each backed by
independently-verified evidence (see phase-1 v1→v3 below) rather than
asserted. Single delegation attempt, exit 0, valid unified diff produced on
the first try — no repair cycle needed.

**Post-delegation defects found and fixed (mechanical, root-caused before
fixing):**

1. One stray extra space on the `apt-get` continuation line before `&& rm -rf
   /var/lib/apt/lists/*` (5 spaces instead of the reference pattern's 4 —
   confirmed byte-for-byte via `od -c` against `apps/api/Dockerfile` lines
   15-18). Corrected to match the reference exactly.
2. Missing trailing newline on the generated file. Added.

Every other line matched the reference pattern's structure, ordering, and
substitution requirements exactly — no other correction was needed.

**Live execution evidence (real image, real infra, not structural-only):**
built `dubbridge-gateway-t4d:test` from `apps/gateway/Dockerfile` (`docker
build -f apps/gateway/Dockerfile -t dubbridge-gateway-t4d:test .` — succeeded,
image ID `a57cfd0d9046`, 160MB / 34497819 bytes, `Entrypoint
[/app/dubbridge-gateway]`, `ExposedPorts 8081/tcp` confirmed via `docker
inspect`). Ran two real containers on Docker's `host` network (chosen because
`config/local.toml`'s `[gateway] upstream_api_base_url =
"http://localhost:8080"` is a fixed TOML value with no environment-variable
override in `crates/config`, and `host` networking lets that literal
`localhost:8080` correctly resolve to the separately-running API container
without touching any file outside this task's `apps/gateway/Dockerfile`
writable path): the T4c-built `dubbridge-api-t4c:test` image as the upstream,
then `dubbridge-gateway-t4d:test`. HP-1: `/health/live` → `200`,
`/health/ready` → `200` with body `{"service":"gateway","status":"ready",
"component":"api","component_status":"ok"}`. EC-1: after `docker stop` on the
API upstream container, `/health/live` stayed `200` while `/health/ready`
transitioned to `503` with body `{"service":"gateway","status":"not_ready",
"component":"api","component_status":"unreachable"}` — confirming readiness
depends on the upstream and liveness does not, exactly as required. Both test
containers removed after verification (`docker ps -a` confirmed 0 leftover
`t4d` containers); local Compose infra
(`local-postgres-1`/`local-redis-1`/`local-minio-1`) confirmed untouched and
still `Up` throughout.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Phase 1 (task-analysis, pre-delegation):
  - v1 packet: `BLOCKED`, 5 findings (digest validity/working-status
    asserted without evidence; gateway port/config-loading mechanism
    asserted without evidence; unconstrained `COPY . /usr/src/app` build
    context; vague comment-substitution instructions; acceptance criteria
    not fully achievable from packet alone) —
    `t4d-phase1-resp.json` (scratchpad).
  - v2 packet (digest/port/config claims backed with direct verification
    evidence — `docker images --digests`, `config/staging.toml` read,
    `apps/gateway/src/main.rs` read; build-context and acceptance-scope
    notes added): `BLOCKED`, 3 findings (binary-name-equals-package-name
    rule still asserted without a direct check; missing constraint against
    copying extra files from the builder stage; blank-line-map ambiguity
    for comment preservation) — `t4d-phase1-v2-resp.json` (scratchpad). Per
    the revised-packet re-review rule, a distinct phase-1 event from v1, not
    an overwrite.
  - v3 packet (`[[bin]]`-absence verified by direct `grep` against both
    `apps/gateway/Cargo.toml` and `apps/api/Cargo.toml`; explicit
    single-`COPY --from=builder` constraint and line-for-line-except-listed-
    substitutions requirement added): `PASS`, 0 findings —
    `t4d-phase1-v3-resp.json` (scratchpad). Distinct phase-1 event from v1
    and v2.
- Phase 2 (code-solution, post-implementation, against the final corrected
  `apps/gateway/Dockerfile` plus the live HP-1/EC-1 transcript, compared
  against the reference pattern): `PASS`, 0 findings —
  `t4d-phase2-resp.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass, not the N-pass
  consolidated-aggregate mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at every phase.
- disposition_divergence: `none`
- Primary-agent disposition: phase-1 v1's 5 findings and v2's 3 findings were
  each resolved by adding independently-verified evidence to the packet
  (not by reassertion) and re-submitted for their own fresh phase-1 pass;
  phase-2 raised no findings to disposition.
- REVIEW-OVERRIDE: not used — all three phase-1 attempts and phase-2 have
  artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | image starts against a healthy API instance and reaches ready | `docker build -f apps/gateway/Dockerfile -t dubbridge-gateway-t4d:test .` then `docker run --network host dubbridge-api-t4c:test` (upstream) + `docker run --network host dubbridge-gateway-t4d:test` → `curl http://localhost:8081/health/live` = `200`, `curl http://localhost:8081/health/ready` = `200` with `component_status: "ok"` | passed |
| EC-1 | Edge case | stopping the API upstream makes readiness non-ready while liveness stays available | Same running containers, `docker stop dubbridge-api-t4d-upstream` then re-checked: `/health/live` = `200` (unchanged), `/health/ready` = `503` with `component_status: "unreachable"` | passed |

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-17`
- Statement: I verified `apps/gateway/Dockerfile` against every packet
  requirement by direct file inspection and independently re-verified the
  two post-delegation mechanical defects (extra continuation-line space,
  missing trailing newline) with a byte-level comparison (`od -c`) against
  the approved `apps/api/Dockerfile` reference before correcting them. I
  executed a real `docker build` producing a working image, then executed
  both HP-1 and EC-1 as live container transcripts against the real T4c API
  image and real Docker networking — not a structural-only certification
  (unlike T4b, which was scoped structural-only pending this and the
  now-available T4c-proven digest/build path). I confirmed the `host`
  network choice was necessary only because `upstream_api_base_url` has no
  env-var override in `crates/config` and is out of this task's writable
  path — not a deviation from the task's HP-1/EC-1 intent. I confirmed no
  test container was leaked (`docker ps -a` clean of `t4d` containers) and
  the local dependency stack (`local-postgres-1`/`local-redis-1`/
  `local-minio-1`) was left running, unmodified, throughout. I confirmed no
  file outside `apps/gateway/Dockerfile` was touched (`git status` scoped to
  `apps/gateway/`).
- Commands run: `pgrep -fl ollama`; `lsof -iTCP:11434 -sTCP:LISTEN`; Ollama
  warm-up probes for both models; `python3 scripts/delegate-low-rri.py
  --mode full-file ...`; `od -c` byte comparison against
  `apps/api/Dockerfile`; `docker build -f apps/gateway/Dockerfile -t
  dubbridge-gateway-t4d:test .`; `docker inspect dubbridge-gateway-t4d:test`;
  `docker run -d --name dubbridge-api-t4d-upstream --network host -e
  DUBBRIDGE_ENV=local dubbridge-api-t4c:test`; `docker run -d --name
  dubbridge-gateway-t4d --network host -e DUBBRIDGE_ENV=local
  dubbridge-gateway-t4d:test`; `curl -s -o /dev/null -w '%{http_code}'
  http://localhost:8081/health/live`; same for `/health/ready`; `docker stop
  dubbridge-api-t4d-upstream`; re-run of both curl checks; `docker rm -f
  dubbridge-gateway-t4d dubbridge-api-t4d-upstream`; `docker ps -a` (leak
  check) before and after; `git status apps/gateway/`.

Reviewability budget: not evaluated — this is a single new 36-line file,
trivially within any derived Low-band review budget; no margin question.

### S-230-T4e: Gateway image contract tests

**Type:** development/test
**Effort:** S — RRI 21 Low
**Depends on:** S-230-T4d
**Status:** [x] Done
**Writable path:** `scripts/test-production-images.sh`

Add named gateway contract/runtime cases that codify T4d's manual evidence as
a repeatable harness case, run against the image built in T4d. **HP-1:**
contract mode verifies the expected `dubbridge-gateway` binary path, bound
port (`8081`), and liveness/readiness paths respond against the running T4d
image with a healthy API. **EC-1:** runtime mode detects that an unavailable
API upstream (matching T4d's EC-1 transcript) makes gateway readiness fail
without making liveness dependent on that upstream, and fails the harness
case if readiness stays healthy instead. Evidence: RRI artifact; Muse phase
reviews; harness tests executed against the real T4d image; HP/EC
certification; owner verification. Status artifact: this ledger. Stop before
modifying `apps/gateway/Dockerfile` or any other image.

Added `contract_gateway()`/`run_gateway()` to
`scripts/test-production-images.sh`, mirroring `contract_api()`/`run_api()`.
`contract_gateway()` checks `apps/gateway/Dockerfile` for
`ENTRYPOINT ["/app/dubbridge-gateway"]` and `EXPOSE 8081`. `run_gateway()`
starts `dubbridge-api-t4c:test` as an upstream dependency (`--network host`,
since `config/local.toml`'s `upstream_api_base_url` is a fixed
`http://localhost:8080` with no env override; connection strings use
`localhost` against the docker-compose published host ports 5432/6379/9000,
since bridge-network service hostnames are unreachable under host
networking), then `dubbridge-gateway-t4d:test`, polls `:8081/health/live`
and `/health/ready`, and for EC-1 stops the API dependency container and
re-checks readiness (must fail) and liveness (must still pass).
`cleanup_gateway()` mirrors `cleanup_api()`'s `|| true`-guarded RETURN trap.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0–25 band primary, both phases)
- Task-analysis review (Phase 1): trimmed packet
  (`t4e-phase1-review-prompt-v2.txt`, ~30 lines) after the original packet
  stalled; retried after the local resource-recovery protocol
  (kill → unload → memory-pressure check → bounded probe → rebuilt packet).
  Verdict: `PASS`, no findings.
- Code-solution review (Phase 2): trimmed packet
  (`t4e-phase2-review-prompt-v2.txt`, 172 lines: terse header + full 161-line
  diff) after the original 187-line packet stalled the same way; same
  recovery protocol applied, retried at `num_ctx=16384`, `num_predict=1536`,
  `temperature=0`, `think=false`. Completed in 22.2s. Verdict: `PASS`, no
  findings (`{"verdict":"PASS","findings":[]}`).
- Passes run / usable: `1/1` both phases (single-pass, not the N-pass
  consolidated mode).
- Muse Glimmer fallback: not triggered (primary model itself, after packet
  resizing, produced usable results both times).
- D14 fallback: not triggered.
- disposition_divergence: `n/a` (no findings to reconcile).
- Primary-agent disposition: accepted both PASS verdicts as-is.

### Reflection log

Not a separate section for RRI 0–25 (folded into the mandatory review step
above per `AGENT_WORKFLOW_GUIDE.md`). During personal review of the
delegated implementation (prior to the Phase 2 packet above), three defects
were found and fixed before acceptance: (1) `git apply` fuzzy-context scope
creep on Packet A touched unrelated whitespace outside the declared anchor —
applied the single intended line change manually instead; (2) Packet B
delegation attempt 1 returned a mismatched `PATH` header, rejected by
`delegate-low-rri.py`'s own validation, and attempt 2 (bounded repair cycle)
returned the correct path; (3) the accepted Packet B content used
env-var passthrough (`-e VAR="${VAR}"`, evaluating to empty strings) instead
of `run_api()`'s actual hardcoded-literal pattern, and used bridge-network
hostnames unreachable under `--network host` — both rewritten by hand after
verifying `run_api()`'s real pattern directly (`scripts/test-production-images.sh`
lines 142–155) and cross-checking `infra/local/docker-compose.yml`'s
published host ports. A fourth defect (`DUBBRIDGE_ENV` unset, causing the
gateway container to fail closed per ADR-026) was found during live
execution, not static review, and fixed by cross-referencing T4d's own
documented live-execution evidence in this ledger.

### Unit coverage certification

This task adds a bash test-harness case rather than Rust unit-testable
logic; HP-1/EC-1 evidence is the live execution transcript against the real
T4c/T4d images, following the same pattern T4c/T4d used for their own
closure records.

| Case ID | Type | Behavior | Evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | contract checks Dockerfile ENTRYPOINT+EXPOSE; gateway `:8081/health/live` and `/health/ready` return 200 against `dubbridge-gateway-t4d:test` with a healthy `dubbridge-api-t4c:test` upstream | `bash scripts/test-production-images.sh contract gateway` and `bash scripts/test-production-images.sh run gateway`, both exit 0, live against real images | passed |
| EC-1 | Edge case | after stopping the API dependency container, gateway `/health/ready` fails and `/health/live` still passes | same live `run gateway` execution, EC-1 block observed: readiness check correctly fails, liveness check correctly still passes | passed |

Additional regression check: `bash scripts/test-production-images.sh run api`
still passes (no regression to the pre-existing `api` case); `bash -n
scripts/test-production-images.sh` syntax check passes on bash 3.2.57; no
orphaned containers after either run (cleanup traps verified).

Reviewability budget: not applicable (Low band; diff line count not the
constraint encountered — see Reflection log for the actual packet-size
issue with the *review* step, which is separate from the formal
`qa-review-budget` gate).

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record, RRI 0–25 direct
  execution band)
- Date: `2026-08-19`
- Statement: I verified every happy path and edge case defined for this
  task has live-execution evidence that replicates the expected behavior
  against the real `dubbridge-api-t4c:test` and `dubbridge-gateway-t4d:test`
  images, with no regression to the pre-existing `self-check`/`api` cases.
- Commands run: `bash -n scripts/test-production-images.sh`;
  `bash scripts/test-production-images.sh contract self-check`;
  `bash scripts/test-production-images.sh contract api`;
  `bash scripts/test-production-images.sh contract gateway`;
  `bash scripts/test-production-images.sh run api`;
  `bash scripts/test-production-images.sh run gateway`

### S-230-T4f: Migration production image

**Type:** development/config
**Effort:** S — RRI 13 Low (recomputed at task-presentation time via
`scripts/rri.py`; corrects the ledger's provisional RRI 17 estimate, same
band, no gate change)
**Depends on:** S-230-T4a, S-230-T2
**Status:** [x] Done — 2026-08-20
**Writable path:** `apps/cli/Dockerfile`

Create a digest-pinned multi-stage one-shot image for `dubbridge-cli`,
including the compile-time migration directory and no runtime Rust toolchain.
**HP-1:** it applies the current migrations to an empty local database (via
`infra/local/docker-compose.yml` Postgres) and exits zero. **EC-1:** an
unreachable database exits non-zero. Evidence: RRI artifact; Muse phase
reviews; image size/base digest; empty-DB and rerun transcripts; HP/EC
certification; owner verification. Status artifact: this ledger. Stop without
changing migrations or CLI source.

**RRI:** 13 (Low). `python3 scripts/rri.py --touches apps/cli/Dockerfile --cc
3 --D 1 --K 1 --P 0 --T 2 --A 0 --X 1` — single new config file, no anchor-
rubric match, no penalties. Base value 13 -> band Low -> local delegation
route.

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` (Low-band
developer, `DUBBRIDGE_LOW_RRI_MODEL` default) via
`scripts/delegate-low-rri.py --mode full-file --target-path
apps/cli/Dockerfile`. The packet fixed the exact requirements (base image
digests identical to `apps/api/Dockerfile`/`apps/gateway/Dockerfile`,
`dubbridge-cli` package/binary name, `COPY config/` +
`ENV DUBBRIDGE_CONFIG_DIR=/app/config` per `AppConfig::load()`'s fail-closed
loader, no `infra/migrations/` copy since `sqlx::migrate!` embeds migrations
at compile time, no `EXPOSE`/`CMD` since this is a one-shot job) against the
verified contents of `apps/cli/src/main.rs` and the reviewed sibling
`apps/gateway/Dockerfile`. Attempt 1 (default `--num-predict 4096`) produced
a complete, valid tagged response with no repair needed — the fully-specified
packet and close sibling precedent (T4b/T4d) left no ambiguity for the model
to resolve.

**Post-delegation defect:** the tagged-block response left a stray trailing
`---` template marker after `ENTRYPOINT` and an extra leading space before
`&&` in the `apt-get` line (matching the same class of wrapper/template
artifact documented in T4a/T4b — not model-authored logic). Both stripped
before verification; confirmed by diff (2 cosmetic line edits, no semantic
change) — this is the documented tooling-failure exception for a direct
orchestrator edit, not an undocumented bypass of local authorship.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Phase 1 (task-analysis, pre-delegation): `PASS`, 0 findings —
  `t4f_phase1_result.json` (scratchpad). Packet included the verified
  `main.rs` contents, the `AppConfig::load()` config-dir requirement, the
  `sqlx::migrate!` compile-time-embedding fact, and the full reviewed
  `apps/gateway/Dockerfile` as structural reference.
- Phase 2 (code-solution, post-implementation, against the final file
  content plus independently-executed HP-1/EC-1/rerun transcripts):
  `PASS`, 0 findings — `t4f_phase2_result.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at both phases.
- disposition_divergence: `none`
- Primary-agent disposition: no findings to disposition at either phase.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | migrations apply to empty DB, exit 0 | `docker build -f apps/cli/Dockerfile -t dubbridge-cli:t4f-test .` (160MB, id `sha256:cb2495c02a71ba9a23a2099bb99cb224eb2f74944799cc9a22ba9bd80c00cd87`) then `docker run --rm --network local_default -e DUBBRIDGE_ENV=local -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@local-postgres-1:5432/t4f_empty_test dubbridge-cli:t4f-test` against a freshly created empty database (`CREATE DATABASE t4f_empty_test`) → exit 0, log lines "applying migrations" then "migrations applied successfully", 27 tables created including `_sqlx_migrations` | passed |
| HP-1 | Happy path | rerun is idempotent | same `docker run` invocation repeated against the now-migrated database → exit 0, log includes `relation "_sqlx_migrations" already exists, skipping` then "migrations applied successfully" (no partial/duplicate application) | passed |
| EC-1 | Edge case | unreachable database exits non-zero | same image run with `DUBBRIDGE_DATABASE_URL` pointing at an unresolvable hostname → exit 1, stderr `Error: database connection failed: ... failed to lookup address information: Name or service not known`, no "migrations applied successfully" line | passed |

Runtime stage verified to contain no Rust toolchain (`debian:bookworm-slim`
base with only `ca-certificates`/`libssl3` installed; binary copied from the
discarded builder stage). Both stages pinned by `sha256` digest, matching the
already-reviewed `apps/api`/`apps/gateway` digests. Test database dropped
after verification (`DROP DATABASE t4f_empty_test`).

Reviewability budget: not evaluated — 32-line single new file, trivially
within any derived Low-band review budget; no margin question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-20`
- Statement: I verified HP-1 and EC-1 by building the real image from
  `apps/cli/Dockerfile` and running it with Docker against a real,
  independently-created empty Postgres database on the local Compose
  infrastructure, confirming exit codes and full log/table-creation output
  directly rather than accepting the delegated file at face value; I also
  independently identified and corrected the two cosmetic wrapper-artifact
  defects (trailing marker, stray space) before verification, which is why
  the verified file differs from the raw delegation output by exactly those
  two lines.
- Commands run: `docker build -f apps/cli/Dockerfile -t dubbridge-cli:t4f-test .`;
  `docker exec local-postgres-1 psql -U dubbridge -d dubbridge -c "CREATE DATABASE t4f_empty_test;"`;
  `docker run --rm --network local_default -e DUBBRIDGE_ENV=local -e DUBBRIDGE_DATABASE_URL=... dubbridge-cli:t4f-test` (HP-1, then rerun);
  `docker run --rm -e DUBBRIDGE_ENV=local -e DUBBRIDGE_DATABASE_URL=postgres://...@nonexistent-host-unreachable:5432/... dubbridge-cli:t4f-test` (EC-1);
  `docker exec local-postgres-1 psql -U dubbridge -d dubbridge -c "DROP DATABASE t4f_empty_test;"`.

### S-230-T4g: Migration image contract tests

**Type:** development/test
**Effort:** S — RRI 20 Low (recomputed at task-presentation time via
`scripts/rri.py`: `--touches scripts/test-production-images.sh --cc 5 --D 1
--K 1 --P 0 --T 2 --A 0 --X 1` -> base 13 -> band Low, same band as the
ledger's provisional RRI 20, no gate change)
**Depends on:** S-230-T4f
**Status:** [x] Done — 2026-08-20
**Writable path:** `scripts/test-production-images.sh`

Add one-shot migration cases that codify T4f's manual evidence as a repeatable
harness case, run against the image built in T4f. **HP-1:** contract mode
requires `dubbridge-cli` as the entry point and the compiled migration set in
the T4f image, and the runtime case reproduces T4f's empty-DB zero-exit
result. **EC-1:** runtime mode requires a non-zero exit against an unreachable
database (matching T4f's EC-1 transcript) and must not report migration
success. Evidence: RRI artifact; Muse phase reviews; harness tests executed
against the real T4f image; HP/EC certification; owner verification. Status
artifact: this ledger. Stop before modifying `apps/cli/Dockerfile` or any
other image.

**Implementation routing:** local delegation to `qwen3.8:27b-mlx` (Low-band
developer) via `scripts/delegate-low-rri.py --mode before-after`, in two
separate single-anchor edits (per-edit anchors kept small, matching
`feedback_before_after_needs_small_anchors`): edit 1 appended `migration` to
`CASE_LIST`; edit 2 inserted `contract_migration()`/`run_migration()`
immediately before `# Main execution`, mirroring `contract_api`/`run_api`'s
structure with no HTTP health polling (the CLI image is a one-shot job, not a
server). Each edit's delegation packet went through its own phase-1 review
(see below), consistent with `feedback_phase1_review_per_delegation_packet` —
a materially revised packet is a new phase-1 event, not a carry-forward of an
earlier PASS.

**Post-delegation defect (mechanical, orchestrator-fixed):** independent
verification against the real T4f image and real local Postgres
(`local-postgres-1`) surfaced a `set -e` bug in the delegated `run_migration`:
`ec1_output=$(docker run ...)` followed by `ec1_exit=$?` on the next line
aborted the script before `ec1_exit=$?` could execute, because EC-1's `docker
run` is *expected* to exit non-zero (unreachable DB) and `set -e` treats a
non-zero command-substitution result as script-fatal unless guarded. `hp1_exit`
did not need the same fix since HP-1's `docker run` is expected to succeed.
Fixed by initializing `ec1_exit=0` and capturing via `|| ec1_exit=$?` on the
`docker run` line itself — a one-line mechanical fix to code the local model
authored, not new logic, matching the documented tooling-failure exception
pattern used in T4a/T4b/T4f. Confirmed by direct real-infrastructure
verification (below) and independently re-checked by Muse Glimmer phase 2.

A second, purely cosmetic artifact — 3 pre-existing trailing-whitespace blank
lines inside `run_api()` (unrelated to this task, already present at `HEAD`)
were incidentally stripped by the before-after wrapper's diff/apply step on
both edits — was restored to its original byte content each time, keeping the
final diff scoped to exactly the `CASE_LIST` line and the two new functions.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Ollama restart + local-stack precheck: performed once for this task ID
  (`S-230-T4g`) before the first local-model call — old PID 49570 killed, new
  PID 52379 confirmed listening on `11434`; warm-up probe returned
  `done_reason: stop` with non-empty content at production
  `num_ctx=65536`/`num_predict=4096`.
- Phase 1 (task-analysis, pre-delegation), edit 1 (CASE_LIST): `PASS` on the
  first pass — bundled into the same review as edit 2's first attempt.
- Phase 1 (task-analysis, pre-delegation), edit 2 (`contract_migration`/
  `run_migration`), attempt 1: `BLOCKED` — 5 findings (2 high: DB-provisioning
  under-specified, "compiled migration set" check method undefined; 2 medium:
  ambiguous success-log assertion, unspecified unreachable-host value; 1 low:
  CLI image-tag override usage pattern not exemplified). Packet revised with
  the exact DB name/commands from T4f's verified transcript, the
  `sqlx::migrate!` proxy check, and the literal unreachable hostname.
- Phase 1, edit 2, attempt 2 (revised packet): `BLOCKED` under a reduced-context
  recovery profile (see below) — 3 findings (2 medium: cleanup-comment
  instruction self-contradictory, ENTRYPOINT grep escaping not shown verbatim;
  1 low: insertion anchor relied on context not present in the reduced
  excerpt). Packet revised again with a verbatim fixed `cleanup_migration()`
  body/comment, the exact `contract_api`-derived grep line, and an explicit
  byte-for-byte insertion anchor.
- Phase 1, edit 2, attempt 3 (revised packet): `PASS`, 0 findings.
- Phase 2 (code-solution, post-implementation, against the final diff plus
  independently-executed HP-1/EC-1/contract-mode transcripts, including the
  `set -e` fix): `PASS`, 4 info-level findings confirming HP-1/EC-1 coverage,
  the `set -e` fix's correctness, and pattern consistency with
  `contract_api`/`run_api`/`contract_gateway`/`run_gateway`.
- Passes run / usable: `1/1` per phase (single-pass mode); 3 phase-1 attempts
  total for edit 2 due to two rounds of packet revision, each with its own
  independent review per `feedback_phase1_review_per_delegation_packet`.
- Aggregate status: `PASS`
- Local resource-recovery protocol: triggered once, between phase-1 attempt 1
  and attempt 2 of edit 2 — the first re-review request returned `done_reason:
  stop` with **empty content** (0 bytes) despite `think:false`, matching the
  known `muse-glimmer` real-packet failure mode
  (`feedback_muse_glimmer_think_flag_defect`). Followed the documented
  protocol: unloaded the model, confirmed low free memory via
  `memory_pressure` (≈73MB free), ran a bounded `num_ctx<=16384`/
  `num_predict<=1024` probe (succeeded), then rebuilt the actual review packet
  as a reduced excerpt (file header + `run_api()` reference + insertion point
  only, instead of the full 378-line file) under the same reduced profile for
  all three phase-1 attempts and the phase-2 review. This is a capacity
  symptom, not a stall — the model was actively loaded and responding to
  `/api/ps` keepalive throughout.
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at every phase once the reduced-context profile was
  applied; no attempt exhausted the retry-then-fallback chain.
- disposition_divergence: `none`
- Primary-agent disposition: all `BLOCKED` findings across both phase-1
  revision rounds were accepted and resolved by revising the delegation
  packet (not overridden); phase-2 info findings required no further action.
- REVIEW-OVERRIDE: not used — every phase has an artifact-backed verdict.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | contract mode passes for the real T4f Dockerfile/CLI source | `bash scripts/test-production-images.sh contract migration` against the real `apps/cli/Dockerfile` (line 33 `ENTRYPOINT ["/app/dubbridge-cli"]`) and `apps/cli/src/main.rs` (line 11 `sqlx::migrate!(...)`) → `Contract check for migration case` / `Contract check passed for migration`, exit 0 | passed |
| HP-1 | Happy path | run mode reproduces T4f's empty-DB zero-exit result | `bash scripts/test-production-images.sh run migration` — rebuilt the real `dubbridge-cli:t4f-test` image (`docker build -f apps/cli/Dockerfile -t dubbridge-cli:t4f-test .`), ran it via the harness against a fresh throwaway database (`t4g_contract_test`) created on the real `local-postgres-1` dependency container → internal trace confirms `docker run` exit 0 with combined output containing `dubbridge-cli: migrations applied successfully`; harness prints `Run check passed for migration`, exit 0 | passed |
| EC-1 | Edge case | run mode fails closed against an unreachable database | Same `run migration` invocation's internal EC-1 stage — `docker run` against `DUBBRIDGE_DATABASE_URL` pointing at `nonexistent-host-unreachable` → non-zero exit, combined output containing `failed to lookup address information: Name or service not known`, no `migrations applied successfully` line anywhere in the captured output | passed |

Regression check: `bash scripts/test-production-images.sh contract self-check`,
`contract api`, and `contract gateway` all still pass unchanged after both
edits. `bash -n scripts/test-production-images.sh` confirms no syntax errors
introduced. Throwaway test database dropped by `run_migration`'s own
`cleanup_migration` RETURN trap (verified via `psql -c '\l'` showing no `t4g_*`
database remaining); the `dubbridge-cli:t4f-test` test image removed after
verification (`docker rmi`), matching T4f's own test-image lifecycle.

Reviewability budget: not evaluated — 68-line addition to an existing 378-line
file, trivially within any derived Low-band review budget; no margin
question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-20`
- Statement: I verified HP-1 and EC-1 by rebuilding the real T4f image from
  `apps/cli/Dockerfile` and running the new harness case against real Docker
  and a real, independently-created throwaway Postgres database on the local
  Compose infrastructure, confirming exit codes and log output directly via
  both the harness's own output and a full `bash -x` trace rather than
  accepting the delegated code at face value. I independently diagnosed and
  fixed the one `set -e`/command-substitution defect the delegated code
  contained (a one-line mechanical fix, not new logic), reverted an unrelated
  cosmetic whitespace side-effect from the before-after wrapper on both edits,
  and re-ran phase-2 review against the corrected final diff before closing.
- Commands run: `docker build -f apps/cli/Dockerfile -t dubbridge-cli:t4f-test .`;
  `bash scripts/test-production-images.sh contract migration`;
  `bash -x scripts/test-production-images.sh run migration` (initial trace,
  isolated the `set -e` defect); `docker exec local-postgres-1 psql -U
  dubbridge -d dubbridge -c "DROP DATABASE IF EXISTS t4g_contract_test;"`
  (cleanup between attempts); `bash scripts/test-production-images.sh run
  migration` (final clean run, exit 0); `bash scripts/test-production-images.sh
  contract self-check`, `contract api`, `contract gateway` (regression check);
  `bash -n scripts/test-production-images.sh` (syntax check);
  `docker exec local-postgres-1 psql -U dubbridge -d dubbridge -c "\l"`
  (confirmed no residual `t4g_*` database); `docker rmi dubbridge-cli:t4f-test`.

### S-230-T4h: Exact ASR dependency lock

**Type:** development/config
**Effort:** S — RRI 7 Low (recomputed at task-presentation time via
`scripts/rri.py`; corrects the ledger's provisional RRI 14 estimate, same
band, no gate change)
**Depends on:** S-230-T4a
**Status:** [x] Done — 2026-08-20
**Writable path:** `workers/asr-worker-py/requirements.txt`

Replace the floating constraint with the exact existing lower-bound release
`faster-whisper==1.1.0`; do not add packages. **HP-1:** a clean Python 3.12
environment installs the requirement and `pip freeze --all` reports the exact
version. **EC-1:** the production dependency input contains no `>=`, wildcard,
or unpinned direct dependency. Evidence: RRI artifact; Muse phase reviews;
install/freeze transcript; HP/EC certification; owner verification. Status
artifact: this ledger. Stop before editing either Dockerfile.

**RRI:** 7 (Low). `python3 scripts/rri.py --touches
workers/asr-worker-py/requirements.txt --cc 1 --D 1 --K 1 --P 0 --T 0 --A 0
--X 1` — single-line constant edit in a 1-line config file, no anchor-rubric
match, no penalties. Base value 7 -> band Low -> local delegation route.

**Implementation routing:** direct mechanical edit by the primary agent
(orchestrator), not routed through `scripts/delegate-low-rri.py`. The change
is a single-token constraint edit (`>=1.1.0` -> `==1.1.0`) in a one-line file
with no ambiguity, no branching logic, and no candidate diff for a model to
construct — there is nothing a delegation packet would add beyond restating
the one-line instruction. Independent Low-band review (Muse Glimmer) still
ran against the real diff and real verification evidence per the mandatory
Step 1-A gate; this is not a bypass of review, only of the delegation
mechanism for a change with no authoring decision to delegate.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Ollama restart + local-stack precheck: performed once for this task ID
  (`S-230-T4h`) before the first local-model call — old PID 52379 killed, new
  PID 4883 confirmed listening on `11434`; warm-up probe returned
  `done_reason: stop` with non-empty content at production
  `num_ctx=65536`/`num_predict=4096`.
- Combined phase 1/phase 2 review (task-analysis and code-solution collapsed
  into one pass, consistent with the trivial single-line nature of the
  change — packet included the real diff, acceptance criteria, and the
  independently-executed HP-1/EC-1 verification transcripts below):
  `PASS`, 0 blocking findings — 3 informational confirmations only
  (constraint change noted, `pip freeze` result confirmed, no floating/
  wildcard constraints confirmed) — `t4h_phase1and2_result.json`
  (scratchpad).
- Passes run / usable: `1/1` (single-pass mode).
- Aggregate status: `PASS`
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced a usable verdict.
- disposition_divergence: `none`
- Primary-agent disposition: no blocking findings to disposition; the 3
  informational items match the independently-verified evidence exactly.
- REVIEW-OVERRIDE: not used — artifact-backed verdict exists.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | clean Python 3.12 env installs and reports exact version | `python3.12 -m venv <scratch venv>` (Python 3.12.13) then `pip install -r workers/asr-worker-py/requirements.txt` then `pip freeze --all \| grep faster-whisper` → `faster-whisper==1.1.0` exactly | passed |
| EC-1 | Edge case | no floating/wildcard/unpinned constraint in the file | `grep -nE '>=\|<=\|~=\|\*' workers/asr-worker-py/requirements.txt` → no matches; file content is exactly `faster-whisper==1.1.0` | passed |

Reviewability budget: not evaluated — 1-line single-file constant edit,
trivially within any derived Low-band review budget; no margin question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-20`
- Statement: I verified HP-1 and EC-1 directly by creating a real, clean
  Python 3.12.13 virtual environment, installing the requirement from the
  actual repository file, and confirming the exact resolved version via
  `pip freeze --all`, then independently grepped the file for any floating
  or wildcard constraint syntax and confirmed none remain.
- Commands run: `python3.12 -m venv <scratch venv>`;
  `pip install -r workers/asr-worker-py/requirements.txt`;
  `pip freeze --all | grep -i faster-whisper`;
  `grep -nE '>=|<=|~=|\*' workers/asr-worker-py/requirements.txt`;
  scratch venv removed after verification.

### S-230-T4i: Worker native-runtime image

**Type:** development/config
**Effort:** S — RRI 13 Low (recomputed at task-presentation time via
`scripts/rri.py`; corrects the ledger's provisional RRI 21 estimate, same
band, no gate change)
**Depends on:** S-230-T4a, S-230-T4h
**Status:** [x] Done — 2026-08-20
**Writable path:** `apps/worker-runner/Dockerfile`

Create the digest-pinned multi-stage worker image with the Rust binary
(`dubbridge-worker-runner`), `ffmpeg`, and `ffprobe`, but no ASR Python layer
yet. Record resolved OS package versions. **HP-1:** the built image contains
all three executables (`dubbridge-worker-runner`, `ffmpeg`, `ffprobe`) and each
is directly invocable. **EC-1:** the runtime contains no Rust build toolchain,
and removing a native tool from the build (or a manual removal transcript
against the built image) causes an invocation failure. Evidence: RRI artifact;
Muse phase reviews; build transcript; base digest, image size and OS
inventory; executable-presence transcript; HP/EC certification; owner
verification. Status artifact: this ledger. Stop before bundling Python/ASR.

**RRI:** 13 (Low). `python3 scripts/rri.py --touches
apps/worker-runner/Dockerfile --cc 2 --D 1 --K 2 --P 0 --T 1 --A 0 --X 2` —
single-file Dockerfile edit, no anchor-rubric match, no penalties. Base value
13 -> band Low -> local delegation route (ledger's original planning estimate
was 21; re-run at implementation time scored 13, same band, no route change).

**Implementation routing:** a Dockerfile matching this task's scope already
existed untracked in the working tree (owner-authored draft, digest-pinned to
the same real OCI digests resolved in T4c for `apps/api/Dockerfile`, correctly
covered by the repo-root `.dockerignore` added in T4c). The orchestrator
adopted it as the candidate rather than re-delegating from scratch, but it
still went through its own mandatory phase-1 review before being treated as
accepted (per the per-packet review rule — an existing draft is not exempt
from review just because a model didn't just author it).

**Ollama restart + local-stack precheck:** performed once for this task ID
(`S-230-T4i`) — old PID `12245` killed (interactive `quit` was cancelled by
a system dialog, so the process was killed directly), new PID `41354`
confirmed listening on `11434`. Warm-up probes at production
`num_ctx=65536`/`num_predict=4096`, `think=false`: `muse-glimmer:30b-q4_K_M`
→ `done_reason: stop`, 15-char content; `gemma4:26b-a4b-it-qat` →
`done_reason: stop`, 15-char content. Both usable before the first real
review call.

**Phase-1 finding and resolution:** the first phase-1 pass (Muse Glimmer,
against the candidate Dockerfile text only, pre-build) returned `BLOCKED`
with one blocking finding: the task's "Record resolved OS package versions"
requirement had no evidence anywhere — the Dockerfile installs `ffmpeg` via
apt but nothing captured which versions apt would actually resolve. Two
informational findings also noted: (1) `ffprobe` is not installed explicitly,
relying on the Debian `ffmpeg` package bundling it — a documented, verified
assumption (Debian's `ffmpeg` package has shipped `ffprobe` in the same
package for years), not a defect; (2) `COPY . /usr/src/app` in the builder
stage still copies Python/ASR sources into the build context, mitigated by
the existing repo-root `.dockerignore`.

The orchestrator did not argue the blocking finding away — it was correct:
nothing had actually built the image or captured real package versions yet.
Docker was available in this environment (via Colima), unlike T4b's
delegation environment which had none, so the orchestrator built the image
for real instead of deferring runtime evidence to T4j (the contract-test
child), which is what T4b/T4c did under a no-Docker constraint that does not
apply here.

**Build execution note:** the first `docker build` invocation was
interrupted (killed) by the orchestrator after mistaking slow-but-genuine
rustup/cargo network activity for a stall (verified after the fact via
`docker inspect` RestartCount and `colima ssh -- top`/`docker stats` showing
real CPU/network I/O, not a hang). The kill produced a misleading "exit 0"
because it terminated the process cleanly via the `tail` pipe, not because
the build actually completed — no image existed after that attempt. The
build was re-run to completion without interruption (logged directly to a
file rather than through a `tail` pipe this time), producing the image below.
This is recorded as a root-caused operational mistake corrected within the
same task, not a build defect.

Full command:
```
docker build -t dubbridge-worker-runner-t4i:test -f apps/worker-runner/Dockerfile .
```
Result: `Successfully built e8069bd44166` / `Successfully tagged
dubbridge-worker-runner-t4i:test`, `EXIT_CODE=0`.

**Image evidence:**
- Digest: `dubbridge-worker-runner-t4i@sha256:e8069bd4416673eae615ee9bf233299e26b2b644371c89563a22d4fc65fce9de`
- Content size: 194MB; total disk usage: 729MB
- Base digests (unchanged from the reviewed candidate):
  `rust:1-bookworm@sha256:6e957ef098dcc77d33e310261e4ed5843bb108d5c3b5dc2b476cbc8b6caf53fa`
  (builder), `debian:bookworm-slim@sha256:817e6cf99d6fc127ff4ffe8580049b60deba0adfbbb2bd65ddc3ef8fbb7aade0`
  (runtime)
- **Resolved OS package versions** (`dpkg -l` inside the built container,
  resolving the phase-1 blocking finding):
  - `ca-certificates` `20250419~deb12u1` (all)
  - `ffmpeg` `7:5.1.9-0+deb12u1` (arm64)
  - `libssl3` `3.0.20-1~deb12u2` (arm64)

**HP-1 executed transcript** (`docker run` against the built image, not
simulated):
- `dubbridge-worker-runner`: present at `/app/dubbridge-worker-runner`,
  `-rwxr-xr-x`, 17,631,264 bytes. Invoking it with no environment configured
  produces `Error: DUBBRIDGE_ENV is not set; set it to one of: local,
  staging, production` — the application's own ADR-026 fail-closed config
  validation firing correctly (not a missing-binary or crash error).
- `ffmpeg -version`: `ffmpeg version 5.1.9-0+deb12u1 ... built with gcc 12
  (Debian 12.2.0-14+deb12u1)` — succeeded.
- `ffprobe -version`: `ffprobe version 5.1.9-0+deb12u1 ...` — succeeded,
  confirming the bundled-with-`ffmpeg` assumption flagged in phase-1 was
  correct.

**EC-1 executed transcript** (`docker run` against the built image):
- No Rust build toolchain: `which cargo rustc` inside the running container
  produced no output, exit code 1.
- Manual removal + invocation failure: `rm /usr/bin/ffmpeg` followed by
  `ffmpeg -version` in the same container produced `/bin/sh: 3: ffmpeg: not
  found`, exit code 127 — matching EC-1's acceptance wording exactly.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Ollama restart + local-stack precheck: performed once for this task ID
  (see above) — new PID `41354`; warm-up `done_reason: stop` for both
  `muse-glimmer:30b-q4_K_M` and `gemma4:26b-a4b-it-qat`.
- Phase 1 (task-analysis, pre-build, against the candidate Dockerfile text):
  `BLOCKED`, 1 blocking finding (missing resolved-OS-package-version
  evidence) + 2 informational — `t4i_phase1_result.json` (scratchpad).
- Phase 2 (code-solution, post-build, against the real executed HP-1/EC-1
  transcripts and resolved package versions): `PASS`, 0 blocking findings —
  3 informational confirmations (HP-1 satisfied, EC-1 satisfied, package
  versions now recorded) — `t4i_phase2_result.json` (scratchpad).
- Passes run / usable: `1/1` per phase (single-pass mode).
- Aggregate status: `PASS` (phase 1 blocking finding resolved by producing
  the missing evidence, not by re-arguing the finding)
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at both phases.
- disposition_divergence: `none`
- Primary-agent disposition: phase-1 blocking finding accepted as correct
  and resolved by actually building the image and capturing real `dpkg -l`
  output (Docker was available in this environment, unlike the T4b
  precedent); phase-1 informational findings both independently verified
  (ffprobe bundling confirmed via executed `-version` transcript;
  `.dockerignore` coverage confirmed pre-existing from T4c); phase-2 raised
  no findings to disposition beyond confirmatory notes.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | built image contains all three executables, each directly invocable | `docker run --rm --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'ls -la /app/dubbridge-worker-runner'` → `-rwxr-xr-x`, 17631264 bytes; `docker run --rm --entrypoint ffmpeg ... -version` → `ffmpeg version 5.1.9-0+deb12u1`; `docker run --rm --entrypoint ffprobe ... -version` → `ffprobe version 5.1.9-0+deb12u1` | passed |
| EC-1 | Edge case | no Rust toolchain in runtime; missing native tool causes invocation failure | `docker run --rm --entrypoint /bin/sh ... -c 'which cargo rustc'` → no output, exit 1; `docker run --rm --entrypoint /bin/sh ... -c 'rm /usr/bin/ffmpeg && ffmpeg -version'` → `/bin/sh: 3: ffmpeg: not found`, exit 127 | passed |

Reviewability budget: not evaluated — single-file, 34-line Dockerfile,
trivially within any derived Low-band review budget; no margin question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-20`
- Statement: I verified HP-1 and EC-1 directly against a real, freshly built
  Docker image (not a structural-only review) — confirmed the compiled
  Rust binary, `ffmpeg`, and `ffprobe` are all present and independently
  invocable inside the container, confirmed the binary's fail-closed
  behavior on missing `DUBBRIDGE_ENV` matches existing ADR-026 semantics
  rather than being a build defect, confirmed no Rust toolchain exists in
  the runtime stage, and confirmed that removing a native tool produces a
  real invocation failure. I confirmed the resolved OS package versions via
  `dpkg -l` inside the built container, resolving the phase-1 blocking
  finding with real evidence rather than argument. I confirmed no file
  outside the declared writable path was touched (`git status` showed only
  `apps/worker-runner/Dockerfile` as untracked/added). I also recorded, as
  an explicit operational note rather than omitting it, that my first build
  attempt was killed prematurely on a mistaken stall diagnosis and had to be
  re-run to completion.
- Commands run: `docker build -t dubbridge-worker-runner-t4i:test -f
  apps/worker-runner/Dockerfile .`; `docker images
  dubbridge-worker-runner-t4i:test`; `docker run --rm --entrypoint /bin/sh
  dubbridge-worker-runner-t4i:test -c 'ls -la /app/dubbridge-worker-runner'`;
  `docker run --rm --entrypoint /app/dubbridge-worker-runner
  dubbridge-worker-runner-t4i:test`; `docker run --rm --entrypoint ffmpeg
  dubbridge-worker-runner-t4i:test -version`; `docker run --rm --entrypoint
  ffprobe dubbridge-worker-runner-t4i:test -version`; `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'which cargo
  rustc'`; `docker run --rm --entrypoint /bin/sh
  dubbridge-worker-runner-t4i:test -c 'rm /usr/bin/ffmpeg && ffmpeg
  -version'`; `docker run --rm --entrypoint /bin/sh
  dubbridge-worker-runner-t4i:test -c 'dpkg -l ca-certificates libssl3
  ffmpeg'`; `git status apps/worker-runner/Dockerfile`.

### S-230-T4j: Worker native-runtime contract tests

**Type:** development/test
**Effort:** S — RRI 13 Low (recomputed at task-presentation time via
`scripts/rri.py`; corrects the ledger's provisional RRI 21 estimate, same
band, no gate change)
**Depends on:** S-230-T4i
**Status:** [x] Done — 2026-08-20
**Writable path:** `scripts/test-production-images.sh`

Add worker cases for the Rust executable and native media tools only,
codifying T4i's manual evidence as a repeatable harness case run against the
image built in T4i. **HP-1:** contract mode requires `dubbridge-worker-runner`,
`ffmpeg`, and `ffprobe` in the T4i runtime image, reproducing T4i's
executable-presence transcript. **EC-1:** runtime mode fails when either
native tool is missing or non-executable (matching T4i's EC-1 transcript).
Evidence: RRI artifact; Muse phase reviews; harness tests executed against the
real T4i image; HP/EC certification; owner verification. Status artifact: this
ledger. Stop before modifying `apps/worker-runner/Dockerfile` or adding
Python.

**RRI:** 13 (Low). `python3 scripts/rri.py --touches
scripts/test-production-images.sh --cc 3 --D 1 --K 2 --P 0 --T 1 --A 0 --X 2`
— single-file edit adding two functions matching an existing pattern, no
anchor-rubric match, no penalties. Base value 13 -> band Low -> local
delegation route (ledger's original planning estimate was 21, same band, no
route change).

**Ollama restart + local-stack precheck:** performed once for this task ID
(`S-230-T4j`) — old PID `41354` (carried over from T4i's own restart earlier
in this session) killed directly (interactive `quit` was again cancelled by
a system dialog, matching the T4i precedent), new PID `19386` confirmed
listening on `11434`. Warm-up probes at production `num_ctx=65536`/
`num_predict=4096`, `think=false`: `muse-glimmer:30b-q4_K_M` ->
`done_reason: stop`, 5-char content; `gemma4:26b-a4b-it-qat` ->
`done_reason: stop`, 5-char content. `qwen3.8:27b-mlx` confirmed present via
`/api/tags`. Both review models usable before the first real review call.

**Implementation routing:** the `CASE_LIST="self-check api gateway migration"`
-> `... worker"` edit (a single-token addition to an already-verified
constant, no new logic) was applied directly by the orchestrator as the
narrow mechanical-edit exception, not delegated. The substantive logic — the
`contract_worker()`/`run_worker()` function bodies — was delegated to Qwen
Developer (`qwen3.8:27b-mlx`) via `scripts/delegate-low-rri.py --mode
before-after`, anchored on the small `# Main execution` / `main() {` header
(2-line BEFORE block, well under the 40-line safety cap), per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Handoff prompt format` and the
target-file-size/small-anchor discipline (443-line file, before-after mode
used with a small anchor rather than full-file rewrite).

**Phase-1 findings and resolution (3 packet revisions before PASS):**

- **v1** (`muse-glimmer:30b-q4_K_M`, against the packet + full file text):
  `BLOCKED`, 2 blocking findings — (1) `contract_worker()`'s description
  ambiguously allowed a `docker image inspect` call, which would violate the
  established convention (confirmed by re-reading `contract_migration`/
  `contract_api`/`contract_gateway`: all three are static-only, no Docker
  invocation); (2) EC-1's "single shell command" requirement was stated only
  in prose, risking the implementer splitting removal and re-invocation into
  two separate `docker run` calls, which would invalidate the test under
  `--rm` ephemeral-container semantics. Both findings verified as genuine
  specification gaps in the packet, not false positives — resolved by
  rewriting the packet with an explicit static-only contract_worker()
  constraint and explicit single-invocation EC-1 wording.
- **v2** (revised packet, own fresh phase-1 pass per the per-packet review
  rule): `BLOCKED` again, 2 blocking findings — (1) the static-only
  constraint was still described in prose without concrete grep patterns;
  (2) the EC-1 guarded-capture syntax needed under the file's active `set
  -euo pipefail` (line 2) was not made explicit, risking a bare failing
  `docker run` aborting the whole script instead of being handled by the
  function. Both verified genuine: the exact `ffmpeg` grep pattern was
  independently checked against the real Dockerfile
  (`grep -qE '^\s*ffmpeg\s*\\?\s*$' apps/worker-runner/Dockerfile` ->
  matches) before including it verbatim in the packet; the `set -e` guard
  requirement was cross-checked against `run_migration`'s existing
  `|| ec1_exit=$?` pattern in the same file. Resolved by adding literal,
  pre-verified bash code blocks for both functions' critical sections
  instead of prose description.
- **v3** (revised packet, own fresh phase-1 pass): `PASS`, 0 findings.

**Delegation attempt 1 of 2 — repair required:** Qwen Developer's first
before-after response reproduced `contract_worker()`/`run_worker()`
correctly per the reviewed v3 spec, but its AFTER block dropped the `main()
{` line from the 2-line BEFORE anchor (`# Main execution` / `main() {`),
which the wrapper's literal find-and-replace would have applied verbatim —
this would have orphaned the `main` function body outside any function
definition, a real correctness defect caught before application (the
`--apply` flag was withheld on this attempt; nothing was written to disk).
Per the Moderate-band cross-delegate discipline (repair goes back to the
local pipeline, not fixed directly by the orchestrator), a repair packet was
built explicitly instructing the implementer to preserve both anchor lines,
and — per the per-packet review rule — this repair packet received its own
fresh phase-1 pass (`PASS`, 1 informational confirmation) before
re-delegation.

**Delegation attempt 2 of 2 (repair) — success:** Qwen Developer's repaired
response preserved `# Main execution` / `main() {` correctly, kept both
function bodies structurally identical to the already-reviewed spec, and was
applied with `--apply`. The diff also showed three unrelated
trailing-whitespace normalizations inside the pre-existing `run_api()`
function (lines 115, 138, 189) — investigated and confirmed as a deliberate,
documented wrapper behavior (`_strip_trailing()` in
`scripts/delegate-low-rri.py::apply_before_after`, applied to the *entire*
original file content to avoid whitespace-sensitive BEFORE-block matching
failures), not a model-authored scope leak or semantic change (bash
disregards trailing whitespace on blank/comment lines); accepted without
further repair.

**Real execution evidence (not simulated):**

- `bash scripts/test-production-images.sh contract worker` -> exit 0,
  "Contract check passed for worker" (static-only, no Docker daemon call).
- Rebuilt `dubbridge-worker-runner-t4i:test` for real via `docker build -t
  dubbridge-worker-runner-t4i:test -f apps/worker-runner/Dockerfile .`
  (T4i's original `:test` tag was not persisted across the session) ->
  `Successfully built 8e37af071eaa` / `Successfully tagged
  dubbridge-worker-runner-t4i:test`, exit 0. Cached base layers from T4i's
  earlier build in this session kept the rebuild fast.
- `bash scripts/test-production-images.sh run worker` -> exit 0, "Run check
  passed for worker" — HP-1 and EC-1 both executed for real against the
  rebuilt image inside the harness.
- **Independent reproduction outside the harness** (HP-1): `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'ls -la
  /app/dubbridge-worker-runner'` -> `-rwxr-xr-x 1 root root 17631264 ...`;
  `docker run --rm --entrypoint ffmpeg ... -version` -> `ffmpeg version
  5.1.9-0+deb12u1`; `docker run --rm --entrypoint ffprobe ... -version` ->
  `ffprobe version 5.1.9-0+deb12u1`.
- **Independent reproduction outside the harness** (EC-1): `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'rm
  /usr/bin/ffmpeg && ffmpeg -version'` -> `/bin/sh: 1: ffmpeg: not found`,
  exit 127.
- **Negative-path harness test** (confirms the guarded-capture pattern
  fails closed under a real failure instead of a `set -e` abort):
  `DUBBRIDGE_WORKER_IMAGE_TAG=nonexistent-image-xyz:test bash
  scripts/test-production-images.sh run worker` -> exit 1, clear `ERROR:
  HP-1 FAILED: ...` plus surfaced Docker `pull access denied` message.
- `bash scripts/test-production-images.sh contract self-check` (unaffected
  case, regression check) -> unchanged output, exit 0.
- `bash -n scripts/test-production-images.sh` -> syntax OK.
- `git status --short` -> only `scripts/test-production-images.sh` modified.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band phase-1/phase-2 primary)
- Ollama restart + local-stack precheck: performed once for this task ID
  (see above) — new PID `19386`; warm-up `done_reason: stop` for both
  `muse-glimmer:30b-q4_K_M` and `gemma4:26b-a4b-it-qat`.
- Phase 1 (task-analysis, pre-delegation, against the packet + full file):
  `BLOCKED` (v1, 2 blocking) -> `BLOCKED` (v2, 2 blocking) -> `PASS` (v3, 0
  findings) — three fresh phase-1 passes, one per materially revised
  packet, per the per-packet review rule. A fourth fresh pass on the repair
  packet also returned `PASS` (1 informational) before the second
  delegation attempt.
- Phase 2 (code-solution, post-implementation, against the real diff + the
  independently-verified execution evidence above): `PASS`, 0 blocking
  findings — 5 informational confirmations (static-only contract check
  confirmed; guarded exit-code capture confirmed; single combined-invocation
  EC-1 confirmed; HP-1 checks confirmed; no unintended side effects/scope
  leaks observed).
- Passes run / usable: `1/1` per phase-2 pass (single-pass mode); 4 total
  phase-1 passes across packet revisions, all usable.
- Aggregate status: `PASS` (both phase-1 blocking-finding rounds resolved by
  producing corrected, pre-verified packet content, not by re-arguing the
  findings; the delegation-attempt-1 defect was resolved via cross-delegated
  repair, not a direct orchestrator fix)
- Isolated adjudicator (D14): not triggered — Muse Glimmer was available and
  produced usable verdicts at every phase-1 and phase-2 call.
- disposition_divergence: `none`
- Primary-agent disposition: all four phase-1 blocking findings accepted as
  correct and resolved with verified evidence (grep pattern independently
  matched against the real Dockerfile; `set -e` guard pattern cross-checked
  against `run_migration`'s existing precedent); the delegation-attempt-1
  dropped-`main()`-line defect was independently caught by the orchestrator
  before `--apply`, and repaired via a fresh cross-delegated repair packet
  rather than a direct edit; the three trailing-whitespace diff lines in
  `run_api()` were investigated and confirmed as deliberate wrapper
  behavior, not a scope violation.
- REVIEW-OVERRIDE: not used — both phases have artifact-backed verdicts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | contract mode requires `dubbridge-worker-runner`, `ffmpeg`, `ffprobe` present in the T4i runtime image | `bash scripts/test-production-images.sh contract worker` -> exit 0; `bash scripts/test-production-images.sh run worker` -> exit 0 (HP-1 checks executed inside `run_worker()`); independently reproduced outside the harness via direct `docker run --entrypoint {sh,ffmpeg,ffprobe}` invocations, all succeeding with expected output | passed |
| EC-1 | Edge case | runtime mode fails when a native tool is missing or non-executable | `run_worker()`'s EC-1 block (`rm /usr/bin/ffmpeg && ffmpeg -version` in one combined `docker run`) executed inside `bash scripts/test-production-images.sh run worker` -> exit 0 overall because the harness correctly treats the expected non-zero EC-1 sub-exit as a pass condition; independently reproduced outside the harness: same command directly -> `ffmpeg: not found`, exit 127; negative-path harness test with a nonexistent image tag -> exit 1 with clear `ERROR: HP-1 FAILED: ...` message, confirming the guarded-capture pattern correctly fails closed under `set -euo pipefail` rather than aborting silently | passed |

Reviewability budget: not evaluated — single-file, ~70-line net addition to
an existing 443-line script, trivially within any derived Low-band review
budget; no margin question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies)
- Date: `2026-08-20`
- Statement: I verified HP-1 and EC-1 directly against a real, freshly
  rebuilt Docker image (not a structural-only review) — confirmed
  `contract worker` and `run worker` both pass end-to-end through the real
  harness script, independently reproduced every HP-1/EC-1 check outside
  the harness with matching results, and independently confirmed the
  negative path (missing image) fails closed with a clear error instead of
  a silent `set -e` abort. I confirmed `contract_worker()` stayed
  static-only (no Docker daemon calls), matching the existing
  `contract_api`/`contract_gateway`/`contract_migration` convention, and
  that EC-1 is a single combined `docker run` invocation as required (not
  split across two, which would have silently invalidated the test under
  `--rm` ephemeral-container semantics). I confirmed no file outside the
  declared writable path was touched (`git status --short` showed only
  `scripts/test-production-images.sh` modified) and that the script remains
  syntactically valid (`bash -n`) and the pre-existing `self-check` case is
  unaffected. I also recorded, as an explicit note rather than omitting it,
  that the first delegation attempt produced a real correctness defect
  (dropped `main() {` line) caught before application, resolved through a
  cross-delegated repair packet rather than a direct fix, consistent with
  the Moderate-band cross-delegate-on-failure discipline applied here at
  Low band as well.
- Commands run: `python3 scripts/rri.py --touches
  scripts/test-production-images.sh --cc 3 --D 1 --K 2 --P 0 --T 1 --A 0
  --X 2`; `python3 scripts/delegate-low-rri.py <packet> --mode before-after
  --target-path scripts/test-production-images.sh --before-file <before>
  --allow-path scripts/test-production-images.sh --task-id S-230-T4j
  --attempt 1 --rri 13 --no-think --temperature 0.1 --out <result>` (attempt
  1, not applied); same command with `--attempt 2 --apply` (repair,
  applied); `bash -n scripts/test-production-images.sh`; `bash
  scripts/test-production-images.sh contract worker`; `docker build -t
  dubbridge-worker-runner-t4i:test -f apps/worker-runner/Dockerfile .`;
  `bash scripts/test-production-images.sh run worker`; `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'ls -la
  /app/dubbridge-worker-runner'`; `docker run --rm --entrypoint ffmpeg
  dubbridge-worker-runner-t4i:test -version`; `docker run --rm --entrypoint
  ffprobe dubbridge-worker-runner-t4i:test -version`; `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4i:test -c 'rm
  /usr/bin/ffmpeg && ffmpeg -version'`; `DUBBRIDGE_WORKER_IMAGE_TAG=nonexistent-image-xyz:test
  bash scripts/test-production-images.sh run worker`; `bash
  scripts/test-production-images.sh contract self-check`; `git status
  --short scripts/test-production-images.sh`.

### S-230-T4k: Worker ASR-bundle image

**Type:** development/config
**Effort:** S — RRI 13 Low (recomputed at task-presentation time via
`scripts/rri.py`; corrects the ledger's provisional RRI 21 estimate, same
band, no gate change)
**Depends on:** S-230-T4j
**Status:** [x] Done — 2026-08-21
**Writable path:** `apps/worker-runner/Dockerfile` (planned); widened during
execution to also include `workers/asr-worker-py/requirements.txt` — see
`### Scope-widening note` below.

Extend the worker image with Python, the ASR worker, and its exact
dependencies; set the two `DUBBRIDGE_ASR_WORKER_*` paths to locations that
exist inside the image and set `ASR_MODEL_SIZE=small`. **HP-1:** the bundled
worker accepts a valid protocol request end to end (Python 3,
`workers/asr-worker-py/main.py`, exact `faster-whisper==1.1.0`,
`DUBBRIDGE_ASR_WORKER_PATH`, `DUBBRIDGE_ASR_WORKER_PYTHON`, default
`ASR_MODEL_SIZE=small` all present and resolving inside the image). **EC-1:** a
deliberately invalid selected model produces a visible protocol/job failure
and never falls back to `large-v3` or a false-ready result. Evidence: RRI
artifact; Muse phase reviews; build and protocol transcripts;
`pip freeze --all`; image size; HP/EC certification; owner verification.
Status artifact: this ledger. Stop before translation bundling or
cross-service smoke.

**RRI:** 13 (Low). `python3 scripts/rri.py --touches
apps/worker-runner/Dockerfile --cc 2 --D 1 --K 2 --P 0 --T 1 --A 0 --X 2` —
single-file additive Dockerfile edit matching an existing bundling pattern
(the standalone `workers/asr-worker-py/Dockerfile`), no anchor-rubric match,
no penalties. Base value 13 -> band Low -> local delegation route (ledger's
original planning estimate was 21, same band, no route change).

**Ollama restart + local-stack precheck:** performed once for this task ID
(`S-230-T4k`) — prior server PID `76448` killed; the Ollama.app supervisor
auto-relaunched a new server, new PID `76584` confirmed listening on
`11434`. Warm-up probes at production `num_ctx=65536`/`num_predict=4096`,
`think=false`: `muse-glimmer:30b-q4_K_M` -> `done_reason: stop`, 18-char
content (`{"status":"ready"}`); `gemma4:26b-a4b-it-qat` -> `done_reason:
stop`, 18-char content (`{"status":"ready"}`). Both models in the RRI 0-25
chain passed cleanly; no resource-recovery protocol needed.

**Task-analysis review (Phase 1):** the delegation packet was independently
reviewed by Muse Glimmer before being sent to the local implementer, on both
the original packet and the attempt-2 repair packet (see below) after it was
materially revised — each got its own PASS.

Task-analysis review: muse-glimmer (inline, not persisted as a separate
artifact file — packet review transcript captured in this section) - PASS

**Implementation routing evidence:** local delegation via
`scripts/delegate-low-rri.py --mode before-after --target-path
apps/worker-runner/Dockerfile`, model `qwen3.8:27b-mlx` (default
`DUBBRIDGE_LOW_RRI_MODEL`).

- **Attempt 1** (packet: additive instructions, before-file = original
  34-line Dockerfile): phase-1 PASS from Muse Glimmer on the packet, but the
  model's returned patch failed independent review before application —
  fabricated sha256 digests on both `FROM` lines (values not matching the
  real base-image digests), a rewritten builder stage
  (`COPY Cargo.toml Cargo.lock ./` + `COPY src ./src` in place of
  `COPY . /usr/src/app` + `-p dubbridge-worker-runner`, which would break
  the Cargo workspace's path-dependency resolution across crates), and a
  dropped `AS runtime` stage name. **Not applied** — `git status --short`
  confirmed the working tree stayed clean after attempt 1.
- **Repair (attempt 2):** cross-delegate-on-failure discipline applied at
  Low band — an explicit repair packet enumerating each attempt-1 defect and
  restricting the model to two additive insertion points only (the apt
  package list, and after the existing `DUBBRIDGE_CONFIG_DIR` block) was
  independently phase-1-reviewed (Muse Glimmer PASS on the revised packet,
  a new verdict distinct from attempt 1's packet review) before
  re-delegation to the same local model. The returned patch preserved both
  base-image digests, the builder stage, and the `AS runtime` name
  byte-for-byte; applied via `--apply`. One cosmetic double-space before
  `&& rm -rf /var/lib/apt/lists/*` was fixed directly as a mechanical
  lint-driven refactor of already-verified logic (no behavior change),
  matching the narrow direct-edit exception in
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
  decomposition`. 2/2 repair budget used within limit (attempt 2 succeeded,
  no further escalation needed).

### Scope-widening note

Building the image and executing HP-1 surfaced a real, pre-existing defect
outside the originally planned `Writable path`: `faster-whisper==1.1.0`
imports `requests` directly inside its own code
(`faster_whisper/utils.py:8`), but `workers/asr-worker-py/requirements.txt`
only pinned `faster-whisper==1.1.0` and left `huggingface-hub` unpinned,
which today resolves to `1.28.0` — a version that no longer carries
`requests` as its own transitive dependency (it uses `httpx` internally).
The result was `ModuleNotFoundError: No module named 'requests'` on
`import faster_whisper`, reproduced identically in the pre-existing
standalone `workers/asr-worker-py/Dockerfile` build (unrelated to this
task's changes), confirming the defect predates T4k and is not something
this task's Dockerfile edit introduced. Per an explicit user decision
during this task, the `Writable path` was widened by one file
(`workers/asr-worker-py/requirements.txt`) to add `requests==2.34.2` —
verified via `pip3 install --dry-run` inside the built image to confirm a
clean, non-conflicting resolution — because HP-1 could not otherwise be
certified at all. Recorded here rather than silently expanding scope
undocumented.

### HP-1 / EC-1 evidence

- **HP-1:** `docker build -t dubbridge-worker-runner-t4k:test -f
  apps/worker-runner/Dockerfile .` succeeded; inside the built image,
  `python3` resolves to `/usr/bin/python3`, `/app/asr_worker/main.py` exists
  (3141 bytes), `DUBBRIDGE_ASR_WORKER_PATH=/app/asr_worker/main.py` resolves
  `is_file()` true, `DUBBRIDGE_ASR_WORKER_PYTHON=python3`, default
  `ASR_MODEL_SIZE=small` confirmed via `os.environ.get(...)` inside the
  image without any override, `import faster_whisper` and
  `from faster_whisper import WhisperModel` both succeed after the
  `requests` fix, `pip3 show faster-whisper` reports `Version: 1.1.0`
  exactly. Full protocol round-trip: piping a valid-shaped JSON request
  (`{"job_id":"...", "audio_uri":"file:///tmp/dummy.wav", ...}` against a
  synthetically generated silent WAV) through `python3 /app/asr_worker/
  main.py` returns a well-formed JSON response on stdout with exit 0 for
  the happy-path protocol shape (verified via the `audio_not_found` and
  successful-parse branches of `main.py`; full transcription is not
  exercised because it requires network access to download Whisper model
  weights, unavailable in the build sandbox — HP-1's scope per the task
  definition is protocol/dependency resolution end-to-end, not a live
  transcription run).
- **EC-1:** `docker run --rm -i -e
  ASR_MODEL_SIZE=not-a-real-invalid-model-xyz --entrypoint /bin/sh
  dubbridge-worker-runner-t4k:test` piping a request against a real
  (dummy, silent) WAV file through `python3 /app/asr_worker/main.py`
  returns `{"error_code": "transcription_failed", "message": "Invalid
  model size 'not-a-real-invalid-model-xyz', expected one of: tiny.en,
  tiny, base.en, base, small.en, small, medium.en, medium, large-v1,
  large-v2, large-v3, large, distil-large-v2, distil-medium.en,
  distil-small.en, distil-large-v3, large-v3-turbo, turbo"}` with exit
  code 1 — confirmed via direct exit-code capture into a file, not through
  a chained `echo` that could mask `$?`. Never silently substitutes
  `large-v3` and never returns a false-ready/exit-0 result.
- **Regression check:** `bash scripts/test-production-images.sh contract
  worker` (T4i/T4j's existing check) still passes unchanged;
  `/app/dubbridge-worker-runner` remains present and executable
  (`-rwxr-xr-x`, 17631264 bytes) inside the extended image.
- **Image size:** `dubbridge-worker-runner-t4k:test` = 1.28GB vs. T4i's
  `dubbridge-worker-runner-t4i:test` = 729MB — a +551MB delta from the
  Python runtime, pip, and the faster-whisper dependency tree (ctranslate2,
  onnxruntime, tokenizers, huggingface-hub, requests, etc.).
- **`pip freeze --all`:** `faster-whisper==1.1.0`, `requests==2.34.2`,
  `huggingface_hub==1.28.0`, `ctranslate2==4.8.1`, `onnxruntime==1.29.0`,
  `tokenizers==0.23.1`, `av==18.1.0`, `numpy==2.4.6`, plus their own
  transitive dependencies (full list captured in the build transcript for
  this task).

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (`DEFAULT_REVIEW_MODEL` for the RRI 0-25
  chain's primary reviewer)
- Command: `GEMMA_REVIEW_TASK_ID=S-230-T4k REVIEW_PATHS="apps/worker-runner/Dockerfile
  workers/asr-worker-py/requirements.txt" make qa-gemma-review`
- Passes run / usable: `3/3`
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Artifacts: `/tmp/dubbridge-gemma-review.json` (aggregate),
  `/tmp/dubbridge-gemma-review.pass{1,2,3}.json` (per-pass, not persisted
  in-repo); receipt at `docs/audit/gemma-evidence/S-230-T4k.json`
- Isolated adjudicator: `not triggered` — trigger: n/a (primary reviewer
  produced a usable 3/3 aggregate)
- D14 provider route: `n/a` — reason: D14 not triggered
- disposition_divergence: `null`
- Primary-agent disposition: no findings to disposition; diff independently
  re-verified against the applied Dockerfile and requirements.txt content
  before accepting the PASS verdict
- Review artifact: docs/audit/gemma-evidence/S-230-T4k.json

Code-solution review: muse-glimmer docs/audit/gemma-evidence/S-230-T4k.json - PASS

### Reflection log (applied to Muse Glimmer's Phase 2 output, per the RRI 0-25 route)

- **Draft verdict:** Muse Glimmer's 3/3-pass aggregate reported `status:
  pass`, zero findings across consensus/pass-specific/disagreement buckets.
- **Critique findings:** independently re-checked whether "no findings" was
  itself plausible given the diff's small size (10 lines across two files)
  and the fact that both HP-1 and EC-1 had already been verified against a
  real built image before the review ran — no logical-correctness gap
  found; the one real defect in this task (missing `requests` pin) was
  already caught and fixed by direct build/run verification *before* the
  Gemma pass, not something the reviewer needed to catch. Confirmed the
  reviewer's summary line ("Dockerfile adds Python runtime and ASR worker
  assets with explicit env vars; requirements pin added") accurately
  describes the actual diff content, not a generic restatement.
- **Revisions applied:** none needed — no issues found in Draft or Critique.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Python 3 + `main.py` + exact `faster-whisper==1.1.0` + both `DUBBRIDGE_ASR_WORKER_*` env vars + default `ASR_MODEL_SIZE=small` all resolve inside the built image, and a valid-shaped protocol request round-trips through `main.py` | Manual evidence-backed verification against the real built image (`docker build` + `docker run` transcripts in `### HP-1 / EC-1 evidence` above) — this task's own acceptance contract is image-build/runtime evidence, not a Rust/Python unit test; T4l (dependent task) codifies this as a repeatable `scripts/test-production-images.sh` harness case | passed |
| EC-1 | Edge case | an invalid `ASR_MODEL_SIZE` fails closed with a visible `transcription_failed` error and exit 1, never falling back to `large-v3` or a false-ready result | Manual evidence-backed verification against the real built image (`### HP-1 / EC-1 evidence` above); T4l codifies this as a repeatable harness case | passed |

Reviewability budget: not evaluated — 10-line net addition across two
files, trivially within any derived Low-band review budget; no margin
question.

### Owner final verification

- Owner: `matias` (primary agent, orchestrator of record for this Low-band
  task per the RRI 0-25 route — no separate human approval gate applies;
  the scope-widening to `workers/asr-worker-py/requirements.txt` was
  explicitly authorized by the human user mid-task via `AskUserQuestion`
  before being acted on)
- Date: `2026-08-21`
- Statement: I verified HP-1 and EC-1 directly against a real, freshly
  built Docker image (not a structural-only review) — confirmed Python 3,
  the ASR worker source, both `DUBBRIDGE_ASR_WORKER_*` env vars, and the
  default `ASR_MODEL_SIZE=small` all resolve inside the image; confirmed
  `import faster_whisper` succeeds at exactly version `1.1.0`; confirmed an
  invalid model size fails closed with exit 1 and a clear
  `transcription_failed` error rather than silently substituting
  `large-v3` or returning a false-ready result. I independently discovered,
  diagnosed, and fixed a real pre-existing dependency gap (`requests`
  missing from `workers/asr-worker-py/requirements.txt`) that blocked HP-1
  outright, confirmed it was not introduced by this task by reproducing the
  identical failure against the unmodified standalone
  `workers/asr-worker-py/Dockerfile`, and got explicit human authorization
  before widening the writable-path scope to fix it. I confirmed the
  attempt-1 local-delegation output was correctly rejected before
  application (fabricated image digests, a rewritten builder stage that
  would have broken the Cargo workspace build, a dropped stage name) and
  that the repair (attempt 2) preserved every byte of content outside the
  two authorized additive insertion points. I confirmed
  `bash scripts/test-production-images.sh contract worker` (T4i/T4j's
  existing check) still passes unchanged and the Rust binary remains
  present and executable in the extended image. I confirmed no file outside
  the (widened, explicitly authorized) writable paths was touched
  (`git status --short` showed only `apps/worker-runner/Dockerfile`,
  `workers/asr-worker-py/requirements.txt`, and this ledger modified).
- Commands run: `python3 scripts/rri.py --touches
  apps/worker-runner/Dockerfile --cc 2 --D 1 --K 2 --P 0 --T 1 --A 0 --X 2`;
  `python3 scripts/delegate-low-rri.py <packet> --mode before-after
  --target-path apps/worker-runner/Dockerfile --before-file <before>
  --allow-path apps/worker-runner/Dockerfile --task-id S-230-T4k --attempt 1
  --rri 13 --no-think --temperature 0.1 --out <result>` (attempt 1, not
  applied); same command with `--attempt 2 --apply` (repair, applied);
  `docker build -t dubbridge-worker-runner-t4k:test -f
  apps/worker-runner/Dockerfile .`; `docker run --rm --entrypoint /bin/sh
  dubbridge-worker-runner-t4k:test -c '...'` (HP-1 env/import checks);
  `docker run --rm -i -e ASR_MODEL_SIZE=not-a-real-invalid-model-xyz
  --entrypoint /bin/sh dubbridge-worker-runner-t4k:test -c '...'` (EC-1);
  `bash scripts/test-production-images.sh contract worker`; `docker images
  dubbridge-worker-runner-t4k:test --format "{{.Size}}"`; `docker run --rm
  --entrypoint /bin/sh dubbridge-worker-runner-t4k:test -c 'pip3 freeze
  --all'`; `GEMMA_REVIEW_TASK_ID=S-230-T4k
  REVIEW_PATHS="apps/worker-runner/Dockerfile
  workers/asr-worker-py/requirements.txt" make qa-gemma-review`; `git status
  --short`.

### S-230-T4l: Worker ASR-bundle contract tests

**Type:** development/test
**Effort:** S — RRI 24 Low
**Depends on:** S-230-T4k
**Status:** [ ] Planned
**Writable path:** `scripts/test-production-images.sh`

Add ASR-bundle cases that codify T4k's manual evidence as a repeatable harness
case, run against the image built in T4k. **HP-1:** contract mode requires
Python 3, `workers/asr-worker-py/main.py`, exact `faster-whisper==1.1.0`,
`DUBBRIDGE_ASR_WORKER_PATH`, `DUBBRIDGE_ASR_WORKER_PYTHON`, and default
`ASR_MODEL_SIZE=small` inside the T4k image, reproducing T4k's valid-request
transcript. **EC-1:** a deliberately invalid selected model must produce a
visible protocol/job failure and never fall back to `large-v3` or a
false-ready result (matching T4k's EC-1 transcript). Evidence: RRI artifact;
Muse phase reviews; harness tests executed against the real T4k image; HP/EC
certification; owner verification. Status artifact: this ledger. Stop before
modifying `apps/worker-runner/Dockerfile` or translation bundling.

### S-230-T4m: Translation-bundle image (conditional)

**Type:** development/config
**Effort:** S — RRI 21 Low
**Depends on:** S-230-T4l and completed `S-150-T3b`/`S-150-T3c`
**Status:** [ ] Conditional — skip when the translation runtime is still open
**Writable path:** `apps/worker-runner/Dockerfile`

Bundle only the translation runtime frozen by completed S-150 tasks, naming
the exact finished worker entry point and its runtime dependencies. **HP-1:**
the bundled worker accepts a valid protocol request with the worker path and
interpreter resolved inside the image. **EC-1:** a missing entry point or
interpreter fails before queue consumption. Evidence: RRI artifact; Muse phase
reviews; build/protocol transcript; dependency inventory; image-size delta;
HP/EC certification; owner verification. Status artifact: this ledger. Stop
without changing translation source. Do not invent unfinished S-150 variable
names.

### S-230-T4n: Translation-bundle contract tests (conditional)

**Type:** development/test
**Effort:** S — RRI 24 Low
**Depends on:** S-230-T4m
**Status:** [ ] Conditional — skip when T4m is skipped
**Writable path:** `scripts/test-production-images.sh`

Add the translation-worker paths, interpreter variables, and protocol case
that codify T4m's manual evidence as a repeatable harness case, run against
the image built in T4m. **HP-1:** contract mode names the exact finished
worker entry point and its runtime dependencies, reproducing T4m's
valid-request transcript. **EC-1:** the harness case fails when the bundled
worker or dependency is absent (matching T4m's EC-1 transcript). Evidence: RRI
artifact; Muse phase reviews; harness tests executed against the real T4m
image; HP/EC certification; owner verification. Status artifact: this ledger.
Stop before modifying `apps/worker-runner/Dockerfile`.

### S-230-T4o: Full local image-pipeline contract

**Type:** development/test
**Effort:** S — RRI 25 Low
**Depends on:** S-230-T4c, T4e, T4g, T4l; T4n if executed
**Status:** [ ] Planned
**Writable path:** `scripts/test-production-images.sh`

Add the final composed test sequence without adding a production descriptor.
**HP-1:** against `infra/local/docker-compose.yml` infrastructure, the migration
image exits zero, API/gateway reach ready, and one preparation-to-ASR flow
produces observed downstream state using the built images rather than
`cargo run`. **EC-1:** a failed migration, dependency readiness failure, or ASR
failure stops the sequence and returns non-zero instead of accepting a 2xx as
success. Evidence: RRI artifact; Muse phase reviews; harness self-tests; HP/EC
certification; owner verification. Status artifact: this ledger. Stop before
running the full sequence or editing Compose.

### S-230-T4p: Execute and record local image evidence

**Type:** operational/evidence
**Effort:** S — RRI 19 Low
**Depends on:** S-230-T4o
**Status:** [ ] Planned
**Writable path:** `docs/audit/s-230-t4-local-image-evidence.md`

The primary orchestrator runs T4o and records exact commands, exit codes,
immutable base digests, image sizes, OS/Python inventories, readiness results,
migration output, and observed downstream ASR state. **HP-1:** all required
images complete the local flow. **EC-1:** one intentionally degraded dependency
produces the expected non-ready/non-zero result. If T4m/T4n were skipped, record
the translation-image rebuild debt explicitly. Evidence/status artifact: the
single audit file plus this ledger. This operational task is not a Qwen patch;
stop before provisioning Digital Ocean or editing deployment descriptors.

### S-230-T4q: T4 parent closeout and status sync

**Type:** docs-only
**Effort:** S — RRI 10 Low
**Depends on:** S-230-T4p; T4n only when its condition fired
**Status:** [ ] Planned
**Writable paths:** this ledger, `docs/plan/s-230-poc-v1-digitalocean.md`,
`docs/plan/roadmap.md`

Verify every applicable child is Done, link all phase-review and execution
artifacts, record skipped conditional children and debt, mark the T4 parent
Done, and make T5 depend on the completed aggregate. **HP-1:** all three status
documents agree on the same T4 state. **EC-1:** any open mandatory child or
missing evidence keeps T4 and T5 blocked. Evidence: `make qa-docs` and
`git diff --check`. Phase reviews, Reflection, and unit certification are `n/a`
for this docs-only closeout; do not alter implementation files or start T5.

---

## S-230-T5: Production deployment descriptor and secret boundary

**Type:** config-only
**Effort:** M
**Depends on:** S-230-T4q (which closes the S-230-T4 parent)
**Status:** [ ] Planned

**Problem (plan G5, G6):** `S-030` Phase 3 is deferred, so no production
descriptor exists; `config/production.toml` holds `*.example` placeholders and
there is no `.env.example` at the repository root.

**Acceptance criteria:**

- `infra/production/docker-compose.yml` exists, is explicitly labelled the
  production descriptor (the counterpart to the local-only banner in
  `infra/local/`), and composes reverse proxy, gateway, api, worker-runner,
  Redis, and the one-shot migration job with a startup ordering that gates the
  applications on migration success.
- An environment template enumerates every required `DUBBRIDGE_*` variable with
  no real secret values committed.
- `config/production.toml` carries real Digital Ocean values for every non-secret
  field; secrets remain injected only (ADR-026 Decision 4).
- The POC upload ceiling is set explicitly and documented as the mitigation for
  the gateway buffering debt (plan G8).
- TLS termination and the public hostname are declared.
- **The auth configuration surface is complete (plan G11).** `config/production.toml`
  has no `[auth]` block and `AppConfig::validate()` rejects `auth.is_none()` in
  production-like environments (`crates/config/src/lib.rs:196`–`200`), so the
  template must supply all five double-underscore variables that
  `Env::prefixed("DUBBRIDGE_").split("__")` expects: `DUBBRIDGE_AUTH__ISSUER`,
  `DUBBRIDGE_AUTH__AUDIENCE`, `DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH`,
  `DUBBRIDGE_AUTH__JWT_SECRET`, `DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS`.
- The template uses the double-underscore names **only**, with a comment
  recording that the single-underscore variants read by
  `AuthSettings::from_env()` (`crates/config/src/lib.rs:361`–`382`) belong to the
  legacy `AppConfig::from_env()` reader that no binary calls. Setting them
  produces no error and no effect.
- The same auth set is applied to **all three services**, since `apps/api`,
  `apps/gateway` and `apps/worker-runner` each call `AppConfig::load()` and each
  fails closed at boot on a partial set.
- `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` is set explicitly rather than left to the
  24-hour serde default (`crates/config/src/lib.rs:146`), because there is no
  refresh path. The chosen value and its rationale are recorded by `S-230-T7c`,
  which owns the decision; T5 owns only carrying it in the template.
- `rsa_public_key_path` is supplied as an explicit placeholder with an inline
  comment recording that ADR-031 made the field dead; removing it is T9 debt, not
  T5 work.
- **Conditional on `S-230-T3b`:** if `S-150-T3b`'s configurable translation
  provider is done and requires a real (non-fake) credential by the time this
  task executes, the template adds whichever `DUBBRIDGE_*` variable(s) that
  provider needs (see `S-230-T3b` §"Downstream coupling"). T5 owns only
  carrying the variable in the template, exactly as it already does for
  `S-230-T7c`'s JWT-expiry value; `T3b`'s children define the name and
  requirement.

**Evidence to emit:** the descriptor and environment template; a local
dry-run of the descriptor against local infrastructure; confirmation that no
secret value is committed.

**Status artifacts affected:** this ledger; `docs/plan/roadmap.md` S-030 Phase 3
and X21 wording; ADR-026 implementation references if the secret boundary moves.

**Handoff prompt:** Author the production Compose descriptor, environment
template, and real `config/production.toml` values. Commit no secrets.

**Stop condition:** Stop after a local dry-run of the descriptor. Do not deploy.

---

## S-230-T6: First deploy and end-to-end smoke on Digital Ocean

**Type:** operational
**Effort:** L (operational; RRI expected well below the effort impression)
**Depends on:** S-230-T5
**Status:** [ ] Planned

**Acceptance criteria:**

- Droplet, managed PostgreSQL, Spaces bucket, DNS, and TLS are provisioned and
  recorded.
- Migrations applied via the T2 runner; api, gateway, and worker report ready via
  the T3 probes.
- The first account is created against `POST /auth/register` directly — at T6
  time the mobile app still has no registration screen — and the runbook records
  the exact call. This step stays in the runbook even if `S-230-T7b` later adds
  the screen: it is the operator's recovery path when no UI is reachable.
- A real video completes the full path: login, upload, rights confirmation,
  finalize, HLS preparation, ASR, subtitle generation, review-task creation,
  approval, publication, and in-app playback — with evidence for each stage
  (audit rows, artifact records, storage keys, manifest fetch).
- **Every stage is asserted on observed downstream state, never on a 2xx alone.**
  A successful finalize response is not evidence that preparation ran; the
  corresponding artifact rows, preparation status transitions and the review task
  appearing in the inbox are. This is the direct lesson of plan G10, where a green
  API and a silently inert pipeline coexisted.
- Managed-PostgreSQL TLS behavior through `create_pool` is confirmed rather than
  assumed.
- A runbook records provisioning, deploy, migrate, rollback, log access, and
  observed timings for preparation and ASR.
- **Conditional on `S-230-T3b`/`T4`:** if the deployed worker-runner image
  bundles the translation worker (see `S-230-T4`'s conditional bullet), the
  smoke run additionally asserts a translated-subtitle artifact on observed
  downstream state for at least one target language, held to the same "never
  a 2xx alone" standard as every other stage above. If the image does not yet
  bundle it, this task proceeds exactly as originally scoped and the gap is
  recorded at T9, not silently passed over.

**Evidence to emit:** provisioning record, deploy transcript, per-stage E2E
evidence, runbook, cost summary.

**Status artifacts affected:** this ledger; `docs/plan/roadmap.md`; README status
table.

**Handoff prompt:** Provision, deploy, migrate, and drive one real video through
the entire pipeline on Digital Ocean; record evidence per stage and write the
runbook.

**Stop condition:** Stop after the smoke run and runbook. Do not change product
code to make the smoke pass — a failure is a finding, not a patch target.

---

## S-230-T7: Mobile POC build against the deployed backend

**Type:** development/operational
**Effort:** M
**Depends on:** S-230-T6
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** A build configured with the deployed `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL`
  completes login, upload, review, publish, and playback against the DO backend.

**Edge cases considered:**

- **EC-1:** A missing or malformed gateway URL surfaces the existing
  `ConfigErrorScreen` rather than failing opaquely at first request.
- **EC-2:** An expired or rejected token drives the existing logout path, not a
  silent stall.

**Acceptance criteria:**

- The POC build targets the deployed hostname over HTTPS with no local fallback
  compiled in.
- `npm run typecheck && npm run lint && npm test` stay green.
- Install and run instructions for a POC tester are recorded.

**Files expected to change:** mobile environment/build configuration only.
Product screens are expected to need no change; if any does, record why.

**Evidence to emit:** build transcript, `make qa-mobile` output, on-device
walkthrough evidence, distribution instructions.

**Status artifacts affected:** this ledger; README mobile section.

**Handoff prompt:** Produce a distributable mobile build pointed at the deployed
Digital Ocean backend and verify the full flow on a device.

**Stop condition:** Stop after the device walkthrough. Do not start T8.

---

## S-230-T7b: Mobile registration screen

**Type:** development (mobile)
**Effort:** M (provisional; recompute with `scripts/rri.py`)
**Depends on:** S-230-T7
**Status:** [ ] Planned — droppable, first drop candidate

> Added 2026-08-16 at owner request, promoting a secondary finding of the
> S-070/S-090/S-095/S-150 coverage review into planned work (plan G12). The
> `T7b` suffix keeps T8 and T9 stable.

**Problem (plan G12):** the backend is complete and the mobile surface is
missing. `apps/api/src/routes/auth.rs:23` mounts `POST /auth/register` and its
handler returns `(StatusCode::CREATED, Json(AuthSuccessResponse))` — the same
payload shape login returns, so registration can authenticate directly.
`apps/gateway/src/auth/mod.rs:16` already relays the route. But
`mobile/src/screens/` has only `LoginScreen.tsx`, and `AuthContextValue`
(`mobile/src/auth/AuthProvider.tsx:21`–`28`) exposes no `register` method.

Every tester account therefore has to be created by the operator with a direct
API call.

**Read first:** root `DESIGN.md` (mobile visual intent and component usage) and
`mobile/src/screens/LoginScreen.tsx`, whose structure and error presentation this
screen should mirror rather than reinvent.

**Happy paths considered:**

- **HP-1:** A valid email, password and workspace name submitted from the
  registration screen create the account through the gateway relay and leave the
  app authenticated on the post-login screen, with no separate login step —
  `AuthSuccessResponse` already carries `token`, `userId` and `workspaceId`.
- **HP-2:** The session persisted by registration is indistinguishable from one
  persisted by login: `saveAuthSession` stores the same shape and a relaunch
  hydrates it normally.

**Edge cases considered:**

- **EC-1:** A duplicate email returns 409 (`from_register_error`,
  `apps/api/src/routes/auth.rs:162`; covered by
  `register_handler_maps_duplicate_email_to_conflict`, `:470`) and the screen
  shows a distinct "email already registered" message rather than a generic
  failure.
- **EC-2:** A validation failure returns 400
  (`register_handler_maps_validation_errors_to_bad_request`, `:501`) and the
  screen reports which constraint failed without clearing the entered email.
- **EC-3:** A network failure or unreachable gateway surfaces the same
  `network_error` treatment `submitLogin` already uses
  (`mobile/src/auth/AuthProvider.tsx`, `loginErrorKind`), and leaves no partial
  session behind.
- **EC-4:** A missing or malformed runtime config routes to the existing
  `ConfigErrorScreen`, exactly as the login path does.
- **EC-5:** The password field is never logged, never included in an error
  message, and not retained after a failed submission.

**Acceptance criteria:**

- `AuthProvider` exposes a `register` method that shares `submitLogin`'s
  response-validation and session-persistence path rather than duplicating it;
  `isAuthSuccessPayload` remains the single acceptance check for both.
- A `RegisterScreen` exists with reciprocal navigation to and from
  `LoginScreen`.
- Registration failures map to distinct, user-readable states for 409, 400, and
  network, and are covered by tests.
- `npm run typecheck && npm run lint && npm test` stay green.
- No new API route, no change to `apps/api` or `apps/gateway`.

**Files expected to change:** `mobile/src/auth/AuthProvider.tsx`,
`mobile/src/screens/RegisterScreen.tsx` (new), the navigator, and tests.
Recompute the exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts,
`make qa-mobile` output, Reflection log if the band requires it, unit coverage
certification, owner verification, screenshot evidence of the success and the
409 states.

**Status artifacts affected:** this ledger; the plan's G12 entry; the T9 debt
register, which drops the "no registration screen" item if this task lands and
keeps it if the task is dropped.

**Handoff prompt:** Add a mobile registration screen over the existing
`POST /auth/register` relay, reusing the login session-persistence path. Do not
add API routes, password reset, email verification, or invites.

**Stop condition:** Stop after the screen registers an account and lands
authenticated. Do not touch the backend auth surface.

---

## S-230-T7c: Session lifetime and expiry behavior

**Type:** development/config (mobile + descriptor value)
**Effort:** S (provisional; recompute with `scripts/rri.py`)
**Depends on:** S-230-T7
**Status:** [ ] Planned — not a drop candidate

> Added 2026-08-16 at owner request, promoting the second secondary finding of
> the coverage review into planned work (plan G13).

**Problem (plan G13), stated precisely so it is not over-built:** the 401
handling **already exists**. `mobile/src/api/client.ts:57` maps 401 to
`session_expired` and about fifteen call sites act on it by calling
`auth.logout()` — `mobile/src/screens/useUploadFlow.ts:72`, `:82`, `:93`;
`mobile/src/screens/useReviewInboxLoader.ts:108`, `:123`;
`mobile/src/screens/useReviewDetailMutations.ts:42`, `:62`, among others. This
task must not re-implement that.

Two things are actually open:

1. `jwt_expiry_hours` has no production value. Its serde default is 24
   (`crates/config/src/lib.rs:146`) and `config/production.toml` has no `[auth]`
   block at all (plan G11), so today the deployed lifetime would be set by
   omission.
2. `hydrateStoredSession` → `acceptStoredSession`
   (`mobile/src/auth/AuthProvider.tsx`) accepts a persisted session and sets
   status `authed` without checking expiry, so an app launched after the token
   expired renders the authenticated UI and only falls back to login on the
   first 401.

**Happy paths considered:**

- **HP-1:** The POC token lifetime is an explicit value in the environment
  template with a recorded rationale, and a token issued by the deployed API
  carries exactly that `exp`.
- **HP-2:** Launching the app with a stored session that is still valid goes
  straight to the authenticated surface, unchanged from today.

**Edge cases considered:**

- **EC-1:** Launching with a stored session whose token has already expired
  routes to login immediately, without rendering the authenticated surface and
  without waiting for a request to fail.
- **EC-2:** A token that expires mid-session still produces the existing
  `session_expired` → `logout()` behavior; this task changes none of those call
  sites.
- **EC-3:** A stored session whose token cannot be parsed is treated as absent
  and cleared, matching the existing `hydrateStoredSession` catch path — a
  malformed token must never be treated as valid.
- **EC-4:** Expiry evaluation tolerates clock skew consistently with the server's
  `clock_skew_leeway_seconds`, so a client clock a few seconds fast does not
  eject a valid session.

**Acceptance criteria:**

- `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` carries an explicit POC value; the chosen
  number and the reason are recorded here and consumed by the T5 template.
- Stored-session hydration rejects an expired or unparseable token before
  setting status `authed`.
- No refresh-token mechanism, no silent renewal, and no change to the ~15
  existing `session_expired` call sites.
- `npm run typecheck && npm run lint && npm test` stay green.
- The absence of a refresh path is written into the T9 debt register as a
  deliberate POC decision, not an oversight.

**Files expected to change:** `mobile/src/auth/AuthProvider.tsx`,
`mobile/src/auth/session.ts`, tests, and the T5 environment template value.
Recompute the exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts, a test
proving an expired stored session never reaches `authed`, `make qa-mobile`
output, Reflection log if the band requires it, unit coverage certification,
owner verification.

**Status artifacts affected:** this ledger; the plan's G13 entry; `S-230-T5`
(the expiry value); the T9 debt register.

**Handoff prompt:** Set the POC token lifetime deliberately and stop an expired
stored session from rendering as authenticated. Do not add refresh tokens and do
not modify the existing `session_expired` handling.

**Stop condition:** Stop once an expired stored session routes to login on
launch. Do not touch the API's token issuance.

---

## S-230-T8: Subtitle visible in the review surface (optional)

**Type:** development
**Effort:** M
**Depends on:** S-230-T6
**Status:** [ ] Planned — droppable, second drop candidate after S-230-T7b

**Problem (plan G9):** `apps/worker-runner/src/review_enqueue.rs:35` creates
review tasks with `subtitle_artifact_id: None`, so a reviewer approves a
subtitle they cannot see.

**Explicit boundary:** this is **not** `S-150-T6`. It binds the existing nullable
column at creation time for the current subtitle only. It introduces no
generation-aware identity, no version uniqueness, no regeneration semantics, and
does not close `X-S-160-3`.

**Happy paths considered:**

- **HP-1:** Subtitle readiness creates a review task whose `subtitle_artifact_id`
  references the exact persisted subtitle artifact.
- **HP-2:** The mobile review surface renders that subtitle's content alongside
  the player before a decision is recorded.

**Edge cases considered:**

- **EC-1:** When the subtitle artifact cannot be resolved, the review task is
  still created with a null binding and the surface degrades visibly — subtitle
  readiness is never rolled back by a review-enqueue failure (current
  `prepare_review_post_ready` contract).
- **EC-2:** Pre-existing review rows with a null binding stay readable and
  decidable.
- **EC-3:** Subtitle content is only served to a caller already authorized for
  that review task's organization.

**Acceptance criteria:** artifact ID bound at creation; a read endpoint scoped by
the existing org guard; mobile rendering; no change to the ADR-030 publication
gate.

**Files expected to change:** `apps/worker-runner/src/review_enqueue.rs`,
a scoped API route, and `mobile/src/screens/ReviewDetailScreen.tsx`. Recompute
before presentation.

**Evidence to emit:** RRI report, phase reviews, Reflection log, unit coverage
certification, owner verification, screenshot evidence.

**Status artifacts affected:** this ledger; explicitly **not** `X-S-160-3`, which
stays open and owned by `S-150-T6`.

**Handoff prompt:** Bind the existing nullable subtitle artifact column at review
creation and render the subtitle in the review surface. Do not introduce
generation-aware identity.

**Stop condition:** Stop after the review surface renders the subtitle. Do not
touch S-150 artifacts.

---

## S-230-T8b: Translated subtitle visible in the review surface (conditional on T3b + T8)

**Type:** development (mobile + API)
**Effort:** M (provisional; recompute with `scripts/rri.py`)
**Depends on:** S-230-T3b (translated-subtitle artifact must exist), S-230-T8
(the review surface this task extends)
**Status:** [ ] Planned — double-conditional, not on the original 10-day path;
only reachable if both `T3b` and `T8` close

> Added 2026-08-16 at owner request, closing a coupling gap found while
> reviewing `S-230-T3b`'s dependencies: `S-230-T3b` only proves a translated
> subtitle can be *produced and persisted* (`asset_translation_status.
> current_translated_subtitle_artifact_id`, migration `0028`). Nothing makes
> it *visible* to a reviewer. `S-230-T8` renders only the source-language
> `review_tasks.subtitle_artifact_id` (migration `0026`) — a different column,
> bound at a different time, for a different artifact. Without this task, a
> completed `T3b` chain closes with a translated artifact sitting in
> Postgres that no human ever sees, which does not satisfy `S-230-T3b`'s own
> stated objective ("so the POC can demonstrate genuine cross-language
> output") — the demonstration needs a person to actually see it.

**Problem:** `review_tasks` has no column referencing a translated subtitle.
The current pointer to the latest translated subtitle for a given
`(project_id, asset_id, target_language_id)` lives on
`asset_translation_status.current_translated_subtitle_artifact_id`
(`infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql:35`),
not on the review task itself.

**Explicit boundary:** this is **not** `S-150-T6`, for the same reason `T8`
is not `S-150-T6`. It does **not** add a migration, does **not** bind a
translated-subtitle column onto `review_tasks`, introduces no generation-aware
review identity, and does not close `X-S-160-3`. It only adds a **read-only**
resolution path: given a review task's existing asset/target-language
identity, look up the current translated-subtitle pointer from the already-
existing `asset_translation_status` row and render it if present. If a future
task (`S-150-T6`) later adds real generation-aware review binding, this
read-only path is superseded, not conflicting with it.

**Happy paths considered:**

- **HP-1:** For a review task whose asset has a target language with a
  `Ready` translation status, the review surface renders both the
  source-language subtitle (via existing `T8`) and the current
  translated-subtitle text for that target language, without a migration.

**Edge cases considered:**

- **EC-1:** When `asset_translation_status` has no row, or its status is not
  `Ready`, or its `current_translated_subtitle_artifact_id` is null, the
  surface renders the source subtitle only (from `T8`) and shows the target
  language as "translation pending" rather than erroring or showing stale
  content.
- **EC-2:** Translated-subtitle content is only served to a caller already
  authorized for that review task's organization — same guard as `T8`.
- **EC-3:** A review task for an asset with no configured target language at
  all shows no translation section, matching today's behavior exactly.

**Acceptance criteria:** a read endpoint resolves the current translated
subtitle for the review task's asset/target-language via
`asset_translation_status`, scoped by the existing org guard; the mobile
review surface renders it alongside the source subtitle `T8` already added;
no migration; no change to the ADR-030 publication gate; no change to
`S-150-T6`'s eventual scope.

**Files expected to change:** a scoped API route (read-only), and
`mobile/src/screens/ReviewDetailScreen.tsx`. Recompute before presentation.

**Evidence to emit:** RRI report, phase reviews, Reflection log if the band
requires it, unit coverage certification, owner verification, screenshot
evidence of both the translation-present and translation-pending states.

**Status artifacts affected:** this ledger; explicitly **not** `X-S-160-3`,
which stays open and owned by `S-150-T6`.

**Handoff prompt:** Add a read-only resolution of the current translated
subtitle onto the existing review surface `T8` built. Do not add a migration,
do not bind a new `review_tasks` column, do not touch `S-150-T6`'s scope.

**Stop condition:** Stop once the review surface renders translated content
when present and degrades visibly when absent. Do not touch S-150 artifacts
or migrations.

---

## S-230-T9: Status, README, and debt-register closeout

**Type:** docs-only
**Effort:** S
**Depends on:** S-230-T7, and whichever of S-230-T7b / S-230-T7c / S-230-T8 /
S-230-T8b / T3b were executed rather than dropped
**Status:** [ ] Planned

**Acceptance criteria:**

- `docs/plan/roadmap.md` carries the `S-230` row, records the exact `S-150`
  reopening state (which of the six `T3b` children are done vs. still open,
  per the amended scope — no longer a blanket "parked"), and updates S-030
  Phase 3 and X9/X21 wording to match what T1/T5 actually delivered.
- README status table reflects what the deployed POC genuinely does; nothing
  deferred is described as working. If `S-230-T3b` completed all six children,
  README states the POC produces real translated-subtitle text for at least
  one target language, explicitly still without dubbed audio. If `T3b` is
  partial, README states the exact child-completion count rather than
  claiming cross-language capability.
- Debt carried by this slice is registered explicitly: gateway body buffering
  (G8), null-bound review tasks if T8 was dropped (G9), the vestigial required
  `auth.rsa_public_key_path` field that ADR-031 made dead (G11), the coexisting
  single-underscore `AuthSettings::from_env()` reader that no binary calls (G11),
  full-segment in-memory reads in `StorageAdapter::get`, the absent mobile
  registration screen **if T7b was dropped** (G12), the absence of any refresh or
  silent-renewal path (G13, a deliberate POC decision rather than an oversight),
  `StorageSettings`'s `Debug` derive leaving `access_key_id`/`secret_access_key`
  unredacted (T1 phase-2 Gemma finding, accepted-follow-up — matches the
  pre-existing unredacted `jwt_secret`/`client_secret` pattern in the same file;
  a redaction pass is cross-cutting, not T1-scoped), TTS/dubbed audio
  (`S-150-T4`–`T7`) remaining out of scope and blocked on ADR-028 regardless of
  `T3b`'s outcome, and any T6 finding not fixed in-window.
- The status of every droppable task is stated explicitly — executed or dropped,
  and why — so the debt register cannot silently omit a dropped task's gap.
- The stale roadmap note "`S-070` (JWKS) remains recommended before production
  device login" is corrected: ADR-031/S-200 moved token issuance in-house, so the
  open hardening items are `X-S-200-1` (RS256) and `X-S-200-2` (revocation), not
  JWKS discovery.
- `make qa-docs` passes.

**Evidence to emit:** documentation diff; `make qa-docs` output.

**Status artifacts affected:** `docs/plan/roadmap.md`, `README.md`, this ledger,
the S-230 plan.

**Stop condition:** Stop after documentation QA. Do not start S-150 or S-170.
