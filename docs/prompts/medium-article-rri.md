---
type: Prompt
title: "Prompt: Medium Article on RRI (Required Reasoning Index)"
---

# Prompt: Medium Article on RRI (Required Reasoning Index)

**Target model:** `gemma4:26b-a4b-it-qat`

**Usage via Ollama (recommended — avoids escape codes and truncation):**
```bash
curl -s http://localhost:11434/api/generate \
  -d "{\"model\":\"gemma4:26b-a4b-it-qat\",\"prompt\":$(jq -Rs . < docs/prompts/medium-article-rri.md),\"stream\":false}" \
  | jq -r '.response' > docs/prompts/medium-article-rri-output.md
```

**Usage via Cline:** paste this file's content as the first message in a new chat with `gemma4:26b-a4b-it-qat`. Ignore capability warnings — no tools or agentic features are used.

---

## Your role

You are a senior software engineer and technical writer. You write for Medium's
engineering audience: practitioners who build with AI agents, work in product
engineering teams, and care about software quality and process. Your tone is
direct, concrete, and opinionated — you make a clear argument, back it with
specifics, and avoid filler.

---

## Task

Write a complete, publication-ready Medium article in English about the
**Required Reasoning Index (RRI)**: what it is, why it exists, how it works,
and how it changes the way AI agents operate on a real codebase.

The article must be self-contained. A reader with no prior knowledge of RRI
should finish it understanding: the problem it solves, the mechanics, and how
to apply it.

---

## Background: what RRI is (read carefully before writing)

### The problem

AI coding agents are powerful but undifferentiated in how they approach tasks.
They apply the same level of scrutiny to editing a README as they do to
modifying a JWT authentication boundary. This is dangerous: the agent that
auto-executes a docs fix should not auto-execute a change to auth middleware
with the same confidence.

The industry defaults to one of two bad solutions:
- **HITL everywhere** — a human approves every agent action. Slow, doesn't
  scale, defeats the purpose of the agent.
- **Trust the agent entirely** — no checkpoints. Works until it doesn't.

RRI is a third path: a deterministic numeric score that tells the agent (and
the human) exactly how much caution, verification, and oversight a specific
task requires — before any code is written.

### The formula

```
RRI = 100 × ((0.18·C + 0.12·F + 0.15·D + 0.15·T + 0.12·A + 0.12·K + 0.10·P + 0.06·X) / 5)
    + Penalties
```

Eight variables, each scored 0–5. Weights sum to 1.00. The base term is always
in [0, 100]. Additive penalties push the score above 100.

### The eight variables

| Variable | Name | Nature | What it measures |
|---|---|---|---|
| C | Cyclomatic complexity | Objective | How complex is the code being written or changed? |
| F | Files affected | Objective | How many files does this task touch? |
| D | Domain complexity | Subjective (rubric-anchored) | What layer of the system does this task operate in? |
| T | Test-coverage risk | Semi-objective | How well-tested is the area being changed? |
| A | Task ambiguity | Subjective | Does the task have clear acceptance criteria and examples? |
| K | Coupling / side effects | Subjective (rubric-anchored) | How many things depend on what's being changed? |
| P | Public API / security / data impact | Subjective (rubric-anchored) | What's the blast radius if this goes wrong? |
| X | Context size required | Subjective | How much context must the agent hold in mind? |

**Objective variables** (C, F) are measured, not estimated — from tools like
`cargo clippy`, `radon`, `gocyclo`, `eslint`, or `git diff --name-only`.

**Subjective variables** (D, K, P) are anchored to a rubric so that two
independent agents score the same task identically. Example anchor:
- `crates/auth/*`, JWT boundary → D≥4, P≥4, K≥4 (ADR-023)
- `infra/migrations/*` → D≥4, P≥5, K≥4 (ADR-008, ADR-018)
- `docs/**`, formatting → D=0, P=0, K=0

The rubric enforces floors: the agent may score higher, never lower.

### Penalties

Additive, independent, each applied at most once:

| Condition | +Points |
|---|---|
| Refactor + behavior change in same task | +8 |
| No tests and high security/data impact (P ≥ 4) | +10 |
| High complexity and high domain (C ≥ 4 and D ≥ 3) | +10 |
| Task touches auth, authz, permissions, or sensitive data | +10 |
| More than 10 files affected (F ≥ 4) | +8 |
| Architecture or policy decision required | +12 |
| No verification strategy exists | +15 |

### The bands (what the number means operationally)

| RRI | Label | Gate |
|---|---|---|
| 0–25 | Low | Auto-execute: show what you're doing, then do it immediately. No approval. |
| 26–40 | Moderate | Confirm tests exist in the affected area. |
| 41–55 | Med-high | Plan + explicit acceptance criteria required before approval. |
| 56–70 | Complex | Human reviews the plan before any implementation. |
| 71–85 | High | Characterization tests + human reviews the diff, not just the plan. |
| 86–100 | Very high | Produce an ADR + risk analysis + decompose into subtasks first. |
| >100 | Excessive | Stop. Architecture work must happen before any implementation. |

This is not a soft guideline. The band is the gate. The agent cannot skip it.

### Decomposition triggers

Split the task before implementing if:
- RRI > 70, or base RRI > 100
- F ≥ 4 and K ≥ 3 (large surface + high coupling)
- C ≥ 4 and D ≥ 3 (complex logic in sensitive domain)
- Refactor + behavior change combined (always separate these)
- No tests + high impact (characterization tests first, implementation second)

**Split target:** each subtask should score RRI ≤ 55.

### The script

The computation is done by a deterministic Python script (`scripts/rri.py`),
not by the agent's reasoning. The script:
- Measures F from `--touches` paths or `git diff`
- Auto-detects the platform (Rust, Go, Python, React Native) by marker files
  (`Cargo.toml`, `go.mod`, `package.json`, `pyproject.toml`) and runs the
  appropriate CC measurer (`cargo clippy`, `gocyclo`, `eslint`, `radon`)
- Derives D/P/K floors from the anchor rubric and raises agent input to the
  floor — never lowers it
- Auto-applies the four derivable penalties
- Outputs the full band, model tier recommendation, thinking mode, and gate

**The agent supplies only the irreducible judgments**: T (test coverage risk),
A (ambiguity), X (context), and D/P/K above the floor. Everything
deterministic is computed by the script.

Example invocation:
```bash
python3 scripts/rri.py \
  --auto-cc \
  --touches crates/auth/src/lib.rs \
  --D 2 --K 2 --P 0 \
  --T 2 --A 1 --X 2
```

Example output:
```
**Platform:** dubbridge

| Variable     | Score | Evidence                                          | Confidence |
|---|---|---|---|
| C cyclomatic | 1     | cargo clippy -> max CC 8 -> score 1               | High       |
| F files      | 0     | --touches -> 1 file                               | High       |
| D domain     | 4     | anchor rubric: crates/auth (ADR-023) -> floor 4   | High       |
| T coverage   | 2     | agent-supplied                                    | High       |
| A ambiguity  | 1     | agent-supplied                                    | High       |
| K coupling   | 4     | anchor rubric: crates/auth (ADR-023) -> floor 4   | High       |
| P impact     | 4     | anchor rubric: crates/auth (ADR-023) -> floor 4   | High       |
| X context    | 2     | agent-supplied                                    | High       |

**Base value:** 100 × (weighted / 5) = 42
**Penalties applied:** auth_security (+10, P floor >= 4)
**Final RRI:** 52 → band Med-high (41–55) → Effort L · thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
```

### The key insight

RRI is task-scoped, not project-scoped. Two tasks in the same repo can have
RRI 3 and RRI 78. The same task presented twice with different acceptance
criteria will score differently on A. This is intentional: the metric measures
the specific action being taken, not the general state of the codebase.

---

## Article requirements

### Structure (use this as your guide, not your outline)

The article should flow as a coherent argument, not a numbered list of
features. Suggested narrative arc:

1. **Open with the real problem** — agents that treat auth changes and README
   edits with the same caution level. Make it concrete: a short story or
   scenario. One specific failure mode is worth three abstract paragraphs.

2. **Introduce RRI as the answer** — one paragraph, direct. What it is, what
   it produces, what it changes.

3. **Walk through the mechanics** — the formula, the variables, the bands.
   Don't just list them: explain the *reasoning* behind the design choices.
   Why are C and F objective while D and K are rubric-anchored? Why do
   penalties exist rather than just scoring those conditions into the variables?
   Why is "excessive" a separate band rather than just "very high"?

4. **Show the script in action** — a realistic example with a real invocation
   and real output. Show what the agent supplies vs. what the script decides.
   This is the most concrete part: make the reader feel what it's like to
   use it.

5. **The platform profile angle** — briefly explain that the same RRI logic
   works across Rust, Go, Python, React Native because the formula is
   universal and only the CC measurer + anchor rubric vary per ecosystem.
   This is what makes it portable, not project-specific.

6. **What changes for the agent** — not just the mechanics, but the
   behavioral shift. An agent with RRI knows when to slow down, when to
   decompose, and when it can just go. It stops being a yes-machine.

7. **Close with the bigger idea** — RRI is an example of a broader pattern:
   replacing soft guidelines ("be careful with auth") with deterministic,
   auditable criteria. What does engineering process look like when the agent
   knows its own risk level?

### Tone and style

- **Concrete over abstract.** Every concept should have an example.
- **Opinionated.** Don't hedge. Make the argument.
- **No filler.** Don't write "In conclusion" or "As we have seen". Cut any
  sentence that doesn't add information.
- **Code blocks for anything technical.** Formula, invocations, output.
- **Tables for reference material.** Variables, bands, penalties.
- **Target length:** 1,400–1,900 words. Medium's sweet spot for technical
  articles. Long enough to be substantive, short enough to be read.

### Things to avoid

- Do not frame RRI as "a new framework" or "a revolutionary approach". It is
  an engineering tool. Describe it as one.
- Do not list all eight variables with identical depth. Some deserve more
  explanation than others (D, P, K are more interesting because they're
  rubric-anchored; F is trivial).
- Do not repeat the formula more than twice.
- Do not end with a call to action ("try it today!"). End with an idea.

### Suggested title formats (pick one or write a better one)

- *The Agent Knows Its Own Risk Level*
- *Giving AI Agents a Sense of Danger*
- *RRI: A Complexity Score That Actually Governs Agent Behavior*
- *Stop Trusting Agents Blindly. Give Them a Number.*

---

## Output format

Produce the full article as Markdown, ready to paste into Medium's editor.
Use:
- `#` for the title
- `##` for section headers (use sparingly — 3–4 max, or none if the narrative
  flows without them)
- ` ``` ` for code blocks
- `|` tables for the bands, variables, and penalties where appropriate
- No meta-commentary, no "here is the article", no preamble. Start directly
  with the article title.
- Do not show planning notes, thinking steps, bullet outlines, or any internal
  reasoning. Output only the finished article. Begin your response with `#`.
