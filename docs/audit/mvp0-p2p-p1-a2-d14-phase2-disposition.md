---
type: Audit
status: complete
---

# P1.A2 phase-2 code-solution review — D14 disposition

## Routing

- Task-analysis review (phase 1) already recorded PASS:
  `docs/audit/mvp0-p2p-p1-a2-phase1-review.json`.
- Code-solution review (phase 2), RRI 46 Med-high (26-55 chain: Gemma
  primary -> Muse Glimmer intermediate -> D14 final).
- **Gemma** (`gemma4:26b-a4b-it-qat`): 0/3 parseable passes, both the
  initial attempt and the required one retry — every pass returned a bare
  `STATUS: FINDINGS` value without the required `SUMMARY` header
  (`invalid review response: missing SUMMARY header`).
- **Muse Glimmer** (`muse-glimmer:30b-q4_K_M`) intermediate fallback: 0/3
  passes on first attempt — idle timeout after 180s without a token on
  every pass. Resource-recovery protocol invoked: `ollama stop`,
  `GET /api/ps` showed the model loaded at `context_length: 131072` with
  host free memory at ~64MB (`vm_stat`/`memory_pressure`), a clear capacity
  symptom. Reduced-profile warm-test (`num_ctx=16384`, `think=false`,
  `temperature=0`, `num_predict=512`) passed, but the real review packet
  (~17k estimated tokens) does not fit the reduced context budget
  (~15.5k tokens available for prompt after overhead/predict reservation)
  — rebuilding a meaningfully-trimmed packet would degrade review coverage
  silently, so no bounded retry was attempted at the reduced profile.
- **D14 triggered** per both local reviewers being unavailable/unusable.
  Cross-provider route used (caller=claude-code -> reviewer=codex), per
  policy preference for a different provider than the primary orchestrator.
  Codex CLI (`codex-cli 0.151.0-alpha.7.2`, authenticated via ChatGPT
  login) ran `codex exec --sandbox read-only --skip-git-repo-check` against
  the isolated adjudicator packet (diff + acceptance criteria + verified
  command output + empty reconciled_findings, per
  `scripts/adjudicator-packet.py` allowlist contract — no development
  transcript or chain-of-thought crossed the boundary).

## D14 verdict: FINDINGS

Full raw output: `docs/audit/mvp0-p2p-p1-a2-d14-phase2-review.log`.

3 BLOCKING, 2 MAJOR findings. Each independently verified against the
repository (not accepted on citation alone) before disposition:

| # | Severity | Claim | Verification | Disposition |
|---|---|---|---|---|
| 1 | BLOCKING | `isWithinProofRoot` exists but is dead code; worklet-side `proofStorageUri` in `transient-drive.ts` doesn't re-validate the host-supplied URI. Cited file/line (`transient-drive.ts:40`) was actually `closeStoreOrDrive`, not `proofStorageUri` (line 47) — a citation error in the *location*, but the substantive claim (validator unused) was confirmed true by grep: `isWithinProofRoot` had exactly one call site, its own unit test. | Confirmed: `grep -rn isWithinProofRoot` showed zero production call sites before the fix. | **repaired** — wired into `P1SeedProofRunner.runSeedProof` as a pre-flight guard on the host-constructed run root before the worklet starts. |
| 2 | BLOCKING | `writeHashSeed`'s error-path cleanup discards a `closeSeedHandles` failure via `.catch(() => undefined)`, violating the task's own "cleanup failure makes the proof fail" acceptance criterion. | Confirmed by direct read of `transient-seed.ts:100` (pre-fix). | **repaired** — cleanup failure now propagates as `SEED_CLOSE_FAILED`, matching the established pattern already used in `transient-drive.ts`'s `openCloseTransientDrive`. |
| 3 | BLOCKING | Failed/abandoned runs are not cleaned up: the proof runner only closes the RPC port on failure (never deletes the run directory), and the janitor (`listAbandonedProofRuns`) only lists — nothing ever calls `deleteProofRunDirectory` on the result. | Confirmed: `listAbandonedProofRuns` had no caller; no janitor entry point existed. | **repaired** — added `janitorAbandonedProofRuns(maxAgeMs, now?)` to `P1SeedProofRunner.ts`, calling `deleteProofRunDirectory` for each abandoned run, tolerating an already-removed run (`TRANSIENT_STORAGE_NOT_FOUND`) but failing closed on any other deletion error. |
| 4 | MAJOR | No end-to-end test proves shutdown -> exact-run deletion -> absence verification, nor cleanup-after-failure / foreign-path-rejection / stale-run-deletion behavior. | Confirmed: `runSeedProof` (the host orchestrator that actually performs this sequence) had zero test coverage; no test file referenced it. | **repaired** — new `mobile/__tests__/p2p/P1SeedProofRunner.test.ts` (7 tests): HP-A2 full lifecycle, EC-A2 traversal rejection pre-worklet-start, EC-A2 delete failure surfacing, EC-A2 port-close-on-failure, plus 3 janitor tests (deletes only stale, tolerates already-gone, fails closed on delete error). |
| 5 | MAJOR | Diff exceeds the ledger's literal `allowed_paths` list (adds `runtime-client.ts`; touches `transient-drive.ts`, `BareRuntimeClient.ts`, build tooling, shared test utilities). | Confirmed via `git diff --name-only`. Root cause: this session's user-directed further split of `protocol.ts` (to pass `qa-maintainability`'s declaration-line budget) required extracting `BareRpcPort`/`RuntimeProtocolClient` into a new `runtime-client.ts`, with mechanical import-site updates in every consumer. | **accepted-follow-up, not repaired** — this is real drift from the ledger's literal path list, but it is mechanical refactor scope (no new behavior), independently reviewed by its own maintainability-gate pass, fully covered by the existing/expanded test suite, and reverting it would reintroduce the `qa-maintainability` violation this session was explicitly directed to fix. Recorded here rather than silently accepted; the task ledger's `allowed_paths` should be corrected in the same closure pass to reflect the actual file set for audit accuracy. |

## Independently verified evidence (post-fix)

- `npm run typecheck` (mobile/): exit 0
- `npm run lint` (mobile/): exit 0
- `npx jest` (mobile/, full suite): 29/29 suites, 295/295 tests passed
- `npx jest __tests__/p2p/`: 8/8 suites, 60/60 tests passed (+7 new vs. pre-fix)
- `node scripts/build-bare-worklet.mjs --check`: no drift, sha256=`32390ea97d9c17f37b97b0b478b19dc70e0498c5d06dc8ad135d10d4e2f5b1ef`
- `python3 scripts/check-maintainability.py`: Maintainability gate passed

## Primary-agent disposition

3/3 BLOCKING findings repaired with code changes; 2/2 MAJOR findings
disposed — 1 repaired (missing test), 1 accepted-follow-up with reason
recorded (scope drift from a user-directed mechanical refactor already
covered by tests and its own gate).

- disposition_divergence: `n/a` — no reconciliation step preceded this
  review (both local reviewers returned zero usable passes, so D14 ran a
  from-scratch review rather than adjudicating prior findings).
