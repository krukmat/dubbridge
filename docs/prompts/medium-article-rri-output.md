# How Much Does This Task Actually Cost? A Score for AI Agents

> Task complexity is cost. Measure it or pay for it later.

The problem became obvious the moment I started using AI agents on a real codebase. The agent would fix a typo in a README and then, twenty minutes later, propose changes to a JWT (JSON Web Token) authentication boundary — with the exact same confidence level. No signal that one of those required careful reasoning and the other didn't. No change in tone, no flag, no hesitation.

I tried both standard approaches. Approving every action myself turned the agent into a slow rubber stamp — I was doing the thinking anyway. Letting it run unsupervised meant I was one missed diff away from a bad day. Neither worked.

So I built a third option: something that could tell the agent how risky a task was before it wrote a line. I call it the Required Reasoning Index (RRI).

RRI is a deterministic score that maps a specific task to an operational gate. Instead of trusting the agent to judge how much reasoning a task requires, it gives the agent a number: how much caution, verification, and oversight this particular change warrants, based on the actual technical reality of the code it's about to touch.

But the score does more than set a safety gate. It also decides which model runs the task and whether a human needs to see it at all. A low-RRI task — refactoring a utility function, updating a config value — can be delegated to a lighter, cheaper model and executed without review. A high-RRI task gets a more capable model, extended reasoning, and a human in the loop before anything is touched. Without a reliable, deterministic score, those decisions fall back to the agent judging its own complexity — and an agent that can't reliably assess risk can't reliably assess itself either.

The third use took me longer to see: a high score is a signal to replan, not to escalate. If a task scores above 70, the right move isn't to throw a better model at it — it's to split it until each piece scores below 55. Smaller tasks mean less surface area, fewer side effects to miss, and less room for the agent to make questionable calls that quietly become technical debt. Complexity is where debt hides.

## Where RRI Lives in the Workflow

RRI isn't a standalone tool — it's the decision point inside a structured development loop. The workflow looks like this:

1. **Analyze** — the agent reads the codebase, identifies affected files and dependencies.
2. **Plan** — the agent writes a plan: objective, affected files, design decisions.
3. **Score** — the agent runs `rri.py` against the task. This is where the number is produced.
4. **Present and wait** — the agent shows the full RRI breakdown, the band, and the gates. If the score is above 25, it stops and waits for explicit human approval before writing a single line of code.
5. **Implement** — once approved, the agent works task by task in the defined order.
6. **Mark progress** — after each task, the agent updates the task ledger. This is the crash-safe record of what was done, what passed, and what's left.

```
ANALYZE  ->  PLAN  ->  SCORE (rri.py)
                           |
              .------------+------------.
              |            |            |
           RRI 0-25    RRI 26-70    RRI > 70
              |            |            |
          auto-exec    present &     replan:
          no review    wait for    split until
                       approval     RRI <= 55
              |            |            |
              '------------+------------'
                           |
                       IMPLEMENT
                     (one task at a time)
                           |
                     MARK PROGRESS
                      (task ledger)
```

The RRI score at step 3 is what governs the rest. At RRI 0–25 the agent auto-executes: it shows what it's about to do and starts immediately. At RRI 26 and above, it presents the plan and waits. The human doesn't review every task — only the ones the score says deserve it.

This is the HITL model that actually scales. The human isn't approving every action; they're the final authority on high-complexity work, with the score making it unambiguous which work that is. Low-risk tasks flow through without friction. High-risk tasks surface to a human before any damage can be done. The agent never decides unilaterally which category a task falls into — the score does.

## The Mechanics of RRI

The whole point is to move from "trust the model" to "measure the risk." A weighted formula produces a score between 0 and 100, and a set of penalties pushes it higher when specific complexity conditions stack up.

The formula is:

```
RRI = 100 × ((0.18·C + 0.12·F + 0.15·D + 0.15·T + 0.12·A + 0.12·K + 0.10·P + 0.06·X) / 5) + Penalties
```

The formula looks heavier than it is. What matters is that the eight variables split into two kinds: **Objective** ones that tools measure directly, and **Rubric-Anchored** ones that stay subjective but get a minimum floor based on which file the task touches.

```
Var | Name                  | Nature         | Measurement Strategy
----|-----------------------|----------------|-----------------------------------
 C  | Cyclomatic Complexity | Objective      | Tools like radon or gocyclo
 F  | Files Affected        | Objective      | git diff / file path analysis
 D  | Domain Complexity     | Rubric-Anchored| Layer touched (auth/migrations vs UI)
 T  | Test-Coverage Risk    | Semi-Objective | Existing test density in the area
 A  | Task Ambiguity        | Subjective     | Clarity of acceptance criteria
 K  | Coupling/Side Effects | Rubric-Anchored| Impact on downstream dependencies
 P  | Public API / Security | Rubric-Anchored| Blast radius of a failure
 X  | Context Size          | Subjective     | Codebase context the agent must hold
```

Four natures, each with a different level of trust:

- **Objective** — measured by tools, not estimated. The agent has no input here.
- **Semi-Objective** — derived from the codebase (test density, coverage reports), but requires some interpretation.
- **Rubric-Anchored** — subjective in principle, but bounded by a policy map. The agent can score higher than the floor, never lower.
- **Subjective** — pure agent judgment. Ambiguity and context size can't be automated; the agent owns these.

That split is the part I care about most. C and F are facts read straight from the filesystem — the agent can't argue with them. D, K, and P are judgment calls, so I anchor them to a rubric instead of trusting the agent's gut. Any file path flagged as a security boundary — auth, crypto, migrations — carries a mandatory minimum score. The agent can rate it higher, never lower. That floor is what stops an agent from talking itself into treating a security-critical change as "simple."

The base score alone doesn't capture everything. Some combinations demand more caution and oversight regardless of how the individual variables score — a refactor that also changes behavior, a security change with no tests, a task with no verification strategy. For those, RRI adds fixed penalty points on top of the base. Each penalty applies independently and only once, and they can push the score above 100:

```
Condition                                          | +Points
---------------------------------------------------|--------
Refactor + behavior change in same task            |   +8
No tests and high security/data impact (P ≥ 4)     |  +10
High complexity and high domain (C ≥ 4 and D ≥ 3)  |  +10
Task touches auth, authz, permissions, sensitive   |  +10
More than 10 files affected (F ≥ 4)                |   +8
Architecture or policy decision required           |  +12
No verification strategy exists                    |  +15
```

## From Score to Cost

The number on its own is useless. What makes RRI work is that each score maps to a band, and each band defines three things: the gate the agent must clear, the model tier it runs on, and how much human time the task consumes.

```
RRI     | Label     | Gate                                       | Model tier | Human cost
--------|-----------|---------------------------------------------|------------|------------
0–25    | Low       | Auto-execute. No approval needed.           | Economy    | None
26–40   | Moderate  | Confirm tests exist in the affected area.   | Economy    | Minimal
41–55   | Med-high  | Plan + explicit acceptance criteria.        | Balanced   | Review plan
56–70   | Complex   | Human reviews the plan before impl.         | Balanced   | Review plan + diff
71–85   | High      | Char. tests + human reviews the diff.       | Premium    | Review diff
86–100  | Very high | ADR + risk analysis + decompose first.      | Premium    | Full review
>100    | Excessive | Stop. Architecture work happens first.      | Premium    | Architecture session
```

(ADR = Architecture Decision Record.)

This is where complexity becomes cost. A task at RRI 10 is almost free: a cheap model, no human in the loop, done in seconds. A task at RRI 80 costs a premium model run, a design document, characterization tests, and at least one human review session. Sum the RRIs across a project and you have a rough but honest estimate of its effort — before a single line of code is written.

The band isn't a suggestion. At RRI 80 the agent stops and decomposes before touching anything. At RRI 10 it proceeds immediately. The score determines the workflow, not the agent's mood.

## The Scripted Reality

My first instinct was to let the agent score itself. That was a mistake — LLMs don't calculate, they approximate. Ask a model to rate the risk of a change and it will give you a number that sounds plausible, changes between runs, and has no accountability. Two agents scoring the same task on the same codebase will give you different numbers. That's not a risk metric, it's a guess with extra steps.

The other obvious path — MCP servers, external APIs, cloud-based scoring services — trades one problem for another. Now your risk assessment depends on a network call, an external service staying up, and a credential that can expire. A tool that's supposed to tell you whether it's safe to proceed shouldn't itself be a dependency that can fail.

So I took the scoring away from the agent entirely and put it in a script that runs locally, needs no network, and is deterministic by construction: same inputs, same output, every time. A deterministic script (`scripts/rri.py`) does the math, the Cyclomatic Complexity (CC) analysis, and the rubric enforcement. The agent only fills in the genuinely subjective inputs it can't avoid — T (test coverage) and A (ambiguity).

Here's what it looks like on a real task — modifying an authentication library in a Rust project ([dubbridge](https://github.com/krukmat/dubbridge)). The agent supplies its best guesses for the subjective variables; the script handles everything else:

```bash
python3 scripts/rri.py \
  --auto-cc \              # measure C automatically using the ecosystem's tool
  --touches src/auth/lib.rs \
  --D 2 --K 2 --P 0 \
  --T 2 --A 1 --X 2
```

The script processes the input and produces a structured output. Each row shows the variable, the final score after rubric floors are applied, the evidence trail explaining how that score was derived, and a confidence level — High when a tool measured it, High but auditable when the agent supplied it:

```
Platform: rust (auto-detected)

Var   Score  Source
---   -----  ------
C       1    cargo clippy (max CC 8)
F       0    1 file touched
D       4    rubric floor: auth boundary
T       2    agent-supplied
A       1    agent-supplied
K       4    rubric floor: auth boundary
P       4    rubric floor: auth boundary
X       2    agent-supplied

Base:     42   (weighted sum)
Penalty: +10   (auth boundary, P >= 4)
RRI:      52   Med-high -> plan required before implementation
```

Look at what happened: the agent reported low values for D, K, and P — it genuinely thought this was a minor change. But the file was inside an auth boundary, so the script raised all three to their floor and the task landed in "Med-high." The *location* of the change outvoted the agent's read of it. That's the whole idea. The agent doesn't get to decide that auth code is simple.

The output also tells you the cost: Effort L, a Balanced-to-Premium model, a plan required before implementation. That's one task. Multiply that across a sprint and you have something you can actually plan and budget around — not a gut feeling, not a velocity metric that only makes sense in retrospect.

## Built to Travel

From the start I wanted RRI to work on any project, not just this one. The formula, the weights, the penalties, the bands — none of that is specific to Rust or to dubbridge. Complexity is complexity. Risk is risk. The math doesn't care what language you're writing in.

So the design separates what's universal from what's project-specific. The formula never changes. The two things that do change per ecosystem are the CC measurer — the tool that reads code complexity — and the anchor rubric — the map of which paths in your project are sensitive. Everything else is shared.

In practice: drop a `Cargo.toml` in your repo and the script detects Rust and runs `cargo clippy`. Drop a `go.mod` and it switches to `gocyclo`. `package.json` triggers `eslint`. `pyproject.toml` triggers `radon`. No configuration needed — the script walks up the directory tree and picks the right profile automatically. A `--platform` flag overrides it when needed.

The rubric works the same way. A built-in generic rubric already raises floors for paths like `**/auth/**`, `**/migrations/**`, and `**/crypto/**` — conventions that hold across most projects. Your team layers project-specific rules on top, anchored to your own architecture decisions. A new project gets sensible defaults out of the box; a mature one gets the full ADR-backed policy.

The result is a tool any team can adopt without rewriting the core. Same score, same bands, same cost model — just pointed at a different codebase.

## What Actually Changed

What changed most for me wasn't the agent — it was my own role. I stopped babysitting every diff. Now I only get pulled in when the score says the change is worth my attention; below that line, the agent just works and I don't watch. My time goes to the handful of high-RRI tasks that actually deserve a human, instead of being spread thin across everything.

The agent stopped being a yes-machine too. On a high score it slows down on its own — decomposes the task, writes the plan, asks before touching the architecture. And that decomposition matters beyond safety: a task split from RRI 80 into three pieces under 55 isn't just safer, it's cheaper. Less model cost, less review time, less surface area for technical debt to accumulate unnoticed. Complexity is where debt hides, and RRI makes complexity visible before you commit to it.

The deeper shift is that effort estimation stopped being a guess. Before RRI, "how long will this take?" was answered with experience, instinct, and optimism. Now it's answered with a score. Sum the RRI of every task in a project and you have a rough but honest picture of its cost — model tiers, human review sessions, architecture work included. That's not perfect project management, but it's a lot more honest than what we had before.

"Be careful with auth" was never a real safeguard. It lived in memory and good intentions, and both fail under pressure. A number doesn't. Once complexity is something you can measure, the guardrail stops depending on anyone remembering it — and the cost of a project stops being a surprise.

---

The full implementation — `scripts/rri.py`, the anchor rubric, platform profiles, and the policy — lives in [github.com/krukmat/dubbridge](https://github.com/krukmat/dubbridge).
