---
type: TaskList
title: "Tasks: P5 Local HLS playback through the existing player"
status: complete
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p5-local-playback.md
behavioral_coverage_contract: behavior-v2
---

# P5 — planning task ledger

**Status:** **Closed 2026-09-26 — release evidence transferred.** T0/T1/T2
are formally closed; preparation head `e63209f5` completed 15/15 CI, mobile
revalidation is 62/62 suites and 446/446 tests, and owner verification is
complete. P5.T3 no longer remains an independent open task: P7.T2 owns the
physical checklist and P7/T9g retain the certification gate.
**Phase gate:** **P4 PASS — satisfied 2026-09-22.**
**Effort:** provisional per work package below; no new RRI record is fabricated by the automated-evidence remediation.

**Closure amendment — 2026-09-26:** P5-DEV (T0-T2) is complete and P5.T3 is
closed as a standalone record. Its physical controls are now P7.T2 acceptance
criteria, satisfied by compatible exact-RC T7p evidence only when P7.T2
verifies artifact identity. P7.T3/T9g cannot certify/GO without passing those
controls.

**Re-scope 2026-09-26:** P5.T3 is relativized to a **general test** — the
checklist in `docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` — executed inside
the physical tests (S-230-T7p on the exact RC, else P7.T2). It has no dedicated
device run, emulator loop or local-topology work of its own. P5 is closed; no
certification PASS is inferred. Decision:
`docs/audit/mvp0-p2p-p5-t3-sequencing-replan-2026-09-22.md` § Amendment 2026-09-26.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P5.T0 | Gateway/session contract freeze | planning | M | P4 PASS | `[x]` Done 2026-09-18 |
| P5.T1 | Loopback ciphertext decryption gateway | development | L | T0 PASS | **PASS / Done 2026-09-22** — HP/EC evidence PASS; 15/15 CI; owner-verified |
| P5.T2 | Existing VideoPlayer and deterministic teardown | development | M | T1 PASS | **PASS / Done 2026-09-22** — HP/EC evidence PASS; 15/15 CI; owner-verified |
| P5.T3 | Playback and secret-boundary certification | transferred release checklist | M | T2 PASS | **Closed 2026-09-26** — exact-RC evidence is owned by T7p/P7.T2; no P5-CERT PASS claimed |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P5 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p5-local-playback.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P5.T0 — Gateway/session contract freeze

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P4 PASS

**Status:** `[x]` Done — retro-certified 2026-09-18 under owner waiver D0-a.
RRI 25 Low (`scripts/rri.py`,
`mobile/src/p2p/playback/{P2PPlaybackLease,P2PPlaybackController}.ts`).

**Reflection:** Low band, cycle folded into review evidence below.

**Behavioral coverage certification:** HP-P5.T0-1 and EC-P5.T0-1 map to
passing `unit` evidence — 6/6 tests; the tested authorization branch
(wrong-viewer) is covered.

**Named residual (owner-accepted):** other `assertAuthorization` OR-branches
(asset/publication/lineage mismatch, expiry) are implemented but not
individually exercised by a test; no standalone contract/decision artifact
exists separate from the code. Non-blocking — same class as `P2.T4e-cov`.

**Owner final verification:** Matias, 2026-09-18 — closed on "cierra los
otros", accepting the named residual as described in
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.
Commands run: `npm test -- mobile/src/p2p/playback`.

**Acceptance criteria:** Define loopback/session ownership, package path mapping, native unwrap → transient CK handoff, expiry/error handling and teardown evidence; freeze executable paths.

- **HP-P5.T0-1:** An authorized verified package maps to a scoped local playback session.
- **EC-P5.T0-1:** Unverified package or unavailable current authorization has no valid startup transition.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P4 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T1 — Loopback ciphertext decryption gateway

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** **PASS / Done 2026-09-22.** Automated evidence PASS was established 2026-09-18, revalidated by mobile (62/62 suites, 446/446 tests), preparation head `e63209f5` reached 15/15 CI PASS, and owner verification is complete. The previous blocker is
resolved by executable coverage of the actual `ProductPlaybackRuntime`,
including a real OS loopback TCP-listener proof plus component coverage for
session-token scoping, traversal denial, ciphertext/AAD tamper denial,
missing-key denial, deterministic teardown, and CK zeroization on stop/start
failure. Production P5 runtime code was unchanged. Evidence:
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

**Acceptance criteria:** Serve only verified package HLS through loopback; use accepted K1 authenticated decryption and transient authorized CK; validate relative package paths and scope local requests to the session.

- **HP-P5.T1-1:** Verified manifest and segments decrypt at serve time for the authorized session.
- **EC-P5.T1-1:** Traversal, foreign session, altered ciphertext/AAD or missing key denies delivery without remote media fallback.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T2 — Existing VideoPlayer and deterministic teardown

**Type:** development

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** **Closure-ready 2026-09-22; owner verification pending.** Automated evidence PASS was established 2026-09-18 and revalidated by the current mobile gate (62/62 suites, 446/446 tests). The prior transitive blocker
is removed: T1 now proves runtime CK zeroization/listener/package teardown,
while the existing lease tests prove idempotent release and teardown-before-
retry. Additional controller tests now cover asset/publication/lineage/viewer/
expiry authorization mismatches before K1 unwrap/playback startup. Evidence:
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

**Acceptance criteria:** Connect existing player to scoped loopback URL; release transient CK and gateway on stop/sign-out/error according to frozen lifecycle; preserve existing review playback.

- **HP-P5.T2-1:** Video plays end to end; stop closes listener/session and releases the key reference.
- **EC-P5.T2-1:** Gateway/native unwrap failure prevents playback; no HTTP/S3 URL substitution and no plaintext key in disk/logs.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T3 — Playback and secret-boundary certification

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** **Closed 2026-09-26 — transferred to T7p/P7.T2.** P5.T3 is no
longer an independently executable or tracked certification task. Its complete
checklist is owned by the exact-RC physical run in T7p or, if not fully covered
there, P7.T2. The 2026-09-26 emulator attempts produced no
SYNC/VERIFY/PLAYBACK evidence and do not count toward the checklist. They are
historical observations only: `docs/audit/mvp0-p2p-p5-t3-postfix-diagnostic-2026-09-26.md`.
The client discovery repair `P5.T3-r1` is committed (`3f3ffbd`) but is not
device-validated; the physical run is its first proven Android validation.
Its recovered RRI 70 record, reproduced RED and focused 34/34 GREEN evidence
are in `docs/audit/p5-t3-r1-discovery-repair-evidence-2026-09-26.md`.
The standing MVP0-P2P owner review exception applies to this repair; the
evidence record is closed without claiming Android behavioral certification.

**Owner confirmation 2026-09-26:** local-development diagnosis is discarded.
Do not schedule emulator retries, Colima networking repairs or a host-side
seeder. Retain existing diagnostic evidence; certification belongs solely to
the physical T7p/P7.T2 release run.

The previously confirmed P4
filesystem defect is now repaired by P4.T1-r1: the scoped `file:` URI is converted
exactly once to a filesystem path at the Corestore boundary. CI evidence is green
(61/61 mobile suites, 441/441 tests; focused RED→GREEN 7/7 current-source PASS;
committed Bare worklet drift check PASS). Full implementation evidence:
`docs/audit/p4-t1-r1-storage-uri-fix-evidence-2026-09-22.md`.

The required physical evidence is the checklist in
`docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` § Required evidence (SYNC →
VERIFY → PLAYBACK plus the secret/no-fallback, teardown, tamper and review-path
controls), recorded on a physical Android device by the T7p/P7.T2 run against the
exact RC. CI alone never satisfies it, and an emulator run never does.

**Registro histórico (superado) —** BLOCKED at the CLAIM stage of the happy-path attempt. Root
cause is a local-development-environment gap, not a P3/P4/P5 code defect: the
mobile certification harness's default gateway URL
(`mobile/app.config.ts` → `http://10.0.2.2:8081`) matches neither the local
Docker Compose `api` container (8080) nor any gateway service (Compose has none),
and `apps/api` does not itself serve the `/api/*`-prefixed paths the mobile
client calls (only the gateway, which strips that prefix, does). A standalone
`dubbridge-gateway` process happened to already be running outside Compose on
port 8082 in this environment; pointing the harness at it via
`EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` required a full rebuild, which invalidated
the persisted auth session, and no viewer credentials were available to log back
in. Stopped per task rules rather than fabricating credentials or modifying code.
No PASS evidence exists yet. Full attempt record, root-cause trace, and follow-up
recommendations: `docs/audit/mvp0-p2p-p5-t3-android-certification-blocked-2026-09-22.md`.
T1/T2 formal closure still separately pending owner verification/governance sync.

**Acceptance criteria:** Prove package playback, tamper denial, teardown and review-path non-regression with executable evidence and network capture; hand off play capability/state to P6.

- **HP-P5.T3-1:** A complete short video plays from local verified ciphertext using the existing player.
- **EC-P5.T3-1:** Disabling remote audience media still permits the valid local path; key or gateway failure stops it outright.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Handoff:** P7.T2 is the sole evidence producer. It must use the exact RC and
complete the existing certification checklist. A missing or failed control is a
P7 `NOT_CERTIFIED` result, not a reopened P5.T3 task.


## P5-DEV milestone closure — 2026-09-22

**P5 closed 2026-09-26.** T0-T2 are formally closed and P5.T3 is transferred
to T7p/P7.T2. P7/T9g retain the open release-certification decision.
