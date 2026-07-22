# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development commands

```bash
# Local QA (run before committing)
make qa-local          # fmt + clippy + test + cargo check
make qa-coverage       # 90% line coverage gate (llvm-cov, --test-threads=1)
make qa-ci             # full CI mirror: local + docs + rri + deny + mobile + coverage + release build

# Individual gates
make qa-fmt            # cargo fmt --check
make qa-lint           # cargo clippy -D warnings
make qa-test           # cargo test --workspace --all-features
make qa-deny           # dependency advisories / policy
make qa-docs           # doc consistency + task coverage + roadmap drift + OKF frontmatter (deterministic, no LLM)
make qa-docs-review    # qa-docs + Gemma Reviewer pass (task closure / CI, not pre-push)
make qa-mobile         # cd mobile && typecheck + lint + Jest
make qa-roadmap-drift  # script: ledger ↔ roadmap consistency
make qa-maintainability # python3 scripts/check-maintainability.py

# Single crate/test
cargo test -p <crate-name>
cargo test -p <crate-name> -- <test_name> --nocapture

# RRI scoring for a task before implementation
python3 scripts/rri.py

# OKF frontmatter validator (all docs/ .md files need YAML frontmatter)
make qa-okf-frontmatter
```

Local infrastructure (Postgres + Redis + MinIO — never the production descriptor):

```bash
docker compose -f infra/local/docker-compose.yml up -d postgres redis minio
DUBBRIDGE_ENV=local cargo run -p dubbridge-api
```

Install git hooks once per clone:

```bash
git config core.hooksPath .githooks
# or: make install-hooks
```

Mobile:

```bash
cd mobile && npm install
npm run typecheck && npm run lint && npm test
npm run screenshots   # Maestro E2E visual suite (requires Java + device)
```

## Architecture

**Rust workspace** (`Cargo.toml`) owns API, orchestration, persistence, and all governance gates. Python is isolated to AI workers behind typed JSON contracts (`workers/*-py`).

### Apps

| App | Role |
|-----|------|
| `apps/api` | Axum HTTP API; asset ingestion, rights, finalize, HLS playback endpoints |
| `apps/gateway` | Session gateway / BFF; transparent JWT relay after ADR-031/S-200 |
| `apps/worker-runner` | Background job execution surface (apalis) |
| `apps/cli` | Local operational utilities |

### Shared crates (dependency order: domain → db/storage → ingestion → apps)

| Crate | Role |
|-------|------|
| `domain` | Core entities and invariants (no DB, no IO) |
| `db` | SQLx repositories; PostgreSQL is the system of record |
| `storage` | `StorageAdapter` abstraction; local-fs and S3-compatible backends; canonical key layout |
| `ingestion` | `finalize_ingestion_core` — the single fail-closed finalize boundary reused by all intake modes (ADR-021) |
| `auth` | JWT verification, scope enforcement, principal propagation |
| `audit` | Centralized durable audit-emission boundary; domain event types stay in `domain` |
| `jobs` | Job type definitions and apalis scheduling adapters |
| `media` | ffprobe metadata extraction + HLS transcode orchestration |
| `playback` | HLS grant issuance, manifest rewriting, short-lived scoped segment references (ADR-032) |
| `qc` | Deterministic quality checks |
| `config` | Typed fail-closed config loader; requires `DUBBRIDGE_ENV ∈ {local,staging,production}`; production rejects localhost defaults (ADR-026) |
| `connectors` | Planned: `PlatformConnector` trait for owner-authorized platform downloads (ADR-025) |
| `observability` | Tracing/logging helpers; production requires JSON format (ADR-018) |

### Key invariants

- **Rights gate is fail-closed (ADR-008):** no asset moves to any processing stage without a confirmed rights basis.
- **Single finalize path (ADR-021):** all intake modes (upload, platform download, live recording) must use `finalize_ingestion_core`; no parallel weaker path.
- **Immutable artifacts (ADR-006):** ingested originals are never modified; derivatives carry explicit lineage and SHA-256 checksums.
- **Durable audit (ADR-018):** every governance-significant event requires a PostgreSQL audit row plus correlated structured tracing. No fire-and-forget.
- **Compiled defaults forbidden (ADR-026):** `crates/config` loads `config/<env>.toml` + env vars; nothing environment-specific is baked into the binary.
- **Mobile is the only authenticated product UI (ADR-029):** `apps/gateway` is a relay; `mobile/` is the sole first-party surface.

### OKF frontmatter

Every file in `docs/` must have YAML frontmatter declaring `type:` (one of 10 closed values — see `docs/knowledge/README.md`) and `status:`. The `pre-commit` hook and `make qa-docs` enforce this; missing or invalid frontmatter blocks commits.

## Purpose

This file defines how Claude Code should present staged tasks in the `dubbridge` repository.

It is intentionally aligned with `AGENTS.md` so Codex and Claude Code follow the same task-presentation contract.

## Canonical Agent Guides

These documents are the authoritative guides for how agents plan and implement work
in this repository. Read them in this order before acting on any task:

1. `README_AGENT_ORDER.md` — orientation and reading order for agents.
2. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the mandatory workflow
   (analyze → plan → tasks → approval → implement → mark progress).
3. `docs/policies/HITL_AUTONOMY_POLICY.md` — when explicit human approval is
   required and what autonomy is permitted.
4. `AGENTS.md` — the shared task-presentation contract.
5. `docs/adr/` — architecture decisions that constrain implementation.
6. `docs/plan/roadmap.md` — the general plan: slice sequence, dependencies, and
   where each slice/task sits. Read it to locate any task before implementing.

**Precedence rule:**
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is the highest-authority source for
**all** agent-facing decisions: workflow, process, implementation discipline,
task presentation structure, model selection, complexity scoring, testing rules,
commit rules, handoff format, ADR propagation, and language policy.
It overrides this `CLAUDE.md`, the user's global `CLAUDE.md`, and every other
guide listed above without exception.

This `CLAUDE.md` and the user's global `CLAUDE.md` remain authoritative only for
topics not covered by `AGENT_WORKFLOW_GUIDE.md`.

When answering about development-task completion or before marking a
development task done, first determine whether the task is exempt (docs-only,
config-only, migration-only, planning, ADR, task-ledger, or policy-only) or
whether the workflow requires the mandatory `Gemma Reviewer` / D14 review gate
before certification or final verification is discussed.

## Task Presentation Contract

Before executing a task that belongs to a staged plan or task list, present the task first when the workflow or the user requires approval.

Use this structure:

1. `Task ID` and `Task title`
2. `Status`
3. `Effort`
4. `Complexity`
5. `Recommended model`
   - Codex recommendation
   - Claude Code recommendation
6. `Objective`
7. `Context`
8. `Related documents`
9. `Inputs`
10. `Outputs`
11. `Acceptance criteria`
12. `Execution summary`
13. `Pseudocode` if applicable
14. `Diagram`
   - required for development tasks
   - otherwise include if applicable
15. explicit approval wait-state when required

## Complexity And Model Defaults

**When RRI has been computed**, the `Complexity` field in the task presentation must
use the RRI band name — not the Effort-based mapping below:

| RRI range | Complexity to present |
|---|---|
| 0–25 | Low |
| 26–40 | Moderate |
| 41–55 | Med-high |
| 56–70 | Complex |

The Effort → Complexity mapping is a **fallback** used only when no RRI is available:

- `Effort: S` -> `Complexity: Low`
- `Effort: M` -> `Complexity: Medium`
- `Effort: L` -> `Complexity: High`

Default model recommendations:

- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

For RRI 0–25 Low-band tasks, follow the repository workflow guide instead of
these defaults: use local Gemma through Ollama only for eligible simple code
patches; otherwise handle the task directly and report as the orchestrator of
record.

Escalate Claude Code to `Claude Opus 4.1` only for heavy synthesis, long-context comparison, or repeated failure under Sonnet 4.

If the task file defines explicit complexity or model guidance, follow the task file.

## Context Rule

The context section must explain:

- why the task exists
- where it sits in the current stage or plan
- what it unlocks next

Keep it brief and decision-oriented.

## Related Documents Rule

List only the documents that directly govern the task.

Typical sources:

- task file
- linked plan
- workflow guides
- autonomy or policy files
- ADRs
- prompt files
- configs or templates

## Pseudocode Rule

Add pseudocode only when it improves approval quality for non-trivial logic, transformations, workflows or decision trees.

Skip pseudocode for straightforward document creation, direct shell operations or single-file edits.

## Diagram Rule

For development tasks, add a compact Mermaid diagram in every task presentation.
Its purpose is to make the concept, flow, boundary, dependency direction, state
transition, or ownership split easy to approve before implementation starts.

For non-development tasks, add a Mermaid diagram only when boundaries, flows or
architecture are materially easier to evaluate visually.

Skip diagrams for simple documentation tasks unless the subject itself is architectural.

## Approval Line

When approval is required, end with:

`Execution has not started. Approve this task to proceed.`

## Development Closure Rule

For development-task closure, do not present unit coverage certification,
owner final verification, or `[x] Done` as the first completion step. First
state whether the mandatory `Gemma Reviewer` / D14 review gate applies under
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/HITL_AUTONOMY_POLICY.md`, then list the remaining closure
requirements in order.

## Language

Repository instruction files are written in English.

User-facing explanations may be localized, but task metadata and file references should remain precise.
