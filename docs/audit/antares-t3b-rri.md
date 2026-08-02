---
type: Audit
title: "RRI evidence: T3b - Packet schema and hard security-exclusion guarantees"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3b---packet-schema-and-hard-security-exclusion-guarantees
date: 2026-08-02
---

# RRI evidence: T3b — Packet schema and hard security-exclusion guarantees

Task: `docs/tasks/antares-security-specialist-advisor.md` § T3b
Depends on: T3a (`[x] Done (owner-verified, 2026-08-02)`), decomposition of T3 (proposed 2026-08-01)

## Presentation-time computation (2026-08-02, pre-implementation)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/packet_schema.py \
  --touches scripts/antares/packet_schema_test.py \
  --auto-cc \
  --D 2 --K 1 --P 3 \
  --T 1 --A 1 --X 1 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped | Low |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 2 | agent-supplied — no exact anchor-rubric row (`scripts/antares/` is outside `crates/**`); scored one step above "internal crate business logic" floor semantics because the task defines a security-exclusion data contract (credentials/`.env`/`config/production.toml`/out-of-snapshot paths), not plain business logic | Medium |
| T coverage | 1 | agent-supplied — schema + exclusion + size-budget fixtures are the deliverable itself | High |
| A ambiguity | 1 | agent-supplied — scope is narrow (data contract + exclusion guarantees + size-budget partition over an explicit, already-given file list); no repo-traversal algorithm (that is T3c) | High |
| K coupling | 1 | agent-supplied — reuses `scripts/antares/path_containment.py` (T2b) for canonical-path checks rather than re-implementing traversal; no new process/network side effects | High |
| P impact | 3 | agent-supplied — anchor rubric's explicit secrets/auth floor (P=5) applies to touching `crates/auth`/credential-storage/secrets infrastructure itself, which this task does not; scored above T3a's P=2 because a defect here (a missed exclusion pattern) could let a credential or production-config value leak into a packet forwarded to a model, which is a real but narrower failure mode than owning the secrets boundary itself | Medium |
| X context | 1 | agent-supplied — one self-contained module plus its already-existing dependency (`path_containment.py`) | High |

**Base value:** 100 x (weighted / 5) = 27
**Penalties applied:** `no_verification` (+15, manual flag — no diff exists yet)
**auth_security auto-detection:** not triggered — anchor-rubric P floor for `scripts/antares/**` is unmatched (outside `crates/auth`/rights-ledger/secrets-storage paths); the penalty is reserved for touching the auth/secrets system itself, not for a script that defensively filters secrets out of a downstream artifact. Recorded explicitly per the Socratic-doubt/no-hallucination communication rule rather than silently omitted.
**Final RRI: 42 -> band Med-high (41-55) -> Effort L. Codex Balanced->Premium. Claude Balanced->Premium. thinking On**
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Implementation route:** ADR-038 Architect-refined single-attempt gate — Qwen27 (`qwen3.6:27b-q4_K_M`) advisory refinement -> primary hash-bound route receipt (downgrade-only) -> if `GO_LOCAL`, one bounded `qwen3.6:35b-a3b` session (<=8 turns, <=300s, 0 repair attempts) supervised by `scripts/local-agent/run_med_high_task.py`; otherwise Codex/Claude with the full ADR-038 §5 evidence bundle.
**Decomposition:** not triggered — within split target (RRI <= 55, A=1).

## Notes

- This subtask is the second of the T3 -> T3a..T3d split proposed in
  `docs/tasks/antares-security-specialist-advisor.md` § T3 decomposition
  record. It depends on T3a (watchlist) only conceptually (the packet
  carries a CWE drawn from the watchlist) and has no dependency on the
  repository context-closure algorithm (T3c) or the touchpoint integration
  (T3d) — this task defines the packet **data contract** and its exclusion
  guarantees before the closure algorithm that populates it exists.
- D/P landed one band above T3a's (D=1/P=2) because T3a is pure static data
  validation with no filesystem interaction, while T3b's acceptance criteria
  (EC-2/EC-3 below) require correctly excluding live filesystem paths
  (credentials, `.env`, `config/production.toml`, out-of-snapshot paths) from
  a packet — a defect here has a direct secret-exposure failure mode, even
  though the task does not touch the auth/secrets system itself.
- Phase-1/phase-2 reviewer for this band (owner directive 2026-07-21):
  `qwen3.6:27b-q4_K_M`, falling back to Gemma then D14 if unavailable.
- Hard ADR-038 §6 GO_LOCAL exclusions do not categorically bar this task
  (it is not itself auth/security infrastructure, a schema migration, or a
  release cut), but its security-exclusion guarantees are exactly the kind
  of judgment call the Qwen27 advisory refinement step should weigh
  explicitly before recommending `GO_LOCAL` vs `CLOUD_REQUIRED`.

## Post-implementation computation (2026-08-02, execution)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/packet_schema.py \
  --touches scripts/antares/packet_schema_test.py \
  --auto-cc \
  --D 2 --K 1 --P 3 \
  --T 1 --A 1 --X 1
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped | Low |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 2 | agent-supplied — unchanged from presentation; the task still defines a security-exclusion packet contract rather than a plain data-holder | High |
| T coverage | 1 | agent-supplied — 8 packet-schema fixture tests now exist, covering HP-1/HP-2/EC-1..EC-4 | High |
| A ambiguity | 1 | agent-supplied — scope stayed narrow; no repository context-closure logic was added | High |
| K coupling | 1 | agent-supplied — still limited to `cwe_watchlist.py` provenance and `path_containment.py` canonicalization reuse | High |
| P impact | 3 | agent-supplied — unchanged; a defect could still leak sensitive material into a model packet even though this task does not own the secrets system itself | High |
| X context | 1 | agent-supplied — one module + tests + existing containment/watchlist helpers | High |

**Base value:** 100 x (weighted / 5) = 27
**Penalties applied:** none
**Final RRI: 27 -> band Moderate (26-40) -> Effort M. Codex Balanced. Claude Balanced. thinking Off**
**Execution note:** the task was still executed under its already-approved pre-execution Med-high presentation (`42`, driven by the `no_verification` penalty), so the stronger approval/review discipline remained in force even though the delivered implementation lands lower after verification evidence exists.
