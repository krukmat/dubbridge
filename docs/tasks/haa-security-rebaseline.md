---
type: TaskList
title: HAA security rebaseline tasks
status: proposed
plan: docs/plan/haa-security-rebaseline.md
---

# S1–S4 task ledger

**Follow-up owner decision (2026-11-24):** the RRI calculation requires reevaluation and is suspended until further notice for development of this HAA ↔ DubBridge feature. Do not recalculate or automatically reinstate it; preserve historical scores. Resumption requires an explicit owner decision after reevaluation. This does not suspend runtime HITL/authorization rules or technical security gates.


Historical parent RRI 100 Very high (E2 evidence). Owner explicitly waived
RRI as a development gate for this feature on 2026-11-24; no rescoring required.
Confidence is a planning estimate for artifact completion only.
DONE below means documented design artifact, not certified code.

| Leaf | Deliverable / acceptance | Owner | Depends | Confidence | State |
|---|---|---|---|---|---|
| S1.1 | Threat model, privilege boundary, exclusions | ChatGPT | E1 | 96% | DONE design |
| S1.2 | ADR alternatives and selected protected service topology | ChatGPT | S1.1 | 93% | PROPOSED |
| S1.3 | Define protected code, storage, credentials, owner and worker identities | ChatGPT | S1.2 | 92% | DONE design |
| S1.4 | Define bounded IPC and protected operation/target contract | ChatGPT | S1.3 | 90% | DONE design |
| S2.1 | Volatile lease + restart-generation invariant | ChatGPT | S1.2 | 95% | DONE design |
| S2.2 | Durable request/consume/closure state and no replay | ChatGPT | S2.1 | 93% | DONE design |
| S2.3 | Legacy ACTIVE records non-authorizing; no automatic migration | ChatGPT | S2.1 | 96% | DONE design |
| S3.1 | Preserve RRI 100 and demonstrate domain-floor issue | ChatGPT | E2 | 98% | DONE analysis |
| S3.2 | Split old W3/W4 code into reviewable leaves and map gates | ChatGPT | S3.1 | 92% | DONE decomposition |
| S3.3 | Record bounded feature RRI exception and preserve historical evidence | ChatGPT | Owner directive | 99% | DONE (owner waiver) |
| S3.4 | Implement bounded security leaves with acceptance criteria and regression tests | ChatGPT | S1.2 ratified; S3.3 | Unmeasured | PENDING architecture ratification |
| S4.1 | Negative-path matrix and exact-head evidence protocol | ChatGPT | S1–S3 design | 95% | DONE design |
| S4.2 | Privileged provisioning of protected service/worker on Mac | Owner + local | S1.2 ratified; S3.4 | Unmeasured | BLOCKED |
| S4.3 | Consolidated host isolation/rollback/runner/regression gate | Local + ChatGPT | S4.2 | Unmeasured | BLOCKED |
| S4.4 | E4 integrated ceremony and E5 evidence decision | Owner + local + ChatGPT | S4.3 PASS | Unmeasured | BLOCKED |

## Decomposition of existing W3/W4

Each line is a separate candidate leaf with bounded behavior and acceptance
criteria, not automatic execution approval. No feature-development RRI scoring
required. Characterize behavior before changing it; obtain architecture
ratification before authority-changing code. Earlier candidate code remains inactive.

| Original parent | Bounded analysis / verification leaf | Separate authority-changing leaves |
|---|---|---|
| T3.2 | Decision-table fixtures, unknown/negative input cases | Validate scope/risk; classify routine vs protected action; reject invalid contexts |
| T3.3 | State-transition and duplicate-session fixtures | Protected state-store API; atomic uniqueness; request-intent journal; binding validation |
| T3.4 | Exact card/session/Git test vectors | Protected source lookup; live snapshot; trusted pin validation; consume-intent; detached signature/binding; post-check |
| T3.5 | Lease schema/TTL/legacy fixtures | Volatile registry; unique grant admission; service generation; expiry/revocation; audit projection |
| T3.6 | Path/capability test matrix; characterize CC 35 function | Git identity/ancestry checks; card binding; actual operation scope; guard-and-effect serialization |
| T3.7 | Crash/rollback/uncertain-state fixtures | Startup invalidation; terminal journal recovery; no-retry reconciliation; report projection |
| T3.8 | Direct/normalized CLI and worker bypass fixtures | Protected install; typed IPC; preflight; brokered model/edit/test/commit; worker lifecycle; protected final target |
| W4 | Manifest and drift/closure fixtures | Trusted canonical plan reader; consume-intent; detached grant verification; atomic canonical close; projection/recovery |

## Acceptance for S3.3

DONE by explicit owner direction. The feature-scoped exception is recorded
in the workflow guide, RRI policy and this ledger. Prior RRI evidence is
not altered. Runtime authorization, actual isolation and regression gates
remain intact.

No owner-host task is delegated now. No production code, credential, role,
branch, merge or activation changes under this design checkpoint.

## Revised S1–S4 execution order — owner approved 2026-11-24

The earlier table remains historical. ADR-049 accepted; development RRI
suspended, independent review waived, no CI. The revised plan has 15 remote
implementation tasks and 5 local/final certification tasks. No host changes
or enforcement activation are authorized by this checkpoint.

## Implementation checkpoint — S1–S3

| Revised task | Status | Evidence / remaining gate |
|---|---|---|
| S1.1 ADR-049 ratification | DONE | Owner-approved ADR |
| S1.2 protected executor contract | DONE candidate | protected_host, protected_session |
| S1.3 macOS provisioning | PARTIAL | deployment runbook and DAC preflight; no install automation or host proof |
| S1.4 IPC and credentials | PARTIAL | bounded kernel-peer IPC, private file checks; actual HAA credential provisioning pending |
| S2.1 volatile leases | IMPLEMENTED | volatile_authority |
| S2.2 restart invalidation | IMPLEMENTED | generation + empty live registry |
| S2.3 protected admission journal | IMPLEMENTED | explicit SQLite initialization and unique tombstones |
| S2.4 replay/concurrency | IMPLEMENTED candidate | journal uniqueness, service lock, fresh request |
| S2.5 negative tests | AUTHORED | tests not run at candidate HEAD |
| S3.1 runner integration | PARTIAL | worker_runner calls real runner; only single existing file proposal, not production wiring |
| S3.2 guards | PARTIAL | broker patch guard; trusted commit/format/test provenance missing |
| S3.3 canonical closure | PARTIAL | protected_plan atomic state/closure; real plan ingestion/projection not wired |
| S3.4 recovery | IMPLEMENTED candidate | volatile restart and terminal intent; no automatic retry |
| S3.5 integration tests | AUTHORED | synthetic fixtures, execution pending |
| S3.6 certification script | PREPARED | certify_candidate.py includes new suite; no physical/privileged installation automated |

S4 remains gated by regression results and host isolation. NO-GO activation.
No RRI recalculation, no reviewer, no CI, no branch or merge. Do not treat
a candidate gate as proof of non-bypassability.

## S3.7 continuation — 2026-11-24

See `docs/audit/haa-s37-integrated-candidate-2026-11-24.md`. Bounded commit, task intent, pinned plan
initialization, independent closure and signed integration fixture prepared.
S3.7b remains partial (actual isolated verification backend unavailable);
S3.7e suite authored, full execution pending. Code is not activated.
Next gate S4.0 is nonprivileged exact-checkout regression; no Touch ID.

## S4.0 regression evidence — 2026-11-24

Owner-reported exact-checkout PASS at `1574d75eccdba4a6b42d86fc7f1a9c4d311b3685`: 124 tests, 123 pass,
1 skip, zero failures, 10.955 s; Python 3.15.2 / cryptography 50.0.2.
See `docs/audit/haa-s40-regression-2026-11-24.md`. Candidate suite passed; host isolation and real
verification backend remain untested/uncertified. Activation NO_GO.

## S4.0 follow-up

Baseline 1574d75: 123 PASS, one skip. S3.7b exact-tree export and observed-exit
adapter prepared; true identity launcher still unavailable. Local host inspection
must establish account/group separation and trusted installation constraints
before the privileged provisioning plan is finalized. No activation.

## S4.0 expanded PASS / S4.1 inventory

Owner-reported 131 tests (130 pass, one skip), 11.607 s, clean HEAD
`3de677fd3f662a2f65343c47c49829a0d2fec08b`. See `docs/audit/haa-s41-host-inventory-2026-11-24.md`. Only UID 501 detected, admin member;
current Python owned by UID 501. Dedicated service/worker/verifier and
protected runtime must be provisioned. No real identity proof or activation.

## S4.1 deployment candidate checkpoint

The owner-reported regression at 3de677f remains PASS for that exact code
only. MacOSVerifierLauncher, non-root helper and four new mocked launcher
regressions authored at fd212d2db7882645bd6e345d4e8dc4d5f918de03; these new tests are NOT RUN.
See docs/playbooks/HAA_MACOS_IDENTITY_LAUNCH.md for the narrow non-root
identity transition and the proposed protected layout. UID/group allocation,
root-owned interpreter/dependency provenance, actual sudo/ACL policy and
process/network isolation still require a single consolidated local
provisioning intervention. Do not infer authorization to install generic
sudo rules or execute repository code as root. No actual host changes.
S3.7b implementation candidate prepared; certification remains open.
Activation NO_GO, no new Touch ID, no RRI/CI/independent reviewer.


## Owner-approved official Python runtime (supersedes custom runtime path)

Owner reports installation of the official python.org macOS distribution.
Use that installation instead of downloading/building a standalone runtime
or adapting HAA to the Apple CLT Python 3.9. No extra runtime copy is needed.
Require a supported Python >=3.11 (3.14 preferred), arm64 support and exact
local version/provenance verification before use. Expected framework:
`/Library/Frameworks/Python.framework/Versions/<major.minor>/`.
Resolve the actual executable locally; do not assume a particular version,
use Current, /usr/local/bin, Homebrew, a mutable wrapper or a symlink as
the trusted executable. Verify the canonical executable is a regular file,
root-owned, with protected ancestor directories, framework, stdlib,
dependency/import paths and ACLs. Check installer receipt, applicable
signature validation and record exact version and hashes of the runtime
and dependency manifest. An official package alone is not isolation proof.
Do not use user site packages or writable PYTHONPATH. Runtime/dependency
drift requires a new regression and host certificate before activation.

Reconcile the fixed interpreter check in macos_verifier_launcher.py,
the verify_launch.py shebang and its trusted executable search paths to
the actual verified canonical installation path. Make these small
changes in the existing branch and update corresponding fixtures.
Do not create runtime/bin/python3 symlinks or wrappers; the direct
canonical framework path replaces that old layout requirement.
Use a separately protected, pinned dependency environment; resolve
cryptography according to scripts/haa/requirements.txt (currently
>=45,<46), do not silently reuse the owner environment's 50.0.2.
Prepare verified wheels unprivileged; install only reviewed artifacts
administratively, no pip or repository scripts as root. Run the full
candidate suite under the selected runtime/dependencies, then resume
the approved synthetic G1/G2/G3 gates. If any provenance or isolation
condition is unmet, BLOCKED, no fallback to CLT 3.9.

All previous security restrictions remain: no actual HAA credentials,
Touch ID, enforcement, broad sudo, new branches, CI or development RRI.
No repeat runtime download, no compatibility rewrite, no additional
reviewer. The user's local G0 report at 38534bc was 134 PASS, one skip
(135 tests), but did not certify the newly installed official interpreter.
