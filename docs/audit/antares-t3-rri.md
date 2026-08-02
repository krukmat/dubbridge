---
type: Audit
title: "RRI evidence: Antares T3 CWE watchlist and context-complete packet construction"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3---cwe-watchlist-and-context-complete-packet-construction
date: 2026-08-01
---

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | `--auto-cc` fallback (score=0): no local `.rs` files in `--touches`; clippy skipped — Low confidence, consistent with every prior `scripts/antares/` task in this series | Low |
| F files | 3 | `--touches` -> 6 planned files | High |
| D domain | 3 | agent-supplied (no anchor-rubric match) — read-only packet/context construction, no process execution; scored like T2a/T2b (parser/policy), not T2c (D=4, real sandboxed execution) | High |
| T coverage | 2 | agent-supplied — no tests exist yet, but fixtures/context-closure tests are named as the deliverable itself; partial confidence because real-repo dependency traversal has more surface than fixed-input composition | Medium |
| A ambiguity | 2 | agent-supplied — acceptance criteria exist in the ledger, but the watchlist file location/format, the concrete size-budget number, and the deterministic partition algorithm are all undefined | High |
| K coupling | 3 | agent-supplied (no anchor-rubric match) — filesystem/repo-structure traversal across crates and docs, matches the "filesystem, external service, or framework integration" rubric row | High |
| P impact | 4 | agent-supplied (no anchor-rubric match) — hard requirement to exclude credentials/`.env`/`config/production.toml`/out-of-snapshot paths from a packet; a defect here is a data-visibility/secret-exposure risk, matching the "data visibility" rubric row | High |
| X context | 4 | agent-supplied — needs the Rust workspace's crate/module graph plus Python worker/docs structure held simultaneously with the packet contract | Medium |

**Base value:** 100 x (weighted / 5) = 51
**Penalties applied:** `arch_decision` (+12, manual — this task originates a new watchlist schema, packet schema, and context-closure/partition algorithm, none of which are pinned down anywhere yet); `no_verification` (+15, manual — no diff exists yet)
**Final RRI: 78 -> band High (71-85) -> Effort XL. Codex Premium. Claude Premium. thinking On**
**Gates for this band:** Characterization tests + explicit acceptance criteria + human reviews the diff (not just the plan). Decomposition remains mandatory.
**Decomposition:** triggered by RRI >= 56 (also independently by RRI > 70) — split before implementing.

Command run:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/cwe_watchlist.py \
  --touches scripts/antares/cwe_watchlist_test.py \
  --touches scripts/antares/context_closure.py \
  --touches scripts/antares/context_closure_test.py \
  --touches scripts/antares/context_packet.py \
  --touches scripts/antares/context_packet_test.py \
  --auto-cc \
  --D 3 --K 3 --P 4 \
  --T 2 --A 2 --X 4 \
  --penalty arch_decision --penalty no_verification
```

## Self-challenge (Socratic doubt)

D was first drafted at 4 by analogy with T2c's sandboxed-execution score, then
challenged and lowered to 3: T3 performs no process execution, so the closer
precedent is T2a/T2b (parsing/policy, both D=3), not T2c. Re-running the
script at D=4 still landed in the same High band (RRI 81 vs. 78) — the
`arch_decision` + `no_verification` penalties (+27 combined) dominate the
band outcome, not the D judgment call. The band conclusion is robust to this
particular uncertainty; the exact final number is not, and will be
recomputed per subtask after decomposition.

## Notes

- This is a conservative pre-execution score built from a representative
  planned file set (watchlist module + packet/context-closure module, each
  with tests). The real touched-file set may differ once subtasks are scoped.
- Consistent with the T2 -> T2a..T2e precedent in this same ledger, RRI >= 56
  triggers mandatory decomposition before any implementation. See the
  decomposition record in `docs/tasks/antares-security-specialist-advisor.md`
  § T3 for the proposed subtask split; each subtask must recompute its own
  RRI from its narrower scope before being individually presented for
  approval.
