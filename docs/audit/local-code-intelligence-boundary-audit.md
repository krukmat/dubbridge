---
type: Audit
title: "Local Code Intelligence Boundary — Audit Entry Point"
branch: feature/local-code-intelligence-boundary
status: m4_hardening_ready_for_local_audit
---

# Local Code Intelligence Boundary — Audit Entry Point

This is the single navigation point for auditing the code-intelligence boundary on `feature/local-code-intelligence-boundary`.

Repository governance remains authoritative. This document is test/navigation guidance only.

## Scope

M1–M3 are closed by project decision. The M3 operational entry point is:

`scripts/code_intelligence/context_gateway.py`

M4 does not reopen model routing, RRI, or Analyze architecture. It hardens the existing Analyze/handoff CLI path with:

1. graph-to-repository freshness enforcement;
2. defense-in-depth cloud export minimization;
3. bounded context expansion linked to a prior receipt;
4. operational use-and-adjust after the hardening is verified.

The code-intelligence layer remains development tooling, not DubBridge product runtime.

## Read order

1. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
2. `README_AGENT_ORDER.md`
3. this file
4. `docs/plan/local-code-intelligence-boundary.md`
5. `docs/tasks/local-code-intelligence-boundary.md`
6. `docs/plan/local-code-intelligence-m4-operational-adoption.md`
7. `docs/tasks/local-code-intelligence-m4-operational-adoption.md`
8. `scripts/code_intelligence/README.md`
9. `docs/schemas/context-receipt-v1.schema.json`
10. implementation files under `scripts/code_intelligence/`

Do not bulk-load unrelated ADRs/product documentation.

## Architectural invariants

- Source, tests, ADRs, and policies remain authoritative over graph-derived context.
- Cloud agents consume bounded artifacts; they never receive unrestricted graph traversal.
- A concrete CKG/backend remains local and replaceable.
- The executable core is model/vendor agnostic.
- `secret` and `runtime_data` are denied for every target.
- Cloud additionally denies `cross_boundary` and `global_architecture` data.
- Cloud metadata is minimized from allowed task-local evidence rather than copied wholesale.
- Clearly unsafe paths/content are denied by deterministic gateway rules even when mislabeled by the backend.
- Initial and expanded context must be bound to the expected Git revision.
- Expansion must preserve the base receipt target, Git revision, graph revision, and receipt hash chain.

## Expected branch scope

Expected changes are limited to code-intelligence tooling/docs plus narrowly justified repository test fixes already documented on the branch.

Unexpected without explicit justification:

```text
apps/**
crates/**
mobile/**
infra/**
scripts/rri.py
docs/policies/RRI_POLICY.md
existing model-routing implementation
```

## S0 — Branch scope

```bash
git switch feature/local-code-intelligence-boundary
BASE="$(git merge-base origin/main HEAD)"
git diff --name-only "$BASE"...HEAD

git diff --exit-code "$BASE"...HEAD -- \
  scripts/rri.py \
  docs/policies/RRI_POLICY.md \
  apps crates mobile infra
```

Expected: no product-runtime/RRI diff.

## S1 — Syntax and unit suite

```bash
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py

python3 scripts/code_intelligence/context_gateway_test.py
```

Expected: all tests `OK`.

## S2 — Fresh cloud happy path

The reusable fixture carries a deterministic synthetic Git revision, so use that exact revision for fixture-only smoke tests:

```bash
FIXTURE_SHA="0000000000000000000000000000000000000000"
rm -rf /tmp/dubbridge-cki-cloud
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S2 \
  --task "Audit cloud context filtering" \
  --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
  --target cloud \
  --expected-git-revision "$FIXTURE_SHA" \
  --output-dir /tmp/dubbridge-cki-cloud
```

Inspect:

```bash
python3 - <<'PY'
import json
from pathlib import Path
c = json.loads(Path('/tmp/dubbridge-cki-cloud/context-capsule.json').read_text())
r = json.loads(Path('/tmp/dubbridge-cki-cloud/context-receipt.json').read_text())
serialized = json.dumps(c)
assert c['target'] == 'cloud'
assert [x['classification'] for x in c['source_fragments']] == ['task_local']
assert 'crates/auth/src/lib.rs' not in c['files']
assert 'auth::internal::session_topology' not in c['symbols']
assert 'sensitive audit fixture value' not in serialized
assert 'ephemeral audit fixture value' not in serialized
assert len(r['receipt_sha256']) == 64
print('S2 PASS')
PY
```

## S3 — Stale graph fails closed

```bash
rm -rf /tmp/dubbridge-cki-stale
set +e
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S3 \
  --task "Audit stale graph rejection" \
  --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
  --target cloud \
  --expected-git-revision deadbeef \
  --output-dir /tmp/dubbridge-cki-stale
RC=$?
set -e

test "$RC" -ne 0
test ! -e /tmp/dubbridge-cki-stale/context-receipt.json
test ! -e /tmp/dubbridge-cki-stale/context-capsule.json
```

Expected: non-zero exit, expected/received revision in stderr, no success artifacts.

For real operational use the orchestrator should pass `$(git rev-parse HEAD)`, not a synthetic fixture value.

## S4 — Local remains richer but secret-safe

```bash
rm -rf /tmp/dubbridge-cki-local
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S4 \
  --task "Audit local policy" \
  --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
  --target local \
  --expected-git-revision "$FIXTURE_SHA" \
  --output-dir /tmp/dubbridge-cki-local
```

Expected: local keeps allowed cross-boundary/global structural context but excludes explicit secret/runtime material and unsafe runtime paths.

## S5 — Determinism and hash integrity

Run the same verified input twice and compare both JSON files with `cmp`. Then recompute `capsule_sha256` and the unsigned receipt SHA-256 using canonical sorted compact JSON. Expected: byte-identical output and matching hashes.

## S6 — Defense-in-depth misclassification probe

Copy the fixture and change a sensitive fragment to:

```json
{
  "path": ".env.local",
  "content": "GITHUB_TOKEN=should-not-export",
  "classification": "task_local"
}
```

Preserve valid line fields, run the cloud gateway with the matching fixture revision, and assert neither `.env.local` nor `should-not-export` appears in the capsule.

Expected: gateway deny rules override the backend's unsafe `task_local` label.

## S7 — Bounded expansion

Generate a base receipt, then locally produce another backend JSON result with one additional allowed task-local fragment/symbol under the same Git and graph revisions.

Run:

```bash
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S7 \
  --task "Audit bounded expansion" \
  --backend-json /tmp/dubbridge-expanded-graph.json \
  --target cloud \
  --expected-git-revision "$FIXTURE_SHA" \
  --base-receipt /tmp/dubbridge-cki-cloud/context-receipt.json \
  --expansion-reason "Need one adjacent task-local helper" \
  --output-dir /tmp/dubbridge-cki-expanded
```

Expected:

- base receipt hash verifies;
- target/Git/graph revisions match;
- only policy-allowed new context is exposed;
- new receipt contains an expansion record with base receipt SHA, reason, decision, and delta.

## S8 — Expansion cannot bypass policy

Repeat S7 with only global-architecture/cross-boundary/unsafe additional context.

Expected: capsule does not expose the requested forbidden context; expansion decision is `deny` or `reduce` according to whether any allowed delta remains.

Change `graph_revision` or Git revision relative to the base receipt and repeat.

Expected: fail closed before publishing success artifacts.

## S9 — Model/vendor independence

```bash
if grep -RniE \
  'nemotron|qwen|gemma|muse|claude|codex|ollama|codebase-memory|codegraph' \
  scripts/code_intelligence/*.py; then
  echo 'S9 FAIL: model/vendor coupling found in executable core'
  exit 1
fi

echo 'S9 PASS'
```

Documentation may name consumers/candidate producers; executable core must not depend on them.

## S10 — Repository documentation QA

```bash
make qa-docs
```

Expected: pass.

## Deferred probe — pair-level artifact atomicity

Each JSON file is individually replaced atomically. The receipt/capsule pair is not one filesystem transaction.

Do not add transaction/manifest machinery unless M4 operational use exposes a real asynchronous consumer race or crash-recovery issue. If one occurs, create a narrowly scoped M4-T5 task with reproducible evidence.

## M4 operational-use review

After S0–S10 pass, normal DubBridge tasks should use the existing Analyze/handoff path. Record only actionable friction:

- stale graph race;
- missing relevant context;
- irrelevant context;
- policy over-blocking;
- policy under-blocking;
- host resource regression;
- receipt/capsule consumer race.

A recurring/material issue becomes a separate evidence-backed M4-T5 task. Do not create synthetic benchmark work or a KPI dashboard.

## Audit verdict format

Record:

```text
Branch/head reviewed
Merge base reviewed
S0-S10 results
Findings by severity
Required fixes before operational use
Deferred T5 candidates
ADR recommendation: required / amend existing / not required yet
Verdict: PASS / PASS WITH CONDITIONS / BLOCK
```

The key invariant remains:

> Cloud agents consume bounded context artifacts; they do not explore the DubBridge graph directly.
