---
type: TaskList
title: "Tasks: MVP0-P2P P2 encrypted publication"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p2-encrypted-publication.md
behavioral_coverage_contract: behavior-v2
---

# P2 — encrypted P2P publication

## Parent status

- ADR-044: **Accepted 2026-09-05** — dependency satisfied.
- P1: **Done** — dependency satisfied.
- Parent P2 RRI: **131 Excessive / Effort XL** — no direct implementation.
- Parent disposition: mandatory re-scope into T0-T6 below.
- `P2.T0`: **PASS 2026-09-05** — owner selected `AN-R1 + AN-A1` and accepted the publication-state/minimum-audit contract.
- Original `P2.T1` RRI 78 parent: **SUPERSEDED / NON-EXECUTABLE** by lower-RRI decomposition.
- `P2.T1a`-`P2.T1f`: **Done / owner-approved as the complete P2.T1 persistence outcome on 2026-09-06**. Do not reopen for retrospective review.
- `P2.C0`: **PASS 2026-09-06 — RRI 66 Complex / Effort L**. Owner approved the ten-path docs/fixture freeze; four Reflection passes PASS.
- Next executable work belongs to the C0-frozen T2-T6 leaf map below. **C0 does not authorize source execution.** Each leaf must run `scripts/rri.py` on its exact current path set and follow the resulting workflow gate immediately before execution.
- Review exception: existing owner-directed MVP0-P2P P0-P7 phase-1/phase-2 review override remains in force; it does not waive RRI/HITL/Reflection/tests.

Canonical C0 evidence:

- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`
- `docs/audit/mvp0-p2p-p2-c0-rri.md`
- `docs/fixtures/mvp0-p2p-manifest-v1.json`
- `docs/fixtures/mvp0-p2p-publication-contract-v1.json`

## Parent task map

| ID | Objective | Parent RRI | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| P2.T0 | Availability Node trust/operation + O4 implementation contract | 64 Complex | L | **PASS** | ADR-044 Accepted |
| P2.T1 | Durable publication/outbox persistence parent | 78 High | XL | **SUPERSEDED — container only** | T0 PASS |
| P2.T1a–T1f | Six lower-RRI persistence leaves | per historical ledger | S/M | **Done / owner-approved** | sequential |
| P2.C0 | Shared package/crypto/AN/audit contract, golden fixtures, path ownership | **66 Complex** | L | **PASS 2026-09-06** | T1 Done |
| P2.T2 | K1 encrypted package construction + server-wrapped CK custody | 82 High parent | XL | **DECOMPOSED — no parent execution** | C0 PASS |
| P2.T3 | Availability Node ciphertext publication executor | 72 High parent | XL | **DECOMPOSED — no parent execution** | C0 PASS; T2 contract |
| P2.T4 | O4 outbox dispatch + reconciler | 84 High parent | XL | **DECOMPOSED — no parent execution** | C0 PASS; T1 Done; T3 contract |
| P2.T5 | S-120 downstream integration + fail-closed P2P_READY | 74 High parent | XL | **DECOMPOSED — no parent execution** | C0 PASS; T2 + T4 integration |
| P2.T6 | ADR-018 + deterministic crash-window certification + closure | 76 High parent | XL | **DECOMPOSED — no parent execution** | C0 PASS; T1 + T2-T5 |

The old T1/T1a planning artifacts are historical only. Low-band maximization applies to future leaves only where real seams permit it; crypto/key custody and distributed recovery are never split merely to game RRI.

## P2.T0 — Availability Node / O4 implementation contract — PASS

Owner approval on 2026-09-05 froze:

1. **Runtime:** `AN-R1` — dedicated Node.js/TypeScript Availability Node, operationally independent from mobile Bare.
2. **Publication-control authentication:** `AN-A1` — mTLS service identity on a private/non-public control endpoint.
3. **Authority:** PostgreSQL publication state + transactional outbox remain authoritative. Queue use is optional acceleration only.
4. **Idempotency:** same logical publication identity + same ciphertext/manifest identity returns stable evidence; conflicting package/hash fails closed.
5. **State:** `building -> publish_pending -> publishing -> reconciling -> ready`; `failed` terminal only.
6. **Minimum ADR-018 inventory:** intent created; K1 lineage sealed; external publication confirmed; reconciliation entered; `P2P_READY`; terminal publication failure.
7. **Secret deny-list:** no DB credential, plaintext CK, KEK, invite/viewer/business authorization, JWT signing material, or raw service secret crosses into publication payload/logs.

Four integrated T0 Reflection passes are recorded PASS. T0 did not authorize source work.

## P2.T1 — durable publication/outbox persistence — DONE OUTCOME

The former RRI-78 executable parent is a grouping label only. T1a-T1f are Done and owner-approved as one completed persistence outcome on 2026-09-06.

Accepted implementation inputs include `crates/domain/src/p2p_publication.rs`, `crates/db/src/p2p_publication_repo.rs`, and `infra/migrations/0032_create_p2p_publications_and_outbox.sql`. C0/T2-T6 may extend behavior additively, but must not retrospectively reopen T1 acceptance.

Canonical decomposition evidence: `docs/audit/mvp0-p2p-p2-t1-decomposition.md`.

## P2.C0 — shared contract and concurrency freeze — PASS

**Type:** planning/contract only  
**RRI:** **66 Complex / Effort L**  
**Depends on:** P2.T1 Done  
**Status:** [x] PASS 2026-09-06

Owner-approved freeze:

- `p2p-manifest-v1`: restricted RFC8785/JCS canonical UTF-8 JSON; NFC relative paths; `/` only; no empty/`.`/`..`; UTF-8-byte path ordering; SHA-256 lowercase-hex digests.
- `p2p-aad-v1`: asset/publication/lineage/manifest/path binding.
- AES-256-GCM: 96-bit CSPRNG nonce per file, unique under CK lineage; same-lineage retry reuses sealed ciphertext and manifest.
- one fresh 256-bit CK per new lineage; versioned KEK wrap; retry cannot silently rotate CK, KEK version, package, or lineage.
- `availability-publication-v1`: private mTLS `PUT /v1/publications/{publication_id}`; stable evidence for same identity/digest; 409 on conflict; ambiguous outcome remains non-ready.
- P2 audit correlation: no fabricated `ingest_token`; future additive audit correlation uses publication/lineage identifiers.
- `p2p-ready-descriptor-v1`: internal P3 handoff only after PostgreSQL `P2P_READY`, with opaque CK-wrap reference and no plaintext/wrapped CK bytes or viewer/device state.
- exact downstream paths and one-writer integration order frozen in the C0 audit.
- optional queue acceleration removed from October critical-path leaves.

**C0 evidence:** two golden fixtures + contract audit + RRI audit.  
**Task-analysis review:** n/a — planning/contract/docs-only exemption.  
**Code-solution review:** n/a — no source/config/migration implementation.  
**Reflection:** 4/4 PASS — determinism/lineage; trust/secrets; recovery/audit/S-120/S-230 joins; scope/ownership/governance.

---

# C0-frozen executable leaf map

**Scoring rule:** exact writable paths are now known. The `RRI` column intentionally remains `RUN BEFORE EXECUTION` until that leaf is the next executable task, so current coverage/complexity and any newly-created predecessor files are measured rather than guessed. The actual `scripts/rri.py` result controls its approval/authoring route.

## P2.T2 — K1 package construction

| ID | Objective | Exact writable paths | RRI | Status | Depends on |
|---|---|---|---|---|---|
| `T2a` | Dedicated P2 crate bootstrap + manifest/path/digest contract | — split into `T2a-i` + `T2a-ii` | 32 Moderate (parent, superseded by split) | **SPLIT — no parent execution** | C0 PASS |
| `T2a-i` | Crate skeleton bootstrap only (workspace member, crate manifest, empty lib doc) | `Cargo.toml`; `crates/p2p/Cargo.toml`; `crates/p2p/src/lib.rs` | **16 Low** | **[x] Done 2026-09-06** | C0 PASS |
| `T2a-ii` | `p2p-manifest-v1` path/digest contract types | — split into `T2a-ii-1` + `T2a-ii-2` | 29 Moderate (parent, superseded by split) | **SPLIT — no parent execution** | T2a-i |
| `T2a-ii-1` | Path normalization/validation (`normalize_path`, `sort_paths`) | `crates/p2p/src/path.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock` | **12 Low** | **[x] Done 2026-09-06** | T2a-i |
| `T2a-ii-2` | Manifest struct + `p2p-manifest-v1` canonical JSON + digest + AAD | — split into `T2a-ii-2a` + `T2a-ii-2b` | 29 Moderate (parent, superseded by split) | **SPLIT — no parent execution** | T2a-ii-1 |
| `T2a-ii-2a` | Manifest struct + `p2p-manifest-v1` canonical JSON + digest | `crates/p2p/src/manifest.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock` | **14 Low** | **[x] Done 2026-09-06** | T2a-ii-1 |
| `T2a-ii-2b` | `p2p-aad-v1` AAD builder + canonical JSON | `crates/p2p/src/aad.rs`; `crates/p2p/src/lib.rs` | **14 Low** | **[x] Done 2026-09-06** | T2a-ii-1 |
| `T2b` | Prepared-HLS package reader/snapshot | `crates/p2p/src/source.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T2a |
| `T2c` | AES-256-GCM + canonical AAD + nonce invariant | `crates/p2p/src/crypto.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T2a |
| `T2d` | Generate-once CK + versioned KEK wrap/unwrap + zeroization | `crates/p2p/src/key_wrap.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T2c |
| `T2e` | Additive sealed-K1 persistence | `infra/migrations/0033_extend_p2p_publications_k1.sql`; `crates/db/src/p2p_publication_repo.rs` | RUN BEFORE EXECUTION | Planned | T2d; T1 accepted base |
| `T2f` | Ciphertext package assembly/seal + durable manifest/package evidence | `crates/p2p/src/package_builder.rs`; `crates/p2p/src/lib.rs`; `crates/p2p/Cargo.toml`; `Cargo.lock`; `crates/db/src/p2p_publication_repo.rs` | RUN BEFORE EXECUTION | Planned | T2b–T2e |
| `T2g` | K1/golden/cross-runtime certification | `crates/p2p/tests/k1_contract.rs` | RUN BEFORE EXECUTION | Planned | T2f |

**HP-T2-1:** valid S-120 HLS -> one complete ciphertext-only K1 package with manifest/hash evidence and one server-wrapped CK lineage.  
**EC-T2-1:** nonce collision/reuse, missing input, manifest mismatch, wrap failure, or storage failure -> non-ready, no plaintext CK persisted.  
**EC-T2-2:** retry of the same lineage never creates a second CK/package or re-encrypts opportunistically.  
**EC-T2-3:** logs/errors/audit/AN payloads never reveal plaintext CK/KEK/media plaintext.

### P2.T2a-i closure record — Done 2026-09-06

**Honest Low-band split rationale.** Parent `T2a` scored **RRI 32 Moderate**
(`scripts/rri.py --touches Cargo.toml --touches crates/p2p/Cargo.toml
--touches crates/p2p/src/lib.rs --touches crates/p2p/src/manifest.rs --cc 4
--D 2 --K 2 --P 2 --T 2 --A 2 --X 1`). The split follows a real seam — a
mechanical crate skeleton with no logic versus the `p2p-manifest-v1` contract
types — not an RRI-gaming fragmentation. Per this ledger's own scoring rule,
crypto/key custody (`T2c`/`T2d`) stays unsplit. The bootstrap leaf scored
**RRI 16 Low** (`--cc 1 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1`), making it
eligible for local Qwen delegation.

**Scope delivered:** `crates/p2p` registered as a workspace member;
`dubbridge-p2p` crate manifest following the `crates/playback` convention
(`serde = { workspace = true }`, no `sha2` — the workspace does not declare
it, `[lints] workspace = true`); `lib.rs` holding only an ADR-044 doc comment
with no module declarations. No cryptography, manifest logic, or key handling.

Task-analysis review: muse-glimmer (in-session artifact, revision 2) - PASS
Code-solution review: muse-glimmer (in-session artifact) - PASS

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0-25 chain primary)
- Command: direct Ollama `/api/chat` (`num_ctx=32768`, `think=false`, `temperature=0`)
- Passes run / usable: `1/1` phase-1 (revision 2) + `1/1` phase-2
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Isolated adjudicator: `not triggered` — trigger: `n/a, primary reviewer usable`
- D14 provider route: `n/a`
- disposition_divergence: `null`
- Primary-agent disposition: phase-1 revision 1 returned `BLOCKED` with 1
  BLOCKING + 2 MAJOR + 1 MINOR finding, all **accepted as correct** — the
  packet forced the model to read files the wrapper never injects. Revision 2
  embedded the literal root `Cargo.toml` members array and the sibling
  `crates/playback/Cargo.toml`, and pre-resolved four decisions
  (append-order not alphabetical; `serde` workspace / no `sha2`; no redundant
  `#![forbid(unsafe_code)]`; no module declarations). Re-review PASS 0 findings.

### Implementation routing evidence

- **Route:** local Qwen delegation (`scripts/delegate-low-rri.py`,
  `qwen3.8:27b-mlx`), per the RRI 0-25 Low band. The
  `AGENT_WORKFLOW_GUIDE.md § Bounded cloud-implementation priority`
  exception targets Moderate/Med-high code tasks in this slice; this leaf is
  Low, and the host precheck measured 84% free memory with both pipeline
  models responding `done_reason: stop`, so the exception's stated
  memory-saturation premise did not hold at execution time.
- **Attempt 1 (`--mode full-file`, all three paths):** the two **new** files
  were produced correctly and are the ones shipped. The **existing** root
  `Cargo.toml` was regenerated from model memory rather than edited, losing
  `resolver = "3"` (-> `"2"`), `edition 2024` (-> `2021`), the proprietary
  license, every `[workspace.dependencies]` entry, and the entire clippy
  lints block. Detected immediately via `git diff` and reverted with
  `git checkout -- Cargo.toml`; nothing incorrect reached a commit.
- **Orchestrator direct edit (documented tooling-failure exception):** the
  single `  "crates/p2p",` members line was applied by the orchestrator after
  the wrapper mode proved unable to perform a bounded edit on an existing
  file. This is the tooling-failure case, not an orchestrator-diagnosed fix:
  the model's own content for the new files was accepted unchanged.
- **Lesson recorded:** `full-file` is safe only for files that do not yet
  exist; existing files require `--mode before-after`.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | `crates/p2p` builds as a workspace member | integration | `cargo check --workspace` | passed |
| EC-1 | Edge case | crate compiles standalone with no module declarations pointing at absent files | integration | `cargo clippy -p dubbridge-p2p --all-features` | passed |
| EC-2 | Edge case | no unrelated workspace file is altered by the change | contract | `git diff --stat` -> `Cargo.toml \| 1 +, 1 insertion(+)` | passed |

### Owner final verification

- Owner: `Claude Opus 5 (orchestrator of record, under owner-delegated
  autonomous authority granted 2026-09-06 for the absence window)`
- Date: `2026-09-06`
- Statement: I verified the crate builds inside the workspace, contains no
  out-of-scope cryptographic/manifest/key logic, and that the applied diff is
  exactly one added workspace-member line plus two new files. The destructive
  first delegation attempt was reverted before any commit and is recorded
  above rather than omitted.
- Commands run: `cargo check --workspace`; `cargo fmt --check`;
  `cargo clippy --workspace --all-features`; `git diff --stat`;
  `git diff Cargo.toml`

---

### P2.T2a-ii-1 closure record — Done 2026-09-06

**Honest Low-band split rationale.** Parent `T2a-ii` scored **RRI 29
Moderate** (`--touches crates/p2p/src/manifest.rs --touches
crates/p2p/src/lib.rs --touches crates/p2p/Cargo.toml --cc 6 --D 2 --K 1 --P
2 --T 1 --A 1 --X 2`). Path normalization/validation is a real seam: a pure
function pair with rules and test vectors already frozen by the C0 golden
fixture (`docs/fixtures/mvp0-p2p-manifest-v1.json`), fully separable from the
manifest struct/canonical-JSON/digest logic that remains `T2a-ii-2`. Split
leaf scored **RRI 12 Low** (`--cc 5 --D 1 --K 0 --P 1 --T 1 --A 0 --X 1`).

**Scope delivered:** `crates/p2p/src/path.rs` implementing
`normalize_path(&str) -> Result<String, PathError>` (NFC normalization, then
reject-empty / reject-backslash / reject-absolute / per-segment
empty-dot-parent checks, in that fixed order) and `sort_paths(&mut
[String])` (native byte-ordered `str::sort`), plus a `PathError` enum with
manual `Display`/`Error` impls. Orchestrator added the
`unicode-normalization = "0.1"` dependency and `pub mod path;` directly
(single-line edits to existing files, per the `full-file`-is-unsafe-for-
existing-files lesson from `T2a-i`).

Task-analysis review: muse-glimmer (in-session artifact) - PASS
Code-solution review: muse-glimmer (in-session artifact) - PASS

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0-25 chain primary)
- Command: direct Ollama `/api/chat` (`num_ctx=32768`, `think=false`, `temperature=0`)
- Passes run / usable: `1/1` phase-1 + `1/1` phase-2
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Isolated adjudicator: `not triggered`
- D14 provider route: `n/a`
- disposition_divergence: `null`
- Primary-agent disposition: phase-1 PASS on first packet (0 findings, fully
  frozen contract with pre-resolved dependency/error-type/API/validation-
  order decisions). Phase-2 PASS after a repair cycle — see routing evidence.

### Implementation routing evidence

- **Route:** local Qwen delegation (`scripts/delegate-low-rri.py --mode
  full-file`, `qwen3.8:27b-mlx`), RRI 0-25 Low band, single new file.
- **Attempt 1:** produced fully correct logic (all 6 frozen vectors +
  `PathError` + both public functions) but appended one stray non-Rust
  trailing line (`--- CONTENT ---`, 16 bytes, no leading newline in the
  file) after the closing brace, breaking compilation
  (`error: expected item, found '-'`). Not a logic defect — confirmed by
  the raw delegation JSON, the marker sits inside the model's own emitted
  "contents" field.
- **Bounded repair attempt (1/1):** a `--mode before-after` repair packet
  targeting only that trailing line was constructed, but the wrapper's own
  40-line BEFORE-anchor safety guard rejected it (the file's true tail has
  no trailing newline, so a small anchor extraction produced a
  whole-file-sized BEFORE span). This is the **documented tooling-failure
  exception**: the fix was correctly diagnosed, but the delegation wrapper
  could not construct a usable diff for this anchor shape.
- **Orchestrator direct edit (tooling-failure exception):** removed exactly
  the trailing `\n--- CONTENT ---` byte sequence via a scripted, asserted
  string match (`content.endswith(...)` guard before writing) — no Rust
  token, enum variant, function body, or test was touched. Verified
  identical logic before/after via `cargo test`/`clippy`/`fmt`, all clean.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | ordinary relative paths normalize unchanged | unit | `crates/p2p/src/path.rs::tests::test_index_m3u8`, `test_segments_000001_ts` | passed |
| HP-2 | Happy path | NFD input normalizes to NFC | unit | `crates/p2p/src/path.rs::tests::test_captions_cafe_nfd` | passed |
| EC-1 | Edge case | parent-segment path rejected | unit | `crates/p2p/src/path.rs::tests::test_parent_segment` | passed |
| EC-2 | Edge case | absolute path rejected | unit | `crates/p2p/src/path.rs::tests::test_absolute_path` | passed |
| EC-3 | Edge case | backslash rejected | unit | `crates/p2p/src/path.rs::tests::test_backslash` | passed |
| EC-4 | Edge case | sort is byte-ascending | unit | `crates/p2p/src/path.rs::tests::test_sort_paths` | passed |

### Owner final verification

- Owner: `Claude Opus 5 (orchestrator of record, under owner-delegated
  autonomous authority granted 2026-09-06 for the absence window)`
- Date: `2026-09-06`
- Statement: I verified all six C0-frozen path vectors pass exactly as
  specified, no cryptography/manifest/digest logic leaked into this leaf,
  and the only orchestrator-authored change is the documented mechanical
  removal of a non-Rust delegation artifact, with before/after logic
  equivalence confirmed by an unchanged test/clippy/fmt result.
- Commands run: `cargo test -p dubbridge-p2p`; `cargo clippy -p dubbridge-p2p
  --all-features`; `cargo fmt --check`

---

### P2.T2a-ii-2a closure record — Done 2026-09-06

**Honest Low-band split rationale.** Parent `T2a-ii-2` (manifest struct +
canonical JSON + digest + AAD) scored **RRI 29 Moderate**
(`--cc 8 --D 2 --K 1 --P 2 --T 1 --A 1 --X 2`). The `p2p-manifest-v1`
canonical-serialization/digest concern and the `p2p-aad-v1` AAD-builder
concern are two independent golden fixtures in the same C0 file
(`manifest` vs. `aad_example`) with independent test vectors — a genuine
pre-existing seam, split into `T2a-ii-2a` (this leaf, RRI 14 Low,
`--cc 5 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1`) and `T2a-ii-2b` (AAD, RRI 14
Low, not yet executed).

**Scope delivered:** `crates/p2p/src/manifest.rs` defining `Manifest` /
`ManifestFile` with field declaration order matching the frozen
ASCII-sorted JSON key order (the entire canonicalization mechanism —
`serde_json::to_string` preserves declaration order and adds no
whitespace), plus `canonical_json(&Manifest) -> String` and
`manifest_sha256(&str) -> String` (hand-rolled lowercase hex over
`sha2::Sha256::digest`). Orchestrator added `serde_json = { workspace =
true }` and `sha2 = "0.10"` to `crates/p2p/Cargo.toml` and `pub mod
manifest;` to `lib.rs` directly (mechanical existing-file edits, per the
`T2a-i` full-file lesson).

**Fixture pre-verification:** before building the delegation packet, the
orchestrator independently recomputed both `expected_manifest_sha256` and
the golden canonical string's SHA-256 with Python `hashlib` and confirmed
they matched the committed fixture — the delegated test therefore asserts
against a value verified by a second, independent implementation, not
only the fixture file's own claim.

Task-analysis review: muse-glimmer (in-session artifact) - PASS
Code-solution review: muse-glimmer (in-session artifact) - PASS

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0-25 chain primary)
- Command: direct Ollama `/api/chat` (`num_ctx=32768`, `think=false`, `temperature=0`)
- Passes run / usable: `1/1` phase-1 + `1/1` phase-2
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Isolated adjudicator: `not triggered`
- D14 provider route: `n/a`
- disposition_divergence: `null`
- Primary-agent disposition: both phases PASS on first packet with 0
  findings; implementation succeeded on the first delegation attempt with
  no repair cycle needed.

### Implementation routing evidence

- **Route:** local Qwen delegation (`scripts/delegate-low-rri.py --mode
  full-file`, `qwen3.8:27b-mlx`), RRI 0-25 Low band, single new file.
- **Attempt 1:** succeeded outright — 8/8 tests passed (7 existing +1 new)
  on first compile, including the exact frozen canonical-string and
  digest assertions. `cargo fmt --check` flagged only line-wrap/
  trailing-newline formatting (not a finding per policy); `cargo fmt`
  applied once, re-verified 8/8 still passing afterward. No repair
  attempt was needed.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | manifest serializes to the exact frozen canonical JSON byte sequence | unit | `crates/p2p/src/manifest.rs::tests::test_canonical_json_and_digest` | passed |
| HP-2 | Happy path | canonical JSON digests to the exact frozen SHA-256 value | unit | `crates/p2p/src/manifest.rs::tests::test_canonical_json_and_digest` | passed |

### Owner final verification

- Owner: `Claude Opus 5 (orchestrator of record, under owner-delegated
  autonomous authority granted 2026-09-06 for the absence window)`
- Date: `2026-09-06`
- Statement: I verified the canonical JSON and digest match the C0-frozen
  golden fixture exactly, independently re-verified with a second
  implementation (Python hashlib) before delegation, and that no
  AES-GCM/key/AAD logic leaked into this leaf.
- Commands run: `cargo test -p dubbridge-p2p`; `cargo clippy -p dubbridge-p2p
  --all-features`; `cargo fmt --check`; `python3 -c "... hashlib.sha256 ..."`
  cross-check against `docs/fixtures/mvp0-p2p-manifest-v1.json`

---

### P2.T2a-ii-2b closure record — Done 2026-09-06

**Honest Low-band split rationale.** See `T2a-ii-2a` above — same parent
`T2a-ii-2` split at the manifest/AAD seam. This leaf scored **RRI 14 Low**
(`--cc 2 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1`).

**Scope delivered:** `crates/p2p/src/aad.rs` defining `Aad` with field
declaration order matching the frozen `p2p-aad-v1` canonical JSON key
order, plus `canonical_aad_json(&Aad) -> String`. Reuses
`crate::manifest::manifest_sha256` in its own test rather than duplicating
digest logic, proving that function is generic over any canonical string.
Orchestrator added `pub mod aad;` to `lib.rs` directly (mechanical
existing-file edit).

Task-analysis review: muse-glimmer (in-session artifact) - PASS
Code-solution review: muse-glimmer (in-session artifact) - PASS

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0-25 chain primary)
- Command: direct Ollama `/api/chat` (`num_ctx=32768`, `think=false`, `temperature=0`)
- Passes run / usable: `1/1` phase-1 + `1/1` phase-2
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Isolated adjudicator: `not triggered`
- D14 provider route: `n/a`
- disposition_divergence: `null`
- Primary-agent disposition: both phases PASS, 0 findings.

### Implementation routing evidence

- **Route:** local Qwen delegation (`scripts/delegate-low-rri.py --mode
  full-file`, `qwen3.8:27b-mlx`), RRI 0-25 Low band, single new file.
- **Attempt 1:** produced fully correct logic (10/10 tests incl. both new
  assertions) but again appended the same stray non-Rust trailing line
  (`--- CONTENT ---`) already diagnosed in `T2a-ii-1`'s closure record —
  now confirmed as a recurring defect class: the model echoes the
  wrapper's own tagged-block `CONTENT_MARKER` literal
  (`scripts/delegate-low-rri.py:73`) as a spurious closing bookend. The
  file (53 lines) again exceeded the repair wrapper's 40-line anchor cap,
  so rather than re-running a repair delegation already known to fail the
  same guard, the orchestrator applied the same previously-reviewed
  scripted, asserted byte-removal fix directly (documented tooling-failure
  exception), verified identical logic via `cargo test`/`clippy`/`fmt`
  (all clean, no diff needed after removal). Recorded in
  `feedback_full_file_appends_content_marker_echo` for future sessions so
  this defect class is recognized without a second diagnostic pass.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | AAD serializes to the exact frozen canonical JSON byte sequence | unit | `crates/p2p/src/aad.rs::tests::test_canonical_aad_json` | passed |
| HP-2 | Happy path | canonical AAD JSON digests to the exact frozen SHA-256 value via the reused `manifest_sha256` | unit | `crates/p2p/src/aad.rs::tests::test_canonical_aad_json_sha256` | passed |

### Owner final verification

- Owner: `Claude Opus 5 (orchestrator of record, under owner-delegated
  autonomous authority granted 2026-09-06 for the absence window)`
- Date: `2026-09-06`
- Statement: I verified the AAD canonical JSON and its digest match the
  C0-frozen golden fixture exactly, that `manifest_sha256` reuse is
  correct and duplicates no logic, and that no encryption/key-handling
  code leaked into this leaf.
- Commands run: `cargo test -p dubbridge-p2p`; `cargo clippy -p dubbridge-p2p
  --all-features`; `cargo fmt --check`

---

## P2.T3 — Availability Node executor

| ID | Objective | Exact writable paths | RRI | Status | Depends on |
|---|---|---|---|---|---|
| `T3a` | Node/TS service bootstrap + v1 contract validation | `apps/availability-node/package.json`; `apps/availability-node/package-lock.json`; `apps/availability-node/tsconfig.json`; `apps/availability-node/src/contract.ts`; `apps/availability-node/src/server.ts` | RUN BEFORE EXECUTION | Planned | C0 PASS |
| `T3b` | Private mTLS listener + client-identity policy | `apps/availability-node/src/mtls.ts`; `apps/availability-node/src/server.ts` | RUN BEFORE EXECUTION | Planned | T3a |
| `T3c` | Persistent Hyperdrive seed/open + idempotency/conflict behavior | `apps/availability-node/src/hyperdrive_store.ts`; `apps/availability-node/src/server.ts` | RUN BEFORE EXECUTION | Planned | T3b |
| `T3d` | Contract/mTLS/idempotency/traversal/secret certification | `apps/availability-node/test/publication_contract.test.ts`; `apps/availability-node/test/fixtures.ts` | RUN BEFORE EXECUTION | Planned | T3c |

**HP-T3-1:** authenticated same publication+lineage+manifest digest returns stable publication evidence.  
**EC-T3-1:** same logical identity with conflicting lineage/digest -> 409 fail-closed.  
**EC-T3-2:** untrusted mTLS or package path escape -> rejected; service has no DB/business/key authority.

## P2.T4 — O4 dispatcher and reconciler

| ID | Objective | Exact writable paths | RRI | Status | Depends on |
|---|---|---|---|---|---|
| `T4a` | Pure recovery decision kernel | `crates/domain/src/p2p_recovery.rs`; `crates/domain/src/lib.rs` | RUN BEFORE EXECUTION | Planned | C0 PASS; T1 accepted base |
| `T4b` | Bounded PostgreSQL claim/lease/release | `infra/migrations/0034_add_p2p_publication_claim_leases.sql`; `crates/db/src/p2p_publication_repo.rs` | RUN BEFORE EXECUTION | Planned | T4a |
| `T4c` | Backend mTLS Availability Node client | `crates/connectors/src/p2p_availability.rs`; `crates/connectors/src/lib.rs`; `crates/connectors/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T3 contract PASS |
| `T4d` | PostgreSQL outbox dispatcher | `crates/jobs/src/p2p_publication_job.rs`; `crates/jobs/src/lib.rs`; `crates/jobs/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T4b; T4c |
| `T4e` | Worker runtime + PostgreSQL reconciler integration | `apps/worker-runner/src/p2p_publication_runtime.rs`; `apps/worker-runner/src/main.rs`; `apps/worker-runner/Cargo.toml`; `Cargo.lock` | RUN BEFORE EXECUTION | Planned | T4d |
| `T4f` | Lost-dispatch/lost-ACK/stale-lease/duplicate certification | `apps/worker-runner/tests/p2p_publication_recovery_test.rs` | RUN BEFORE EXECUTION | Planned | T4e |

**HP-T4-1:** committed-before-dispatch crash is recovered from PostgreSQL.  
**HP-T4-2:** no queue accelerator exists -> direct dispatch/reconciliation still converges.  
**EC-T4-1:** timeout/unknown result stays non-ready and same-lineage reconciliation proves/re-drives it.  
**EC-T4-2:** duplicate-after-Ready is idempotent; no lineage rotation/regression.

Optional queue acceleration is deferred and requires its own later task/RRI; queue state can never establish readiness.

## P2.T5 — S-120 activation + P2P_READY

| ID | Objective | Exact writable paths | RRI | Status | Depends on |
|---|---|---|---|---|---|
| `T5a` | Characterize existing Ready/transcription ordering before activation | `apps/worker-runner/src/preparation_runtime_tests/p2p_activation.rs`; `apps/worker-runner/src/preparation_runtime_tests.rs` | RUN BEFORE EXECUTION | Planned | C0 PASS |
| `T5b` | Fail-contained P2 activation after Ready + transcription post-ready | `apps/worker-runner/src/p2p_activation.rs`; `apps/worker-runner/src/preparation_runtime.rs`; `apps/worker-runner/src/main.rs` | RUN BEFORE EXECUTION | Planned | T2 PASS; T4 integration PASS; T5a |
| `T5c` | Authoritative ready read model + `p2p-ready-descriptor-v1` | `crates/domain/src/p2p_ready_descriptor.rs`; `crates/domain/src/lib.rs`; `crates/db/src/p2p_publication_repo.rs` | RUN BEFORE EXECUTION | Planned | T5b |
| `T5d` | S-120/ASR + no-false-ready non-regression | `apps/worker-runner/tests/p2p_s120_non_regression_test.rs` | RUN BEFORE EXECUTION | Planned | T5c |

**HP-T5-1:** S-120 Ready and transcription enqueue attempt complete independently while P2 proceeds.  
**HP-T5-2:** durable same-lineage package + external confirmation -> PostgreSQL P2P_READY + minimal P3 descriptor.  
**EC-T5-1:** P2 failure never undoes/delays S-120 Ready; only P2 stays unavailable.  
**EC-T5-2:** queue ACK, AN reachability, or dispatch attempt without durable confirmation remains non-ready.

## P2.T6 — audit + deterministic certification + closure

| ID | Objective | Exact writable paths | RRI | Status | Depends on |
|---|---|---|---|---|---|
| `T6a` | Backward-compatible P2 audit correlation schema | `infra/migrations/0035_extend_audit_events_p2p_correlation.sql` | RUN BEFORE EXECUTION | Planned | C0 PASS |
| `T6b` | Six P2 audit kinds + durable emitter/repository correlation | `crates/domain/src/audit/kind.rs`; `crates/domain/src/audit/event.rs`; `crates/domain/src/audit/tests.rs`; `crates/db/src/audit_repo.rs`; `crates/audit/src/lib.rs` | RUN BEFORE EXECUTION | Planned | T6a |
| `T6c` | Deterministic six-window crash/recovery harness | `apps/worker-runner/tests/p2p_crash_windows_test.rs` | RUN BEFORE EXECUTION | Planned | T2-T5 integration PASS; T6b |
| `T6d` | Ciphertext-only + secret-deny certification | `apps/worker-runner/tests/p2p_secret_boundary_test.rs`; `apps/availability-node/test/secret_boundary.test.ts` | RUN BEFORE EXECUTION | Planned | T3 PASS; T6b |
| `T6e` | P2 evidence/status closeout only | `docs/audit/mvp0-p2p-p2-t6-closure.md`; `docs/plan/mvp0-p2p-p2-encrypted-publication.md`; `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`; `docs/plan/mvp0-p2p-first.md`; `docs/tasks/mvp0-p2p-first.md`; `docs/plan/roadmap.md` | RUN BEFORE EXECUTION | Planned | T6c; T6d; all P2 evidence PASS |

**HP-T6-1:** clean publication produces durable, correlated audit evidence and closes P2 only after all acceptance evidence passes.  
**EC-T6-1:** each injected D3 crash window converges without false Ready or second lineage.  
**EC-T6-2:** required audit persistence failure fails closed where ADR-018 requires it.  
**EC-T6-3:** P2 audit correlation never fabricates/overloads legacy ingestion-token meaning.

## Integration and ownership constraints

- Current workflow executes one approved task ID at a time. Planning/fixture/test-design work may be prepared concurrently, but executable task IDs are not claimed concurrent.
- ADR-040 is the repository's existing **per-module complexity-split routing** decision. When an approved RRI 26–55 task qualifies for multiple writers, use one orchestrator, common base SHA, frozen interfaces, disjoint writable paths, one writer per path, orchestrator-only shared files, and whole-task integration/verification.
- `Cargo.lock`, root `Cargo.toml`, crate module exports, worker `main.rs`, shared repository files, migrations, status ledgers, and any file named by multiple sequential leaves are integration-owner paths. Sequential tasks may modify them only under their own current approval.
- Future migrations are reserved in integration order: `0033` K1 persistence, `0034` claim/leases, `0035` P2 audit correlation. C0 itself created no migration.
- Queue acceleration is not required for October P2 PASS.

## S-230 dependency sync

C0 freezes the contracts consumed later by these cross-slice gates; it no
longer activates the deployment lane by itself:

- `S-230-T6p-a` requires `S-230-T7local PASS + S-230-T7c PASS + MVP0-P2P
  P2-P6 PASS` — the local-Docker-Compose mobile validation, not the
  post-deploy `S-230-T7` confirmation — and freezes only deployment-specific
  ownership/configuration against those implemented surfaces. It consumes
  the existing C0 contracts/fixtures and does not redefine them.
- `S-230-T6p-b` depends on `T6p-a PASS`.
- `S-230-T6p-c` depends on `T6p-b PASS`.
- `S-230-T6p-d` depends on `T6p-c PASS` and proves only backend ciphertext
  publication plus durable `P2P_READY`.
- Invited playback additionally requires T7p, P7, and T9g against the exact
  deployed artifact; T6p-d does not demonstrate it.

## Parent closure criteria

P2 is PASS only when T0, T1, C0, and all required T2-T6 leaves are closed, every HP/EC maps to passing executable evidence, integrated Reflection covers confidentiality, crash consistency, readiness separation, and scope, canonical docs are synchronized, and the repository owner performs final verification.

P3 remains blocked until that P2 PASS is recorded.
