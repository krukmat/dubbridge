---
type: Audit
title: "Local Code Intelligence M4 — Pre-T4 Readiness Audit"
branch: feature/local-code-intelligence-boundary
milestone: M4 Operational Adoption
status: executed_ready_for_t4
---

# Local Code Intelligence M4 — Pre-T4 Readiness Audit

## 1. Purpose

This is the **authoritative pre-T4 audit entry point** for the Local Code Intelligence M4 work currently living on `feature/local-code-intelligence-boundary`.

The audit has two responsibilities:

1. certify that M4-T0 through M4-T3 are safe and operationally coherent on the actual branch checkout; and
2. leave an unambiguous handoff for starting **M4-T4 — Operational use-and-adjust** on the next ordinary DubBridge task.

This audit is not a benchmark program and is not a new architecture phase. M1-M3 remain closed by project decision. M4-T4 begins only after this document's mandatory readiness gates are satisfied.

The invariant to preserve is:

> Cloud agents consume bounded context artifacts. They never receive unrestricted repository/graph traversal through the code-intelligence layer.

---

## 2. Authority and read order

Repository governance remains authoritative. Read in this order:

1. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
2. `README_AGENT_ORDER.md`
3. **this document**
4. `docs/plan/local-code-intelligence-m4-operational-adoption.md`
5. `docs/tasks/local-code-intelligence-m4-operational-adoption.md`
6. `scripts/code_intelligence/README.md`
7. `docs/schemas/context-receipt-v1.schema.json`
8. implementation under audit:
   - `scripts/code_intelligence/backend.py`
   - `scripts/code_intelligence/context_gateway.py`
   - `scripts/code_intelligence/context_gateway_test.py`
   - `scripts/code_intelligence/fixtures/audit-smoke-graph.json`

Background only when needed:

- `docs/audit/local-code-intelligence-boundary-audit.md` — M1-M3 boundary audit history and earlier exploratory findings.
- `docs/policies/RRI_POLICY.md` — only to prove M4 did not change RRI semantics/routing.
- `docs/python-exceptions.md` — only to confirm this remains development tooling rather than product runtime.

Do not bulk-load unrelated ADRs or product documentation merely for completeness.

---

## 3. Scope under audit

### M4-T0 — operational baseline

Expected state:

- M3 is closed.
- `scripts/code_intelligence/context_gateway.py` is the Analyze/handoff boundary used by the agent workflow.
- `JsonGraphBackend` remains the stable backend-neutral JSON interchange in this branch.
- no requirement exists to hard-code the gateway into `run_local_task.py` or a specific model runner.

### M4-T1 — freshness

Expected state:

- the operational CLI requires `--expected-git-revision`;
- backend `git_revision` must match that expected revision before artifacts are published;
- bounded expansion must also match the base receipt's Git and graph revisions.

### M4-T2 — minimum disclosure hardening

Expected state:

- cloud metadata is derived from allowed task-local evidence rather than copied wholesale;
- explicit `secret` and `runtime_data` records are denied for every target;
- cloud additionally denies `cross_boundary` and `global_architecture` data;
- deterministic defense-in-depth rejects clearly unsafe paths/content even if a backend mislabels them `task_local`;
- local remains richer than cloud but still excludes secret/runtime material.

### M4-T3 — bounded expansion

Expected state:

- expansion is another local gateway evaluation, not a cloud graph-query capability;
- the base receipt hash verifies;
- target, Git revision, and graph revision match;
- expansion reason and decision (`allow`, `reduce`, `deny`) are recorded;
- cloud deny/minimum-disclosure policy cannot be bypassed by expansion.

---

## 4. T4 readiness prerequisites

T4 **must not start** until all mandatory prerequisites below are satisfied.

| Gate | Requirement | Blocking? |
|---|---|---:|
| R0 | Audit is run from `feature/local-code-intelligence-boundary` at a recorded HEAD | yes |
| R1 | Branch scope shows no M4 changes to `apps/**`, `crates/**`, `mobile/**`, `infra/**`, `scripts/rri.py`, or RRI policy | yes |
| R2 | Python compile + code-intelligence unit suite pass from the branch checkout | yes |
| R3 | Fresh graph succeeds and stale graph fails closed with no success artifacts | yes |
| R4 | Cloud output minimizes unrelated metadata and blocks mislabeled unsafe material | yes |
| R5 | Local output remains richer without exporting secret/runtime material | yes |
| R6 | Receipt/capsule hashes and deterministic generation verify | yes |
| R7 | Bounded expansion allows justified context and cannot bypass deny policy | yes |
| R8 | Stale/different-revision expansion fails closed | yes |
| R9 | Executable core remains model/vendor agnostic | yes |
| R10 | `make qa-docs` passes and M4 plan/tasks/operator docs agree | yes |
| R11 | No unresolved audit finding permits unsafe cloud disclosure, stale graph use, or unrestricted graph traversal | yes |
| R12 | Audit verdict explicitly states `READY_FOR_T4` or `READY_FOR_T4_WITH_CONDITIONS` | yes |

`READY_FOR_T4_WITH_CONDITIONS` is allowed only for non-security/non-contract items that do not weaken R0-R11. Conditions must be named and owned.

---

## 5. Mandatory audit sequence S0-S10

Run the sequence from a local checkout of the branch.

### S0 — identify branch, HEAD, and merge base

```bash
git switch feature/local-code-intelligence-boundary
git status --short
BRANCH="$(git branch --show-current)"
HEAD_SHA="$(git rev-parse HEAD)"
BASE="$(git merge-base origin/main HEAD)"
printf 'branch=%s\nhead=%s\nmerge_base=%s\n' "$BRANCH" "$HEAD_SHA" "$BASE"

test "$BRANCH" = "feature/local-code-intelligence-boundary"
```

Record the three values in the audit report.

A dirty working tree is not automatically a product defect, but the auditor must identify unrelated local modifications before trusting generated evidence.

---

### S1 — scope and no product/RRI regression

```bash
git diff --name-only "$BASE"...HEAD

git diff --exit-code "$BASE"...HEAD -- \
  apps \
  crates \
  mobile \
  infra \
  scripts/rri.py \
  docs/policies/RRI_POLICY.md
```

Expected: second command exits `0`.

The branch may contain code-intelligence docs/tooling plus pre-existing branch-local audit-hygiene fixes under repository script tests. Review those separately; do not confuse them with product-runtime coupling.

---

### S2 — syntax + behavioral unit suite

```bash
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py

python3 scripts/code_intelligence/context_gateway_test.py
```

Expected: compile succeeds and the unit runner reports all tests `OK`.

At minimum the suite must cover:

- HP-41 / EC-41 freshness;
- HP-42 / EC-42 / EC-43 minimum disclosure and mislabeled unsafe content;
- HP-43 / EC-44 / EC-45 bounded expansion.

---

### S3 — prepare a checkout-bound audit fixture and prove freshness happy path

The committed smoke fixture intentionally uses a synthetic Git revision. Create a temporary checkout-bound copy:

```bash
rm -rf /tmp/dubbridge-m4-audit
mkdir -p /tmp/dubbridge-m4-audit

python3 - <<'PY'
import json
import subprocess
from pathlib import Path

head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
src = Path('scripts/code_intelligence/fixtures/audit-smoke-graph.json')
payload = json.loads(src.read_text())
payload['git_revision'] = head
out = Path('/tmp/dubbridge-m4-audit/base.json')
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n')
print(head)
PY

HEAD_SHA="$(git rev-parse HEAD)"
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S3 \
  --task "Pre-T4 freshness happy path" \
  --backend-json /tmp/dubbridge-m4-audit/base.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --output-dir /tmp/dubbridge-m4-audit/base-out
```

Expected: both `context-receipt.json` and `context-capsule.json` exist and record the current `HEAD_SHA`.

---

### S4 — stale graph fails closed

```bash
rm -rf /tmp/dubbridge-m4-audit/stale-out
set +e
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S4 \
  --task "Pre-T4 stale graph rejection" \
  --backend-json /tmp/dubbridge-m4-audit/base.json \
  --target cloud \
  --expected-git-revision "definitely-not-${HEAD_SHA}" \
  --output-dir /tmp/dubbridge-m4-audit/stale-out
RC=$?
set -e

test "$RC" -ne 0
test ! -e /tmp/dubbridge-m4-audit/stale-out/context-receipt.json
test ! -e /tmp/dubbridge-m4-audit/stale-out/context-capsule.json
```

Expected: non-zero exit and no consumable success pair.

Any stale-graph acceptance is a **BLOCK** for T4.

---

### S5 — cloud minimum disclosure + defense-in-depth

First inspect the normal cloud capsule:

```bash
python3 - <<'PY'
import json
from pathlib import Path

c = json.loads(Path('/tmp/dubbridge-m4-audit/base-out/context-capsule.json').read_text())
s = json.dumps(c)

assert 'crates/playback/src/lib.rs' in c['files']
assert 'crates/auth/src/lib.rs' not in c['files']
assert 'auth::internal::session_topology' not in s
assert 'sensitive audit fixture value' not in s
assert 'ephemeral audit fixture value' not in s
assert 'audit smoke global architecture fragment' not in s
print('S5A PASS')
PY
```

Then deliberately mislabel an obviously unsafe fragment as `task_local`:

```bash
python3 - <<'PY'
import json
from pathlib import Path

src = Path('/tmp/dubbridge-m4-audit/base.json')
p = json.loads(src.read_text())
p['files'].append('.env.audit')
p['source_fragments'].append({
    'path': '.env.audit',
    'start_line': 1,
    'end_line': 1,
    'content': 'GITHUB_TOKEN=should-not-export',
    'classification': 'task_local',
})
Path('/tmp/dubbridge-m4-audit/mislabeled.json').write_text(
    json.dumps(p, indent=2, sort_keys=True) + '\n'
)
PY

rm -rf /tmp/dubbridge-m4-audit/mislabeled-out
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S5 \
  --task "Pre-T4 defense in depth" \
  --backend-json /tmp/dubbridge-m4-audit/mislabeled.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --output-dir /tmp/dubbridge-m4-audit/mislabeled-out

python3 - <<'PY'
import json
from pathlib import Path

c = json.loads(Path('/tmp/dubbridge-m4-audit/mislabeled-out/context-capsule.json').read_text())
s = json.dumps(c)
assert '.env.audit' not in s
assert 'should-not-export' not in s
print('S5B PASS')
PY
```

Any export of the deliberately unsafe material is a **BLOCK** for T4 cloud use.

---

### S6 — local remains richer but safe

```bash
rm -rf /tmp/dubbridge-m4-audit/local-out
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S6 \
  --task "Pre-T4 local policy" \
  --backend-json /tmp/dubbridge-m4-audit/base.json \
  --target local \
  --expected-git-revision "$HEAD_SHA" \
  --output-dir /tmp/dubbridge-m4-audit/local-out

python3 - <<'PY'
import json
from pathlib import Path

local = json.loads(Path('/tmp/dubbridge-m4-audit/local-out/context-capsule.json').read_text())
cloud = json.loads(Path('/tmp/dubbridge-m4-audit/base-out/context-capsule.json').read_text())
s = json.dumps(local)

assert len(local['files']) >= len(cloud['files'])
assert 'crates/auth/src/lib.rs' in local['files']
assert 'sensitive audit fixture value' not in s
assert 'ephemeral audit fixture value' not in s
print('S6 PASS')
PY
```

---

### S7 — determinism and receipt/capsule integrity

Generate the same initial request twice:

```bash
rm -rf /tmp/dubbridge-m4-audit/det-a /tmp/dubbridge-m4-audit/det-b
for OUT in /tmp/dubbridge-m4-audit/det-a /tmp/dubbridge-m4-audit/det-b; do
  python3 scripts/code_intelligence/context_gateway.py \
    --task-id M4-AUDIT-S7 \
    --task "Pre-T4 deterministic artifacts" \
    --backend-json /tmp/dubbridge-m4-audit/base.json \
    --target cloud \
    --expected-git-revision "$HEAD_SHA" \
    --output-dir "$OUT"
done

cmp /tmp/dubbridge-m4-audit/det-a/context-capsule.json \
    /tmp/dubbridge-m4-audit/det-b/context-capsule.json
cmp /tmp/dubbridge-m4-audit/det-a/context-receipt.json \
    /tmp/dubbridge-m4-audit/det-b/context-receipt.json
```

Verify hashes:

```bash
python3 - <<'PY'
import hashlib
import json
from pathlib import Path

root = Path('/tmp/dubbridge-m4-audit/det-a')
receipt = json.loads((root / 'context-receipt.json').read_text())
capsule = json.loads((root / 'context-capsule.json').read_text())

def digest(v):
    return hashlib.sha256(json.dumps(
        v, sort_keys=True, separators=(',', ':'), ensure_ascii=False
    ).encode()).hexdigest()

assert digest(capsule) == receipt['capsule_sha256']
expected = receipt.pop('receipt_sha256')
assert digest(receipt) == expected
print('S7 PASS')
PY
```

---

### S8 — bounded expansion happy path

Create an expanded graph result with one additional task-local helper while preserving Git and graph revisions:

```bash
python3 - <<'PY'
import json
from pathlib import Path

p = json.loads(Path('/tmp/dubbridge-m4-audit/base.json').read_text())
p['files'].append('crates/playback/src/helper.rs')
p['symbols'].append('crate::playback::helper')
p['tests'].append('playback::tests::helper_happy_path')
p['relationships'].append({
    'from': 'crate::playback::resolve_stream',
    'to': 'crate::playback::helper',
    'kind': 'calls',
    'classification': 'task_local',
})
p['source_fragments'].append({
    'path': 'crates/playback/src/helper.rs',
    'start_line': 1,
    'end_line': 5,
    'content': 'audit helper task-local fragment',
    'classification': 'task_local',
})
Path('/tmp/dubbridge-m4-audit/expanded.json').write_text(
    json.dumps(p, indent=2, sort_keys=True) + '\n'
)
PY

rm -rf /tmp/dubbridge-m4-audit/expanded-out
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S3 \
  --task "Pre-T4 freshness happy path" \
  --backend-json /tmp/dubbridge-m4-audit/expanded.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --base-receipt /tmp/dubbridge-m4-audit/base-out/context-receipt.json \
  --expansion-reason "Need the adjacent playback helper" \
  --output-dir /tmp/dubbridge-m4-audit/expanded-out

python3 - <<'PY'
import json
from pathlib import Path

r = json.loads(Path('/tmp/dubbridge-m4-audit/expanded-out/context-receipt.json').read_text())
c = json.loads(Path('/tmp/dubbridge-m4-audit/expanded-out/context-capsule.json').read_text())
assert 'crates/playback/src/helper.rs' in c['files']
assert r['expansions'][-1]['decision'] == 'reduce'
assert r['expansions'][-1]['reason'] == 'Need the adjacent playback helper'
assert 'crates/playback/src/helper.rs' in r['expansions'][-1]['added']['files']
print('S8 PASS')
PY
```

The committed smoke fixture already contains records that cloud policy
excludes. The expansion therefore records the aggregate decision as `reduce`
while still adding the justified task-local helper. HP-43's isolated unit
fixture contains no pre-existing exclusions and continues to exercise the
pure `allow` decision.

---

### S9 — expansion deny policy and stale revision cannot be bypassed

#### S9A — forbidden expansion is denied/reduced

```bash
python3 - <<'PY'
import json
from pathlib import Path

p = json.loads(Path('/tmp/dubbridge-m4-audit/base.json').read_text())
p['source_fragments'].append({
    'path': 'docs/full-system-topology.md',
    'start_line': 1,
    'end_line': 5,
    'content': 'audit forbidden global topology',
    'classification': 'global_architecture',
})
Path('/tmp/dubbridge-m4-audit/forbidden-expansion.json').write_text(
    json.dumps(p, indent=2, sort_keys=True) + '\n'
)
PY

rm -rf /tmp/dubbridge-m4-audit/forbidden-out
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S3 \
  --task "Pre-T4 freshness happy path" \
  --backend-json /tmp/dubbridge-m4-audit/forbidden-expansion.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --base-receipt /tmp/dubbridge-m4-audit/base-out/context-receipt.json \
  --expansion-reason "Give broad architecture topology" \
  --output-dir /tmp/dubbridge-m4-audit/forbidden-out

python3 - <<'PY'
import json
from pathlib import Path

r = json.loads(Path('/tmp/dubbridge-m4-audit/forbidden-out/context-receipt.json').read_text())
c = json.loads(Path('/tmp/dubbridge-m4-audit/forbidden-out/context-capsule.json').read_text())
assert 'audit forbidden global topology' not in json.dumps(c)
assert r['expansions'][-1]['decision'] in {'deny', 'reduce'}
print('S9A PASS')
PY
```

#### S9B — different graph revision fails closed

```bash
python3 - <<'PY'
import json
from pathlib import Path

p = json.loads(Path('/tmp/dubbridge-m4-audit/expanded.json').read_text())
p['graph_revision'] = 'different-graph-revision'
Path('/tmp/dubbridge-m4-audit/stale-expansion.json').write_text(
    json.dumps(p, indent=2, sort_keys=True) + '\n'
)
PY

rm -rf /tmp/dubbridge-m4-audit/stale-expansion-out
set +e
python3 scripts/code_intelligence/context_gateway.py \
  --task-id M4-AUDIT-S3 \
  --task "Pre-T4 freshness happy path" \
  --backend-json /tmp/dubbridge-m4-audit/stale-expansion.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --base-receipt /tmp/dubbridge-m4-audit/base-out/context-receipt.json \
  --expansion-reason "Try stale expansion" \
  --output-dir /tmp/dubbridge-m4-audit/stale-expansion-out
RC=$?
set -e

test "$RC" -ne 0
test ! -e /tmp/dubbridge-m4-audit/stale-expansion-out/context-receipt.json
test ! -e /tmp/dubbridge-m4-audit/stale-expansion-out/context-capsule.json
```

Any stale expansion acceptance is a **BLOCK** for T4.

---

### S10 — model/vendor independence + repository documentation QA

```bash
if grep -RniE \
  'nemotron|qwen|gemma|muse|claude|codex|ollama|codebase-memory|codegraph' \
  scripts/code_intelligence/*.py; then
  echo 'S10 FAIL: model/vendor coupling found in executable core'
  exit 1
fi

make qa-docs
```

Expected: no executable-core coupling and documentation QA passes.

If repository-standard supplementary script QA is normally required for the changed script-test files already present on the branch, run it and record the result separately.

---

## 6. Findings and severity

Classify findings by operational consequence, not theoretical elegance.

### BLOCKER

Any of the following blocks T4:

- stale Git/graph context can produce a consumable artifact;
- unsafe/secret/runtime content reaches a cloud capsule;
- bounded expansion can bypass cloud deny policy;
- a cloud consumer can obtain unrestricted graph traversal through this interface;
- receipt integrity cannot be verified;
- tests/documentation gates required above fail in a way that invalidates the contract;
- M4 introduces product runtime or RRI/model-routing changes outside approved scope.

### MAJOR

T4 may be blocked or conditionally allowed depending on safety impact:

- recurring false-positive policy blocks that make ordinary work impractical;
- metadata over-disclosure without source-content leakage;
- pair-level artifact race demonstrated with a real consumer;
- graph lifecycle race that is fail-closed but repeatedly disrupts work;
- significant host pressure attributable to indexing/query lifecycle.

### MINOR

Does not block T4 unless it accumulates into operational friction:

- documentation wording;
- ergonomics of audit/output paths;
- non-recurring redundant context;
- optional diagnostics that do not change safety/correctness.

Do not create speculative hardening from MINOR findings unless normal T4 use later proves it recurring/material.

---

## 7. Audit verdict

Use exactly one:

### `READY_FOR_T4`

All R0-R12 pass. No blocking finding remains.

### `READY_FOR_T4_WITH_CONDITIONS`

All safety/contract prerequisites pass. Remaining conditions are non-blocking, explicitly named, assigned, and cannot weaken freshness, minimum disclosure, integrity, or bounded expansion.

### `NOT_READY_FOR_T4`

Any mandatory readiness gate fails or a blocker remains.

Do not translate `NOT_READY_FOR_T4` into a weaker gateway policy. Fix the defect or keep the code-intelligence path disabled for T4.

---

## 8. Required audit report format

```md
# M4 Pre-T4 Audit Result

Branch: feature/local-code-intelligence-boundary
HEAD: <sha>
Merge base with origin/main: <sha>
Auditor: <agent/person>
Date: <date>

## Mandatory smokes
- S0: PASS/FAIL — evidence
- S1: PASS/FAIL — evidence
- S2: PASS/FAIL — evidence
- S3: PASS/FAIL — evidence
- S4: PASS/FAIL — evidence
- S5: PASS/FAIL — evidence
- S6: PASS/FAIL — evidence
- S7: PASS/FAIL — evidence
- S8: PASS/FAIL — evidence
- S9: PASS/FAIL — evidence
- S10: PASS/FAIL — evidence

## Findings
- <severity> <finding> — <file/function/reproduction>

## Deferred items
- <item> — <why safe to defer>

## T4 prerequisites
- R0-R12: PASS/FAIL summary

## Verdict
READY_FOR_T4 | READY_FOR_T4_WITH_CONDITIONS | NOT_READY_FOR_T4

## Conditions, if any
- <condition + owner + next action>
```

The completed report should be committed under `docs/audit/` or another repository-standard audit evidence location before T4 begins.

---

## 9. Handoff into M4-T4

A successful audit does **not** mean M4 is complete. It means T0-T3 are ready to be exercised in normal work.

### 9.1 First T4 task selection

Use the **next ordinary DubBridge development task** that would normally pass through Analyze/handoff. Do not create a synthetic benchmark task and do not wait for an artificially ideal scenario.

The first T4 task must follow its normal repository workflow, RRI, approval, implementation, review, and verification rules. Code intelligence is a preprocessing/context-selection capability, not a replacement workflow.

### 9.2 T4 operational sequence

For each ordinary task using the boundary:

```text
Normal DubBridge task
        |
        v
Analyze / local graph result
        |
        v
expected HEAD revision captured
        |
        v
Context Gateway
        |
        +--> Context Receipt
        +--> bounded Context Capsule
                    |
                    v
             existing agent routing
                    |
          enough context?
             /          \
           yes           no
            |             |
            |             v
            |       bounded expansion request
            |             |
            |       local policy evaluation
            |             |
            +-------------+
                    |
                    v
           source/tests verification
                    |
                    v
             normal task review
```

### 9.3 Initial artifact generation

The orchestrator must capture the exact revision being analyzed:

```bash
HEAD_SHA="$(git rev-parse HEAD)"
```

Then the real local graph producer/adapter supplies the task-relevant backend JSON result, and the gateway is invoked with that same `HEAD_SHA` as `--expected-git-revision`.

Do not substitute a branch name such as `main` for the immutable revision check.

### 9.4 Context sufficiency rule

If the initial capsule is sufficient, proceed. Do **not** request additional context merely to make the packet look complete.

If context is insufficient:

1. the agent states the missing context and concrete reason;
2. the local side resolves only the needed additional graph/source evidence;
3. expansion runs through `--base-receipt` + `--expansion-reason`;
4. the new capsule remains bounded by the same target policy;
5. the agent continues from the new capsule.

Never solve missing context by giving a cloud agent direct MCP/graph traversal.

### 9.5 Authority rule during T4

The receipt/capsule is navigation/context evidence only.

The agent must still verify relevant claims against source, tests, ADRs, and repository policy before implementation/review. A graph relationship does not override source reality.

### 9.6 What T4 records

Do not create per-task KPI reports. Record a friction note **only when something actionable happens**.

Allowed friction categories:

- `stale_graph`
- `missing_context`
- `irrelevant_context`
- `policy_over_blocking`
- `policy_under_blocking`
- `host_resource_pressure`
- `artifact_consumer_race`
- `graph_source_mismatch`

Minimal friction note:

```yaml
task: <task-id>
head: <git-sha>
receipt: <receipt-sha256>
category: <category>
observation: <short reproducible description>
impact: <what the agent could/could not do>
action: none | adjust-later | create-M4-T5
```

No note is required when the mechanism simply works.

### 9.7 When a T4 finding becomes M4-T5

Create a narrowly scoped T5 task when a finding is reproducible and either:

- risks unsafe disclosure/correctness;
- recurs across normal work;
- materially prevents ordinary tasks from using the boundary; or
- creates measurable host/runtime instability attributable to this subsystem.

Each T5 development task receives its own RRI, HP/EC cases, review route, tests, and evidence.

Do **not** create T5 merely to add:

- a dashboard;
- Neo4j/GraphRAG;
- a generic policy DSL;
- automatic RRI changes;
- broader cloud graph access;
- theoretical optimizations with no real-task evidence.

### 9.8 T4 stop conditions

Stop using the M4 boundary for cloud context and create/fix a blocking T5 issue if any real task demonstrates:

- unsafe content exported to cloud;
- stale graph accepted;
- expansion bypassing deny policy;
- receipt/hash integrity failure;
- cloud workflow requiring unrestricted graph exploration to function;
- graph/source disagreement that is not caught by source verification and could misdirect implementation.

Do not weaken the gateway to keep the task moving. The normal source-first repository workflow remains authoritative; any fallback must comply with existing DubBridge governance.

---

## 10. T4 completion and transition toward T6

M4-T4 has **no synthetic task quota or benchmark threshold**.

However, T4 cannot be considered operationally exercised without at least one ordinary DubBridge task actually passing through the audited path.

After real use:

- if no material friction appears, keep using the mechanism; no optimization work is required;
- if material friction appears, create only the necessary T5 task(s);
- T4 can move toward closure when real use has occurred and no unresolved blocking T5 finding remains.

M4-T6 then reconciles plan, task ledger, audit evidence, schemas, and operator documentation and makes the ADR decision (`required`, `amend existing`, or `not required yet`).

---

## 11. Audit closure checklist

Before declaring `READY_FOR_T4`, confirm:

- [ ] correct branch/HEAD recorded;
- [ ] merge base recorded;
- [ ] S0-S10 executed from the branch checkout;
- [ ] freshness happy/stale behavior proven;
- [ ] cloud metadata minimization proven;
- [ ] mislabeled unsafe data blocked;
- [ ] local richer/safe behavior proven;
- [ ] receipt/capsule determinism + hashes proven;
- [ ] bounded expansion happy path proven;
- [ ] forbidden/stale expansion cannot bypass policy;
- [ ] executable core remains model/vendor agnostic;
- [ ] `make qa-docs` passes;
- [ ] no product-runtime/RRI drift introduced;
- [ ] findings classified and blockers resolved;
- [ ] final verdict recorded;
- [ ] audit report committed;
- [ ] next ordinary DubBridge task may enter M4-T4 using Section 9.

Only then should the M4 task ledger move T4 from `next after branch-local QA` to `operational/in progress`.
