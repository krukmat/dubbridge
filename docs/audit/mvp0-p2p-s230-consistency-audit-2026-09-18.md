---
type: Audit
title: "MVP0-P2P / S-230 doc-vs-code consistency audit — 2026-09-18"
status: open
---

# MVP0-P2P / S-230 consistency audit — 2026-09-18

Scope: read-only independent audit, report-only per owner instruction. No
source, runtime configuration, architecture decision, release approval,
commit, or deployment is authorized by this record. Remediation tasks derived
from this audit live in
`docs/tasks/mvp0-p2p-s230-consistency-remediation.md`.

Commit range reviewed: last week starting Monday 2026-09-14 through
2026-09-18 (`feature/p2p-mvp-core`, HEAD `c6e28be` at audit time). The user's
linked Claude Docs artifact (`claude.ai/artifact/TDXzep84uGbtUB9akoKk7U`) was
not accessible (access-denied) and is not reflected here.

## Verification performed

- `cargo check --workspace --all-features` — pass.
- `cargo test -p dubbridge-p2p --all-features` — 59/59 pass.
- Availability Node `npm run typecheck` — pass.
- Availability Node `node --test` — **82/86 pass, 4 failing** (see Finding 1).
- Mobile Jest P2P suites — 179/179 pass, 37 suites.
- Worker-runner DB-backed P2P integration tests — 19/19 pass (after
  correcting an env-var name on my side, `DUBBRIDGE_DATABASE_URL` not
  `DATABASE_URL`; not a real defect).
- `make qa-docs` — pass (structural/frontmatter/index checks only; does not
  catch ledger-status-vs-code semantic drift).

## Finding 1 (Alta) — P2.T3c materializer path regression

Commit `8eb2f05` (2026-09-15, "fix(p2p): materialize canonical package
lineage path") changed the Rust materializer
(`crates/p2p/src/package_writer.rs`) to write to
`packages/<publication_id>/<lineage_id>` without updating the Node-side
`package_verification.ts` / T4d dispatcher to match. 4/86 Availability Node
tests fail with `package_invalid`, all in the T3c-Integ certification suite:

- `HP-T3c-1`, `HP-T3c-2`, `EC-T3c-1a`, `EC-T3c-1b` — all in
  `apps/availability-node/test/package-publication-integration.test.js`.

P2.T3c is marked `[x] Done (2026-09-13)` in
`docs/tasks/mvp0-p2p-p2-encrypted-publication.md` but does not hold at
current HEAD. Remediation: CONS-T1/T2.

## Finding 2 (Media) — P2.T3d wrong-scope landing

`docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § P2.T3d documents that
`publication-contract-certification.test.js` landed (commits `f27ff75`/
`b35c71c`, 2026-09-14) instead of the two frozen files
`publication-contract.test.js` + `fixtures.js`. The landed file only covers 4
pure unit parsing tests, missing the mTLS/HTTP/replay/conflict/containment/
secret-boundary acceptance criteria (HP-T3d-1/2, EC-T3d-1/2/3). The ledger
itself states: "Do not report T3d as advanced or closed on the basis of this
file." Remediation: CONS-T3.

## Finding 3 (Media) — T5/T6 ledger stale vs. implemented code

Confirmed via `docs/audit/mvp0-p2p-t5-t6-local-dev-readiness-2026-09-14.md`:
P2.T5a-d and T6a-e are `Planned` in the ledger and in `roadmap.md:402`
("11 leaves with no code started"), but 61 files / +5761/-297 lines / 4
migrations (0034-0037) are implemented at HEAD, confirmed by direct file
existence checks (`p2p_activation.rs`, `p2p_ready_descriptor.rs`,
`p2p_s120_non_regression_test.rs`, `p2p_crash_windows_test.rs`,
`p2p_secret_boundary_test.rs`, `p2p_availability.rs`,
`p2p_publication_job.rs`, `p2p_publication_claim_repo.rs`). No RRI/approval-
card evidence, no `docs/audit/gemma-evidence/` artifact, no Reflection log /
behavioral coverage cert / owner verification block exists for T5/T6.
Precedent for remediation: the P2.T4b-T4f retrospective closure record
(commit `6a6d0c7`, 2026-09-14). Remediation: CONS-T4.

Migration numbering also drifted from what was originally frozen: T6a's
ledger entry names "0035" for audit correlation, but the actual file at that
purpose is `0036` (0035 went to ready-descriptor-evidence instead). Migration
files 0032-0038 all exist on disk. Flag for D0-a ratification.

## Finding 4 (Media) — ADR-026 config gap

`crates/config/src/*.rs` has zero matches for "p2p" (case-insensitive
grep). `apps/worker-runner/src/p2p_publication_runtime.rs:21-45` reads 9
`DUBBRIDGE_P2P_*` env vars directly via `std::env::var`, bypassing the typed
fail-closed loader — confirmed independently by
`docs/audit/mvp0-p2p-t5-t6-local-dev-readiness-2026-09-14.md` Blocker D. None
of the 9 vars are documented in `.env.example`, `config/local.toml`, or
`config/README.md`. Remediation: D0-b, CONS-T9.

## Finding 5 (Media) — ADR-031-adjacent deviation blocking T7local

`infra/local/docker-compose.yml` has no `gateway` service (postgres, redis,
minio, minio-init, api, availability-node, worker-runner, and the three
Python workers only). T7local's own definition
(`docs/tasks/s-230-poc-v1-digitalocean.md`, T7local) requires "build pointed
at the gateway exposed by the local compose stack." Commit `c6e28be`
("fix(local): expose API port for P2P certification") instead directly
exposed the API port — a deviation from ADR-031's transparent-relay model
(gateway should mediate, not be bypassed). Remediation: D0-c, CONS-T11.

## Finding 6 (Baja) — duplicate/orphaned P7Local plan doc

`docs/plan/mvp0-p2p-p7local-certification.md` (added 2026-09-17, commit
`0c5720b`, no guión, no corresponding task ledger) duplicates
`docs/plan/mvp0-p2p-p7-local-certification.md` (has a ledger:
`docs/tasks/mvp0-p2p-p7-local-certification.md`, already Planned with T0-T3
defined). Neither `docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`,
nor `docs/plan/roadmap.md` reference "P7Local"/"P7L" at all (0 grep matches
in any). Remediation: CONS-T0 (annotate only; the ledger-bearing doc already
exists and needs no recreation — see CONS-T13).

## Finding 7 (Baja) — no P6 mobile UI yet

`mobile/src/screens/` and `mobile/src/navigation/` have no "MyContent"/
"Invites" matches. Only backend read-models exist
(`crates/db/src/p2p_dashboard_repo.rs`, `apps/api/src/routes/p2p_dashboard.rs`,
`mobile/src/p2p/P2PDashboardService.ts`). Confirmed against
`docs/audit/mvp0-p2p-p6-t0-preflight-2026-09-17.md`, which explicitly states
P6.T0 is blocked until P3-P5 are PASS and defines GAP-1/GAP-2 as the two
contract decisions P6.T0 itself must resolve before P6.T1 UI work starts.
Remediation: CONS-T10a/T10c.

## Related

- `docs/tasks/mvp0-p2p-s230-consistency-remediation.md` — remediation task map
- `docs/audit/mvp0-p2p-t5-t6-local-dev-readiness-2026-09-14.md`
- `docs/audit/mvp0-p2p-p6-t0-preflight-2026-09-17.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/s-230-poc-v1-digitalocean.md`
- `docs/plan/roadmap.md` § MVP0-P2P
