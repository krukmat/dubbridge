---
type: Audit
title: "Local Code Intelligence Boundary — Audit Entry Point"
branch: feature/local-code-intelligence-boundary
status: ready_for_local_audit
---

# Local Code Intelligence Boundary — Audit Entry Point

This document is the **single navigation entry point** for auditing the Local Code Intelligence Boundary introduced on `feature/local-code-intelligence-boundary`.

It is a navigation and test document, **not a higher-authority policy source**. Repository governance remains authoritative.

## 1. Audit objective

Determine whether this branch introduces a safe, model-agnostic local code-intelligence boundary that:

1. normalizes local graph/CKG results through a replaceable backend contract;
2. produces deterministic `Context Receipt` and `Context Capsule` artifacts;
3. gives cloud agents only bounded context rather than unrestricted graph access;
4. prevents explicitly classified `secret` and `runtime_data` content from export;
5. prevents `cross_boundary` and `global_architecture` fragments/relationships from cloud export;
6. remains isolated from DubBridge product runtime and existing model/RRI routing;
7. fails closed on malformed backend input;
8. remains understandable and replaceable without binding the architecture to Nemotron, Qwen, Gemma, Claude, Codex, or a specific CKG vendor.

The audit is **not** expected to prove benchmark improvements or run a formal POC. The intended adoption mode is use-and-adjust.

---

## 2. Read order

### Mandatory orientation

Read these first and in this order:

1. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — highest-authority agent workflow rules.
2. `README_AGENT_ORDER.md` — repository authority/read-order navigation.
3. **this file** — audit scope, entry point, smoke tests, and review questions.
4. `docs/plan/local-code-intelligence-boundary.md` — architectural intent and non-goals.
5. `docs/tasks/local-code-intelligence-boundary.md` — implementation ledger, behavioral cases, and verification state.
6. `scripts/code_intelligence/README.md` — operator-facing trust boundary and CLI usage.
7. `docs/schemas/context-receipt-v1.schema.json` — receipt contract.

### Implementation under review

Read after the documents above:

1. `scripts/code_intelligence/backend.py`
2. `scripts/code_intelligence/context_gateway.py`
3. `scripts/code_intelligence/context_gateway_test.py`
4. `scripts/code_intelligence/fixtures/audit-smoke-graph.json`

### Conditional references

Read only if the corresponding question becomes relevant:

- `docs/policies/RRI_POLICY.md` — verify this branch does not alter RRI semantics or routing.
- `docs/python-exceptions.md` — confirm this Python surface is development tooling, not a product/ML runtime expansion.
- `AGENTS.md` / `CLAUDE.md` — only where the authoritative workflow guide does not already settle agent behavior.
- relevant ADRs — only when a finding touches an existing protected boundary or the auditor concludes this tooling contract itself requires a new ADR.

Avoid bulk-loading unrelated ADRs, plans, or product documentation. One goal of this subsystem is to reduce unnecessary context discovery.

---

## 3. Branch scope expected by the audit

The branch should remain tooling/documentation-only.

Expected changed areas:

```text
docs/audit/local-code-intelligence-boundary-audit.md
docs/plan/local-code-intelligence-boundary.md
docs/tasks/local-code-intelligence-boundary.md
docs/schemas/context-receipt-v1.schema.json
scripts/code_intelligence/**
```

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

### Smoke S0 — scope/no-regression check

From the repository root:

```bash
git switch feature/local-code-intelligence-boundary
BASE="$(git merge-base origin/main HEAD)"
git diff --name-only "$BASE"...HEAD

git diff --exit-code "$BASE"...HEAD -- \
  scripts/rri.py \
  docs/policies/RRI_POLICY.md \
  apps \
  crates \
  mobile \
  infra
```

**Expected:** the first command lists only the intended tooling/docs surface; the second exits `0` with no diff.

If `main` has advanced since branch creation, branch divergence is informational; judge implementation scope from the merge base rather than assuming zero `behind` commits.

---

## 4. Mandatory smoke suite

All mandatory smokes should be run from a local checkout of this branch. They require only repository tooling plus Python 3 unless otherwise stated.

### Smoke S1 — syntax and unit behavioral contract

```bash
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py

python3 scripts/code_intelligence/context_gateway_test.py
```

**Expected:** compilation succeeds and the test runner reports all tests `OK`.

Behavior covered by the current unit suite:

- `HP-1` valid backend payload normalizes;
- `EC-1` missing revision metadata fails closed;
- `HP-2` cloud artifact generation is deterministic and bounded;
- `EC-2` `secret` / `runtime_data` are never exported;
- `EC-3` cloud omits `cross_boundary` / `global_architecture` relationships and source fragments;
- `EC-4` invalid CLI input leaves no successful output artifacts.

---

### Smoke S2 — cloud CLI happy path and filtering

```bash
rm -rf /tmp/dubbridge-cki-cloud
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S2 \
  --task "Audit cloud context filtering" \
  --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
  --target cloud \
  --output-dir /tmp/dubbridge-cki-cloud
```

Then inspect with stdlib Python:

```bash
python3 - <<'PY'
import json
from pathlib import Path

root = Path('/tmp/dubbridge-cki-cloud')
receipt = json.loads((root / 'context-receipt.json').read_text())
capsule = json.loads((root / 'context-capsule.json').read_text())

fragment_classes = {x['classification'] for x in capsule['source_fragments']}
relationship_classes = {x.get('classification', 'task_local') for x in capsule['relationships']}
serialized = json.dumps(capsule)

assert capsule['target'] == 'cloud'
assert fragment_classes == {'task_local'}
assert relationship_classes == {'task_local'}
assert 'sensitive audit fixture value' not in serialized
assert 'ephemeral audit fixture value' not in serialized
assert 'audit smoke cross-boundary fragment' not in serialized
assert 'audit smoke global architecture fragment' not in serialized
assert len(receipt['capsule_sha256']) == 64
assert len(receipt['receipt_sha256']) == 64
print('S2 PASS')
PY
```

**Expected:** `S2 PASS` and both JSON artifacts exist.

The receipt is expected to retain exclusion metadata; the cloud capsule is the bounded payload intended for an external/cloud consumer.

---

### Smoke S3 — local target receives richer structural context but no explicit secrets/runtime data

```bash
rm -rf /tmp/dubbridge-cki-local
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S3 \
  --task "Audit local context policy" \
  --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
  --target local \
  --output-dir /tmp/dubbridge-cki-local

python3 - <<'PY'
import json
from pathlib import Path

capsule = json.loads(Path('/tmp/dubbridge-cki-local/context-capsule.json').read_text())
fragment_classes = {x['classification'] for x in capsule['source_fragments']}
relationship_classes = {x.get('classification', 'task_local') for x in capsule['relationships']}
serialized = json.dumps(capsule)

assert {'task_local', 'cross_boundary', 'global_architecture'} <= fragment_classes
assert {'task_local', 'cross_boundary', 'global_architecture'} <= relationship_classes
assert 'secret' not in fragment_classes
assert 'runtime_data' not in fragment_classes
assert 'sensitive audit fixture value' not in serialized
assert 'ephemeral audit fixture value' not in serialized
print('S3 PASS')
PY
```

**Expected:** `S3 PASS`.

This test demonstrates that the local/cloud distinction is real rather than cosmetic.

---

### Smoke S4 — byte-level determinism

```bash
rm -rf /tmp/dubbridge-cki-det-a /tmp/dubbridge-cki-det-b

for OUT in /tmp/dubbridge-cki-det-a /tmp/dubbridge-cki-det-b; do
  python3 scripts/code_intelligence/context_gateway.py \
    --task-id AUDIT-S4 \
    --task "Audit deterministic artifacts" \
    --backend-json scripts/code_intelligence/fixtures/audit-smoke-graph.json \
    --target cloud \
    --output-dir "$OUT"
done

cmp /tmp/dubbridge-cki-det-a/context-capsule.json \
    /tmp/dubbridge-cki-det-b/context-capsule.json
cmp /tmp/dubbridge-cki-det-a/context-receipt.json \
    /tmp/dubbridge-cki-det-b/context-receipt.json
```

**Expected:** both `cmp` commands exit `0`.

---

### Smoke S5 — receipt/capsule hash integrity

Generate cloud artifacts as in S2, then:

```bash
python3 - <<'PY'
import hashlib
import json
from pathlib import Path

root = Path('/tmp/dubbridge-cki-cloud')
receipt = json.loads((root / 'context-receipt.json').read_text())
capsule = json.loads((root / 'context-capsule.json').read_text())

def digest(value):
    encoded = json.dumps(
        value,
        sort_keys=True,
        separators=(',', ':'),
        ensure_ascii=False,
    ).encode('utf-8')
    return hashlib.sha256(encoded).hexdigest()

assert digest(capsule) == receipt['capsule_sha256']
expected_receipt_hash = receipt.pop('receipt_sha256')
assert digest(receipt) == expected_receipt_hash
print('S5 PASS')
PY
```

**Expected:** `S5 PASS`.

---

### Smoke S6 — malformed graph input fails closed

```bash
python3 - <<'PY'
import json
from pathlib import Path

src = Path('scripts/code_intelligence/fixtures/audit-smoke-graph.json')
payload = json.loads(src.read_text())
payload.pop('graph_revision')
Path('/tmp/dubbridge-cki-invalid.json').write_text(json.dumps(payload))
PY

rm -rf /tmp/dubbridge-cki-invalid-out
set +e
python3 scripts/code_intelligence/context_gateway.py \
  --task-id AUDIT-S6 \
  --task "Audit malformed payload" \
  --backend-json /tmp/dubbridge-cki-invalid.json \
  --target cloud \
  --output-dir /tmp/dubbridge-cki-invalid-out
RC=$?
set -e

test "$RC" -ne 0
test ! -e /tmp/dubbridge-cki-invalid-out/context-receipt.json
test ! -e /tmp/dubbridge-cki-invalid-out/context-capsule.json
```

**Expected:** non-zero gateway exit and no success artifacts.

---

### Smoke S7 — model/vendor independence of the core

```bash
if grep -RniE \
  'nemotron|qwen|gemma|muse|claude|codex|ollama|codebase-memory|codegraph' \
  scripts/code_intelligence/*.py; then
  echo 'S7 FAIL: model/vendor coupling found in core Python'
  exit 1
fi

echo 'S7 PASS'
```

**Expected:** `S7 PASS`.

Documentation may mention candidate backends or agents; the executable core should not depend on one.

---

### Smoke S8 — repository documentation gates

```bash
make qa-docs
```

**Expected:** repository documentation checks pass.

If additional repository-standard Python/script QA is available locally, run it as supplementary evidence.

---

## 5. Exploratory audit probes — intentionally not automatic PASS gates

These probes are important because they test assumptions that the current MVP boundary deliberately keeps simple. A probe exposing current behavior is not automatically a defect; the auditor should decide whether the assumption is acceptable for DubBridge's threat model.

### Probe P1 — stale `git_revision` / graph-to-checkout binding

The fixture deliberately contains a synthetic `git_revision`. Run S2 and compare:

```bash
git rev-parse HEAD
python3 - <<'PY'
import json
from pathlib import Path
r = json.loads(Path('/tmp/dubbridge-cki-cloud/context-receipt.json').read_text())
print(r['repository'])
PY
```

**Current expected behavior:** the gateway records the backend-provided revision but does not independently compare it with the checked-out `HEAD`.

**Audit question:** must the gateway itself reject stale/mismatched graph revisions, or is that responsibility acceptable in the future backend adapter/orchestrator?

If strict stale-graph prevention is required, record this as a concrete hardening task rather than silently assuming the receipt already enforces it.

---

### Probe P2 — classification trust / mislabeled sensitive content

Create a temporary copy of the fixture where the fragment currently classified `secret` is deliberately mislabeled `task_local`, then run the cloud gateway.

**Current expected behavior:** the gateway trusts the backend classification and will treat that fragment as exportable.

**Audit question:** is the local graph adapter trusted enough to own classification, or should the Context Gateway add defense-in-depth classification/path/content rules?

This is one of the most important trust-boundary questions for a future real CKG integration.

---

### Probe P3 — metadata disclosure through unclassified `files` / `symbols`

The current contract classifies relationships and source fragments, but `files`, `symbols`, `tests`, `anchors`, and governance labels are plain arrays.

The supplied fixture intentionally contains auth-related file/symbol metadata. Inspect the cloud capsule:

```bash
python3 - <<'PY'
import json
from pathlib import Path
c = json.loads(Path('/tmp/dubbridge-cki-cloud/context-capsule.json').read_text())
print('files:', c['files'])
print('symbols:', c['symbols'])
print('boundaries:', c['boundaries'])
PY
```

**Current expected behavior:** these metadata fields are exported even when related source fragments/relationships were filtered.

**Audit question:** does DubBridge consider path/symbol/boundary metadata itself sensitive enough to require classification/minimization?

If yes, this should be a follow-up contract change before unrestricted real-repository use with cloud agents.

---

### Probe P4 — pair-level artifact atomicity

Each JSON artifact is individually written through a temporary file plus `os.replace`. The pair is not committed as one filesystem transaction.

**Audit question:** is per-file atomicity sufficient for Analyze/handoff usage, or must consumers require a completion marker/manifest before reading a receipt/capsule pair?

Do not add transaction machinery unless the audit identifies a realistic consumer race or crash-recovery requirement.

---

## 6. Architectural review questions

The auditor should explicitly answer these questions in addition to running the smokes:

1. **Boundary placement:** is `scripts/code_intelligence/` the correct development-tooling layer, with no product-runtime coupling?
2. **Backend neutrality:** can a future local CKG be replaced without changing the agent-facing receipt/capsule contract?
3. **Cloud trust boundary:** is it impossible for a cloud agent to perform arbitrary graph traversal through this interface?
4. **Minimum disclosure:** are the current classification rules sufficient, especially for metadata fields highlighted by P3?
5. **Classifier trust:** is trusting the backend's classification acceptable, or should the gateway independently enforce additional deny rules?
6. **Freshness:** should graph revision / git revision binding become an enforced invariant rather than recorded provenance?
7. **Authority:** is it clear that source, tests, ADRs, and policies remain authoritative over graph-derived context?
8. **RRI independence:** does the branch correctly avoid changing RRI/model routing before real usage provides evidence?
9. **Resource behavior:** can indexing/querying occur before heavy local inference and then become idle, rather than creating another continuously resident heavy component?
10. **ADR need:** does this become sufficiently durable/cross-cutting to warrant a formal ADR now, or is the current plan/tasks/tooling documentation sufficient until backend integration is selected?

---

## 7. Audit evidence expected

A useful audit report should contain:

```text
Branch/head reviewed
Main/merge-base used
Mandatory smoke results S0-S8
Exploratory probe observations P1-P4
Findings by severity
Required fixes before merge
Deferred hardening items
ADR recommendation: required / not required yet
Final verdict: PASS / PASS WITH CONDITIONS / BLOCK
```

For every blocking finding, reference the exact file/function/contract field involved and provide a reproducible command or fixture mutation where possible.

---

## 8. Merge interpretation

A clean audit does **not** mean a CKG backend has been selected or installed. This branch establishes the local intelligence boundary and export contract only.

Backend selection remains a separate follow-up requiring supply-chain, network/telemetry, repository-containment, update-policy, and host-resource review.

The architecture remains:

```text
local CKG/backend
      ↓
backend-neutral adapter
      ↓
Context Gateway
      ├── local bounded/richer capsule
      └── cloud minimum-disclosure capsule
```

The key invariant is:

> Cloud agents consume a bounded context artifact; they do not explore the full DubBridge graph directly.
