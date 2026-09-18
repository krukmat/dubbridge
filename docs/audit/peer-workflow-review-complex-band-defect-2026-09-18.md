---
type: Audit
title: "scripts/peer-workflow-review.py — Complex-band (RRI 56+) reviewer-order defect"
status: complete
---

# `scripts/peer-workflow-review.py` — Complex-band reviewer-order defect

**Found:** 2026-09-18, while preparing P2.T3d's (CONS-T3) phase-1 review.
Not fixed here — out of scope for CONS-T3/T3d, logged for its own future
task per the repo's out-of-scope-discovery discipline.

## Defect

The script's own docstring (lines 10-12) correctly states the RRI 56+
contract: `gpt-oss:20b Complex -> cross-vendor fallback -> D14`, matching
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`'s
2026-09-13 owner-directed rebinding (`gpt-oss:20b` at the Complex profile is
primary; the cross-vendor peer, e.g. Codex, is fallback only).

`main()` does not implement this for the Complex band. At line 734:

```python
if cross_vendor:
    packet = _build_peer_packet(args.phase, content, args.task_id)
    result = run_cross_vendor_review(packet, args.phase, peer)
    ...
```

`cross_vendor = needs_cross_vendor(args.rri)` is `True` for any RRI >= 56
(`CROSS_VENDOR_MIN_RRI = 56`), and `peer = resolve_peer(args.caller)`
resolves directly to the cross-vendor model (`codex` for
`caller=claude-code`) with **no attempt at `gpt-oss:20b` first**. The
correct gpt-oss-primary-then-fallback sequence (`_run_gpt_oss_review` then
`_run_gemma_fallback` then D14) exists in the code (visible at the bottom
of `main()`, lines 754+) but is gated behind the **Low-band** branch only
(the final `# Low band (RRI 0-25)` comment confirms it was written for that
band, not reused for Complex).

**Reproduction:** dry-run against any RRI >= 56 packet always reports
`reviewer=codex` (or `claude` depending on caller), never `gpt-oss:20b`:

```
$ python3 scripts/peer-workflow-review.py --phase task --rri 70 \
    --caller claude-code --content <any file> --dry-run
[peer-review] rri=70 band=Complex phase=task reviewer=codex caller=claude-code
```

## Impact

Any orchestrator that trusts this script's resolved reviewer for RRI 56+
work skips `gpt-oss:20b` (Complex profile) entirely and goes straight to
the cross-vendor peer — inverting the documented primary/fallback order.
This does not by itself invalidate a resulting review (Codex is still a
legitimate reviewer in the chain), but it silently burns the cross-vendor
fallback as if it were the primary, and never exercises or records a
`gpt-oss:20b`-unavailable condition, undermining the audit trail's
"GPT-OSS 20B fallback: triggered | not triggered — reason" field required
by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Report line contract`.

## Disposition

Not fixed in this session — discovered while executing CONS-T3 (P2.T3d)
analysis, unrelated to that task's own scope
(`apps/availability-node/test/*`). Logged per the out-of-scope-discovery
discipline used elsewhere in this remediation effort (see
`docs/tasks/mvp0-p2p-s230-consistency-remediation.md` § CONS-T2's
`user_account`/mobile-Jest/coverage-gap findings for precedent). Worked
around for T3d's own phase-1/phase-2 review by invoking `gpt-oss:20b`
directly at the Complex profile
(`num_ctx=49152`, `num_predict=10240`, `think=medium`, `temperature=1.0`,
`top_p=1.0`) per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory
workflow before implementing` Step 0, rather than trusting the script's
resolved route.

**Recommended fix (not applied):** in `main()`, when `cross_vendor` is
`True`, first attempt `_run_gpt_oss_review` at the Complex profile
(`num_ctx=49152`), falling back to `run_cross_vendor_review` only on
gpt-oss failure/BLOCKED, mirroring the existing Low-band sequence's
structure but with the Complex profile's context/predict/think values and
cross-vendor (not Gemma) as the second fallback. RRI for this fix: likely
Low (isolated to `peer-workflow-review.py`'s `main()`, no schema/behavior
change beyond call order) — should be scored properly, not assumed, before
implementation.
