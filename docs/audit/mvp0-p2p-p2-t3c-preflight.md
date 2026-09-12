---
type: Audit
title: "MVP0-P2P P2.T3c preflight"
status: complete
task: P2.T3c
---

# P2.T3c — preflight

## Result

- **Status:** `ALCANCE CONGELADO` (scope frozen). D1-D5 resolved with direct
  repository/contract evidence. `P2.T3c`'s own envelope now includes a new
  Rust materializer leaf in `crates/p2p` in addition to its original
  Availability Node leaf — this is the runbook-directed resolution of D2
  ("amplia el alcance de T3c" rather than opening a separate predecessor
  task), not a new task.
- **Local Architect (`qwen3.6:27b-q4_K_M`) disposition:** not invoked this
  session. The prior session already exhausted its two-attempt budget
  (full profile + reduced profile, both timed out with no response —
  `.agent/p2-t3c/local-architect-result.json`,
  `.agent/p2-t3c/local-architect-result-reduced.json`). Per explicit
  orchestrator instruction for this session, a third attempt was not made.
  D1-D5 below are resolved entirely from direct repository/contract
  evidence, consistent with the advisory-only, non-mandatory nature of that
  role (ADR-037).

## State of dependencies

- T3b: satisfied — Done and owner-verified 2026-09-09
  (`docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § P2.T3b).
- T2 package contract: Done. `crates/p2p/src/package_builder.rs::build_package`
  returns `SealedPackage { manifest, manifest_canonical_json,
  manifest_digest_sha256, files }` in memory only; no filesystem I/O.
- C0 contract: satisfied. `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` §3
  freezes `availability-publication-v1`: the backend's `PUT
  /v1/publications/{publication_id}` request to the Availability Node
  **carries** `package_ref` (`packages/<publication_id>/<lineage_id>`)
  relative to the Node's configured ciphertext root. This is the decisive
  evidence for D2: the request contract requires ciphertext bytes to
  already exist under `package_ref` on the shared volume *before* the
  Availability Node is called — the Node only opens/seeds what is already
  there; it does not write the package's own ciphertext members. The C0
  table's `P2.T3c` writable-path row (`hyperdrive_store.ts`, `server.ts`)
  reflects exactly that seed/open responsibility, not materialization.

## D1 — Direct dependencies

- Node checked: `v22.23.0`; npm `10.9.8`.
- Current Availability Node direct dependencies: none of `corestore`,
  `hyperdrive`, or `hyperswarm` are declared (`apps/availability-node/package.json`).
- Proposed exact direct versions: `corestore@7.12.5`, `hyperdrive@13.3.3`,
  `hyperswarm@4.17.1`.
- Compatibility evidence: isolated exact-version install + Node 22 ESM
  import of all three packages passed 2026-09-12 (prior session). No
  `engines` constraint blocks Node 22 per queried npm metadata.
- Decision: `RESOLVED FOR PROPOSAL`. Declared/lockfile-resolved versions are
  recorded when the T3c Node-side leaf is implemented, not at preflight time.

## D2 — Materialization of `package_ref`

- Current producer: none in Rust or TypeScript.
- Current in-memory boundary: `crates/p2p/src/package_builder.rs::build_package`
  (pure function, no I/O).
- Existing filesystem-write precedent in the same architectural layer:
  `crates/storage/src/local.rs` already owns `fs::write`-based atomic-ish
  object writes for the (separate) `StorageAdapter` object store — proving
  the codebase's established pattern for a Rust-owned durable-write boundary
  sits in a dedicated crate module, not inside API handlers or the Node
  service.
- Root and relative path: C0 §3 freezes the relative shape
  `packages/<publication_id>/<lineage_id>` under a dedicated shared
  ciphertext-only volume. No canonical manifest filename was previously
  frozen for that layout.
- **Decision: `RESOLVED — new Rust materializer leaf inside `crates/p2p`.`**
  Because the request contract (C0 §3) places `package_ref` construction as
  a precondition the caller (the future Rust T4c mTLS Availability-Node
  client) must satisfy before issuing `PUT /v1/publications/{id}`, the
  writer belongs on the Rust/backend side of the P2P boundary, in the same
  crate that already owns `SealedPackage` construction
  (`crates/p2p/src/package_builder.rs`) — not inside the Node.js
  Availability Node, and not as a new crate. This keeps the ciphertext-only
  Node boundary intact (ADR-044: the Node gains no business/materialization
  authority) and reuses an existing crate rather than opening one.
  - Canonical manifest filename (frozen for this leaf): `manifest.json`,
    containing `manifest_canonical_json` bytes exactly as produced by
    `build_package`, written first-or-atomically alongside the digest-named
    ciphertext members.
  - Ciphertext member layout: each `SealedFile` is written at its normalized
    relative member path (already validated by `crates/p2p/src/path.rs`)
    under `packages/<publication_id>/<lineage_id>/`.
  - Atomicity: write to a sibling temp directory
    (`packages/<publication_id>/<lineage_id>.tmp-<random>`) then rename into
    place, matching the "no partial package visible to a concurrent reader"
    requirement implied by C0's idempotency contract. This mirrors the
    write-then-visible-rename discipline `crates/storage` already documents
    for the object store (ADR-006 immutable-artifact spirit), applied here
    to the separate P2P shared volume.
  - This leaf does **not** call the Availability Node, does not do network
    I/O, and does not decide `P2P_READY` — it is a pure materialization
    function plus a thin directory-write wrapper, consumed later by the
    still-`Planned` `P2.T4c` (mTLS AN client) leaf, which is out of scope
    for T3c.

## D3 — Root containment

- Ciphertext-package read/write root: a dedicated, injected shared volume
  path (config-provided, not hardcoded), used identically by the new Rust
  materializer leaf (write side) and the Availability Node's
  `hyperdrive_store.ts` (read/seed side). T3c does not select or wire the
  concrete deployment path/mount — that remains T4c/deployment scope
  (excluded per the ledger's existing Exclusions list).
- Availability Node persistent storage root: a separate, distinct injected
  root for Corestore/Hyperdrive's own persistent state — never the same
  directory as the ciphertext-package root, so a package write can never
  collide with or overwrite Hyperdrive's internal storage.
- Write ownership on the shared ciphertext-package root: the new Rust
  materializer leaf (`crates/p2p/src/package_writer.rs`) is the sole writer;
  the Availability Node's `hyperdrive_store.ts` only reads/opens/seeds from
  that root and never writes to it — the two leaves share the same root path
  but not writable responsibility over it (D14 phase-1 review clarification,
  `.agent/p2-t3c/phase1-review.json`).
- Containment rules (apply on both the new Rust write-side path resolution
  and the Node read-side path resolution — reusing the pattern already
  proven in `crates/p2p/src/path.rs::normalize_path`, which already rejects
  absolute paths, backslashes, empty/dot/parent segments):
  - reject an absolute `package_ref`;
  - reject `..`, backslash, or any segment that normalizes to a parent/dot
    segment (existing `PathError` variants already cover this on the Rust
    side; the Node leaf must apply the equivalent check before path join);
  - after joining root + relative `package_ref`, canonicalize
    (`realpath`/`fs.realpath`) and reject if the result does not stay
    strictly under the canonicalized root;
  - reject any path component that is itself a symlink (`lstat` each
    intermediate segment) rather than only checking the final resolved
    target, so a symlinked intermediate directory cannot silently escape;
  - a manifest that references a member path failing any of the above is
    `422 package_invalid` and is never opened/seeded.
- Decision: `RESOLVED`. Verifiable without new deployment configuration —
  both roots are constructor/function parameters, not environment lookups,
  in this leaf's scope.

## D4 — Identity and durable evidence

- Scope boundary confirmed from C0 + the existing D4 preserved constraint:
  the Availability Node **must not** use PostgreSQL or assert `P2P_READY`
  (`docs/playbooks/P2_T3C_ORCHESTRATOR_RUNBOOK.md` §8). `P2P_READY` remains
  exclusively a backend/PostgreSQL decision made later by `P2.T5`, entirely
  outside T3c.
- Durable record (Node-local, not PostgreSQL): `hyperdrive_store.ts` owns a
  local persistent index (e.g., an append-only or key-value record inside
  the Node's own storage root — distinct from the ciphertext-package root
  per D3) keyed by `(publication_id, lineage_id)`, storing
  `manifest_digest_sha256`, the derived stable `external_publication_id`
  (the Hyperdrive public key/discovery identifier is a natural fit — stable
  across restarts because Hyperdrive derives it from the persistent
  Corestore keypair, not regenerated per call), a generated `evidence_id`,
  and the original `confirmed_at` (RFC3339 UTC), written once and never
  overwritten on replay.
- Conflict detection: before any write, look up by `publication_id` alone;
  if found with a different `lineage_id` or `manifest_digest_sha256` than
  the stored record, return `409 publication_conflict` without touching the
  existing record or opening a second drive (per C0 §3 idempotency rule and
  acceptance criterion 3).
- Success-ordering rule (prevents false-success): the durable record is
  written **only after** package validation (digest/size/path containment,
  D3) succeeds **and** the Hyperdrive drive is opened **and** seeding is
  announced **and** any required flush completes — in that order. A failure
  at any earlier step returns `422`/`503` (per D5) and commits nothing.
- Ambiguous-failure retry safety: because the durable record is the sole
  source of "was this already confirmed," and it is only written after the
  full success sequence, a retry after an ambiguous failure (process crash
  mid-seed, network timeout before the caller saw the response) is always
  safe — the retry either finds no record (safe to reattempt the full
  sequence) or finds a complete record for the identical tuple (safe
  idempotent `200` replay per acceptance criterion 2).
- Decision: `RESOLVED`.

## D5 — Concurrency and lifecycle

- Same-`publication_id` serialization: `server.ts`/`hyperdrive_store.ts`
  serializes concurrent requests for the same `publication_id` (e.g., an
  in-process async mutex/queue keyed by `publication_id`) so two concurrent
  identical or conflicting requests cannot race to open two drives or
  produce two differing evidence records.
- Different-`publication_id` requests: proceed independently and
  concurrently; no cross-ID lock.
- `Hyperswarm.join()` completion: treated complete only after the
  library-reported join/flush resolves (awaited, not fire-and-forget)
  before the durable record (D4) is written and `201`/`200` returned.
- Long-lived seed ownership: the Availability Node process itself owns and
  keeps alive each opened Corestore/Hyperdrive/Hyperswarm instance for the
  process lifetime (or until an explicit future close/eviction mechanism —
  out of T3c's scope); T3c does not implement eviction.
- Restart reconstruction: on process restart, the durable index (D4) is the
  source of truth for which publications were previously confirmed; the
  Node reconstructs/reopens each known drive from the persistent storage
  root (D3) lazily on next request, not eagerly for all records at boot
  (avoids an unbounded startup fan-out; explicit for T3c to keep scope
  bounded — eager warm-start is a legitimate future optimization, not
  required for this leaf's acceptance criteria).
- Idempotent `close()`: any close path (tests, future shutdown hook) must be
  safe to call more than once without throwing — required only for the
  test-owned lifecycle in `hyperdrive-store.test.js`, not a running-service
  concern this leaf must wire to a signal handler.
- Failure mapping: every open/write/join/flush/persistence failure maps to
  `503 publication_unavailable`; package/path/digest validation failure
  maps to `422 package_invalid`; neither ever writes a durable D4 record.
- Decision: `RESOLVED`.

## Alcance congelado (frozen scope)

- **Writable paths (final):**
  - `crates/p2p/src/package_writer.rs` (new — Rust materializer: atomic
    directory write of `SealedPackage` under an injected shared ciphertext
    root; returns/validates `package_ref`)
  - `crates/p2p/src/lib.rs` (module export wiring only)
  - `crates/p2p/tests/package_writer_test.rs` (new — focused RED/GREEN
    coverage for D2/D3: atomic write, traversal rejection, symlink
    rejection, idempotent overwrite-of-identical-content behavior)
  - `apps/availability-node/package.json`
  - `apps/availability-node/package-lock.json`
  - `apps/availability-node/src/hyperdrive_store.ts`
  - `apps/availability-node/src/server.ts`
  - `apps/availability-node/test/hyperdrive-store.test.js`
  - `apps/availability-node/test/publication-idempotency.test.js`
- **Excluded (unchanged from the ledger's existing Exclusions list):**
  deployment descriptors/environment loading, listener bind, certificate
  provisioning/rotation, the Rust **mTLS client** that calls the
  Availability Node (`P2.T4c`, still `Planned`), PostgreSQL/outbox, CK/KEK
  handling, invite/viewer/device state, `P2P_READY`, the rest of `P2.T4`,
  and `P2.T3d`'s full cross-boundary certification.
- **Candidate leaves** (independently scoreable/verifiable; both required
  for the parent to close, honest-Low-band-maximization pass applied):
  - **Leaf A — Rust package materializer** (`crates/p2p/src/package_writer.rs`,
    `lib.rs`, `tests/package_writer_test.rs`): pure-ish deterministic logic +
    filesystem I/O, no network, no external service. Verification:
    `cargo test -p dubbridge-p2p --all-features`, `cargo fmt --check`,
    `cargo clippy -p dubbridge-p2p -D warnings`.
  - **Leaf B — Availability Node persistent store** (existing candidate
    envelope: `hyperdrive_store.ts`, `server.ts`, package.json/lock, its two
    test files): Corestore/Hyperdrive/Hyperswarm integration, D3-D5 Node-side
    behavior. Verification: the npm/node commands already frozen in the
    ledger's "Verification commands to freeze in the task packet" section.
  - The two leaves do not share writable paths and can be implemented and
    reviewed independently; the **parent** T3c governs the integrated
    acceptance criteria (1-6) and behavioral examples (HP-T3c-1/2,
    EC-T3c-1/2), since criterion 1 ("first valid package... opened...
    returns exact 201") and HP-T3c-1/2 require Leaf A's output to exist for
    Leaf B's acceptance tests to exercise a real package end-to-end. A
    fixture-built package (bypassing Leaf A, using a hand-constructed
    ciphertext directory matching the same contract) is an acceptable
    interim proxy for Leaf B's own leaf-level tests, but the **parent
    integration** step must run Leaf A's real output through Leaf B before
    T3c can close.
- **Parent integration:** after both leaves pass their own verification,
  run Leaf A's materializer against a real `SealedPackage` (from
  `crates/p2p/src/package_builder.rs`) to produce a package under the
  shared root, then exercise Leaf B's full HP/EC suite against that real
  package (not only the fixture), to close acceptance criterion 1 and
  HP-T3c-1 genuinely end-to-end. This integration check is parent-owned,
  not delegable to either leaf alone.

## RRI

- Parent RRI: **70 — Complex (56-70)** — `scripts/rri.py`, ADR-045 v2
  authority: `C=2 T=3 A=1 X=3 D=3 K=3 P=3`, technical profile
  `L=2 I=3 Q=3 V=3 -> B=3 -> ICI=75 -> band ceiling 70`; risk-band input 22;
  no penalties. See `.agent/p2-t3c/parent-rri.md`.
- Leaf A (Rust materializer): **70 — Complex** —
  `C=1 T=2 A=1 X=1 D=3 K=3 P=3`, `B=3 -> ICI=75 -> 70`. See
  `.agent/p2-t3c/leaf-a-rri.md`.
- Leaf B (Availability Node store): **70 — Complex** —
  `C=2 T=3 A=1 X=2 D=3 K=3 P=3`, `B=3 -> ICI=75 -> 70`. See
  `.agent/p2-t3c/leaf-b-rri.md`.
- **D/K/P justification:** `crates/p2p` and the Availability Node's
  persistence layer have no direct anchor-rubric row. D/K/P=3 was judged by
  analogy to the rubric's `crates/db`/`crates/storage`/`crates/ingestion`
  tier (floor 3, ADR-006/018) because both leaves perform durable
  filesystem writes with path-containment/traversal/symlink-escape defense
  guarding a ciphertext-only boundary that ADR-044 treats as a hard
  authority line — not the `crates/auth`/session tier (floor 4), since
  neither leaf touches authentication, authorization, or session/token
  material.
- **Honest Low-band maximization — result: no further split possible.**
  Both the parent and each independently-scoped leaf land at RRI 70
  (Complex) because `D=3`/`K=3` combined with `T>=2` already saturates the
  ADR-045 technical-profile bottleneck (`B=max(C,K,D,T)`, capped at 4) at
  `B=3` regardless of file count or CC. Splitting further (e.g. one file
  per leaf) would not lower D/K/P — the bottleneck is the *kind* of work
  (durable cross-boundary containment logic on a security-anchored
  boundary), not its size — and would fragment one coherent
  containment/atomicity/idempotency design across unverifiable fragments,
  which `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Honest Low-band
  maximization explicitly forbids ("do not split one invariant across
  unverifiable fragments"). Residual is recorded at its real band per that
  section's `honest-low-max: residual` convention.
- **Consequence:** RRI >= 56 makes this task **Complex band**, which
  requires: (a) **mandatory decomposition before implementation** — already
  satisfied by the Leaf A / Leaf B split, since the band gate is "decompose
  and score every leaf," not "reach Low," and the leaves are independently
  scored and individually implementable; (b) **human reviews the plan**
  before any implementation starts, at minimum — this orchestration pass
  stops at the approval card, with no leaf implementation attempted; (c)
  the cross-vendor peer replaces Gemma/GPT-OSS for both phase-1 and
  phase-2 review (see § Band-routed peer review); (d) 4 Reflection passes
  once implementation is approved and begins.

## Resultado

- **Estado:** ALCANCE CONGELADO
- **Motivo:** D1-D5 resolved with direct repository/contract evidence (no
  invented layout); D2's predecessor requirement is satisfied by expanding
  T3c's own envelope with a new Rust leaf in the existing `crates/p2p`
  crate, per explicit orchestrator instruction, rather than opening a
  separate predecessor task. Scope, leaves, and parent-integration
  obligation are frozen; RRI, phase-1 review, and the approval card follow
  in this same orchestration pass.

Task-analysis review: d14 `.agent/p2-t3c/phase1-review.json` - PASS (same-provider-degraded,
gpt-oss:20b via Ollama; cross-vendor peer excluded by explicit user
instruction to use only the local stack, 2026-09-12; 1 MINOR finding,
disposed as accepted-with-clarification).

Code-solution review: n/a - preflight/scope-freeze only; no source change in
this document.

## Supplementary sub-decomposition pass (2026-09-12, post-freeze)

Requested by the owner after the frozen Leaf A/Leaf B scope above: (1)
attempt to lower the Med-high-eligible portions of the two leaves toward
Moderate or Low; (2) re-analyze the parent-integration step (`T3c-Integ`) on
its own merits and confirm whether its cost can be lowered. This section
records only new candidate sub-leaves and findings; it does not replace or
re-freeze the Leaf A/Leaf B envelope above, which remains the authoritative
scope for T3c as a whole.

**Method constraint:** every score below was produced by direct
`scripts/rri.py` invocation this session (not estimated or recalled), per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Communication format ("Hand-
estimated scores and remembered rules are untrusted — re-derive from the
tool or file"). Phase-1 review for this task used only the local stack
(`gpt-oss:20b` via Ollama, D14 same-provider-degraded) per explicit owner
instruction this session to exclude Codex; see `.agent/p2-t3c/phase1-review.json`.

### Candidate Low-band extractions (point 1)

| Candidate | Scope | RRI | Band | Verified |
|---|---|---|---|---|
| **T3c-S0** | `crates/p2p/src/path.rs`: extend existing `normalize_path` with realpath + per-segment lstat symlink-escape rejection | 25 | Low | `--cc 5 --D 1 --K 1 --P 1 --T 1 --A 0 --X 0` |
| **T3c-S3** | `apps/availability-node/src/containment.ts`: mirror the same containment check for the Node side | 25 | Low | `--cc 5 --D 1 --K 1 --P 1 --T 1 --A 0 --X 0` |
| **T3c-S1a** | `crates/p2p/src/atomic_write.rs`: new generic tmp-file + rename primitive, no P2P-specific semantics | 25 | Low | `--cc 4 --D 1 --K 0 --P 1 --T 1 --A 0 --X 0` |
| **T3c-S2a** | `apps/availability-node/src/publication_record.ts`: pure encode/decode of the durable-index record shape, no filesystem/network I/O | 25 | Low | `--cc 4 --D 1 --K 0 --P 1 --T 1 --A 0 --X 0` |

D justified at floor 1 (not 0) for all four: each still interprets a
domain-specific shape (a P2P path-containment rule, a durable-index record
schema) even though it performs no cross-boundary I/O or durable decision
itself — a purely mechanical D=0 utility (e.g. a string trim helper) was
judged not to fit any of these.

### Residual Med-high/Complex composition pieces

| Candidate | Scope | RRI | Band | Verified |
|---|---|---|---|---|
| **T3c-S1b** | `crates/p2p/src/package_writer.rs` (+ `lib.rs` export + test file): composes S0 (path safety) + S1a (atomic write) into the actual materializer, deciding P2P package/manifest write semantics | 55 | Med-high | `--cc 8 --D 2 --K 1 --P 2 --T 2 --A 1 --X 1` |
| **T3c-S2b** | `apps/availability-node/src/publication_index.ts`: composes S2a (record shape) + filesystem persistence into the durable `(publication_id, lineage_id)` index | 55 | Med-high | `--cc 8 --D 2 --K 1 --P 2 --T 2 --A 1 --X 1` |
| **T3c-S4** | `apps/availability-node/src/hyperdrive_store.ts` + `server.ts` (+ both test files): Corestore/Hyperdrive/Hyperswarm open/announce/join/flush lifecycle, concurrency serialization, and the `201/200/409/422/503` response fan-out | **70** | **Complex** | `--cc 11 --D 3 --K 3 --P 3 --T 3 --A 1 --X 2`, reconfirmed at `--D 2 --K 2 --P 2` (still 70) |

**S4 correction (self-correction, same session):** an earlier verbal
estimate presented to the owner in chat placed S4 at RRI 55 Med-high. Direct
script verification found this wrong — S4 reproduces at **RRI 70 Complex**
regardless of D/K/P (tested both the original D/K/P=3 and a lowered D/K/P=2;
both land at 70). The driver is not domain/coupling/impact judgment but raw
cyclomatic complexity: CC=11 across the open/announce/join/flush lifecycle
and the five-way status-code fan-out pushes `L=2`, which alone saturates the
ADR-045 bottleneck `B=max(C,K,D,T)` at band 3 (`ICI=75 -> ceiling 70`). This
is the same structural mechanism already documented above for the frozen
parent/leaf scores: once the fixed `ICI_BAND_CEILING` lookup crosses a
band boundary, no D/K/P adjustment recovers a lower result. S4 is the
genuine Complex-band residue of Leaf B after extracting S2a/S2b/S3; it
cannot be honestly presented as Med-high.

**Net result for point 1:** four new genuine Low-band leaves extracted
(S0, S1a, S2a, S3). The domain-composing remainder splits into two Med-high
pieces (S1b, S2b) and one Complex piece (S4) that was previously folded
into Leaf B's single RRI-70 score. This is a real reduction in Med-high/
Complex *surface* (less code left in the highest bands) but not a reduction
in the *band* of the hardest remaining piece — S4 is exactly as hard as
before, now merely isolated rather than blended with lower-complexity
neighbors.

### T3c-Integ re-analysis (point 2)

An isolated component score for the parent-integration/verification step
(`--touches crates/p2p/tests/integration_test.rs --touches
apps/availability-node/test/e2e-integration.test.js --cc 3 --D 2 --K 2 --P 2
--T 2 --A 0 --X 2`) computes **RRI 55 Med-high** on its own merits. This
number is real but **does not change T3c-Integ's governance path**, per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Honest Low-band maximization
before presentation (lines ~520-524): *"When the parent envelope scores RRI
26+, its one approval checkpoint remains mandatory before any contained Low
leaf starts. Approval covers the frozen leaf set but does not change the
parent's band-resolved phase reviews or final unified/integrated
verification."* T3c-Integ **is** the parent's final unified/integrated
verification step — it is where acceptance criteria 1-6 are proven
end-to-end across both leaves, per the "Parent integration obligation" in
§ Alcance congelado above — so it is the specific object that rule
protects, not a candidate for independent re-routing. It also fails the
guide's § Honest Low-band maximization step 6 stop condition
("stop when another split would cease to be independently meaningful or
verifiable"): integration cannot be verified in isolation from the leaves it
integrates, so scoring it as a detachable Med-high unit and routing it
through a lighter review chain would fragment the containment/idempotency
invariant the parent RRI-70 score already protects.

**Conclusion for point 2:** T3c-Integ's cost cannot be lowered in the sense
that matters (review chain, approval gate, Reflection count). It remains
bound to the parent's Complex band: cross-vendor-peer-or-D14 phase-1/phase-2
review, the mandatory RRI 56+ human plan approval already recorded above,
and 4 Reflection passes once implementation begins.

### Updated candidate leaf table (supersedes the two-leaf table for
implementation planning; does not change the frozen parent RRI)

| Leaf | RRI | Band | Depends on |
|---|---|---|---|
| T3c-S0 | 25 | Low | — |
| T3c-S3 | 25 | Low | — |
| T3c-S1a | 25 | Low | — |
| T3c-S2a | 25 | Low | — |
| T3c-S1b | 55 | Med-high | S0, S1a |
| T3c-S2b | 55 | Med-high | S2a, S3 |
| T3c-S4 | 70 | Complex | S1b, S2b, S3 |
| T3c-Integ | 70 (inherited, not independently routable) | Complex | all of the above |

None of these subtasks are approved for implementation. This pass only
refines the decomposition already frozen under "Alcance congelado"; the
parent approval requirement, band-resolved review chain, and 4-pass
Reflection gate stated in § RRI above remain unchanged and still govern
T3c as a whole.

Task-analysis review (this section): d14 `.agent/p2-t3c/phase1-review.json`
covers the frozen Leaf A/Leaf B envelope this section refines; no separate
phase-1 packet was built for the eight-leaf table since none of these
leaves are yet approved or being delegated. A dedicated phase-1 pass is
required for each leaf's own delegation packet before it is sent, per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Per-task discipline, at the time
implementation is actually approved.

### T3c-S0 — implemented and closed (2026-09-12)

Owner-approved for implementation the same day as this preflight
("aprobado para que trabajes en S0"), scoped to this one leaf only —
S1a/S2a/S3 remain unapproved. Delivered `verify_contained_realpath` plus
`PathError::SymlinkEscape` in `crates/p2p/src/path.rs`, additive to
`normalize_path`. Full implementation routing, the two rejected Qwen
delegation attempts (duplicate-test-module defect; a destructive full-file
rewrite), the manual mechanical merge, an orchestrator-diagnosed
nonexistent-ancestor canonicalize bug, and a four-step phase-2 review
resource-recovery/fallback chain ending in a D14 isolated review that found
and drove the fix for a genuine dangling-symlink detection gap are recorded
in `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` §
"P2.T3c-S0 — symlink-escape containment check — DONE". Final state: 12/12
`path::` unit tests passing, `fmt`/`clippy` clean, full `dubbridge-p2p`
crate green. This closure does not change T3c's own Complex-band RRI,
approval gate, or the still-unapproved status of Leaf A/Leaf B or the other
seven leaves in the table above.
