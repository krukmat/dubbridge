---
type: TaskList
title: "Tasks: resolve_rri_table's path-vs-text ambiguity"
description: "A nonexistent RRI-table path, or a directory, silently becomes literal table text instead of a diagnosable error; disambiguate the CLI contract."
status: active
slice: rri-table-path-text-ambiguity
---

# Tasks: `resolve_rri_table`'s Path-vs-Text Ambiguity

> **Origin:** filed by `docs/tasks/med-high-escalation-bundle-crash.md` T5,
> per the "Scope boundary on `resolve_rri_table`" section of
> `docs/plan/med-high-escalation-bundle-crash.md`, which deliberately left
> this out of scope there.

## Context

`escalation_packet.resolve_rri_table` (`scripts/local-agent/escalation_packet.py`)
accepts a value documented as "a path to a markdown file **or the table text
itself**":

```python
def resolve_rri_table(rri_table_arg):
    if not rri_table_arg:
        return None
    if not os.path.isfile(rri_table_arg):
        return rri_table_arg
    ...
```

A nonexistent path, or a path to a directory (`os.path.isfile` is `False` for
both), silently falls into the `return rri_table_arg` branch and is treated
as literal table text — in the directory case, the directory path string
itself becomes the rendered "RRI table". Neither case ever raises, so neither
destroys an escalation bundle (that failure mode — a read failure on a path
that *does* resolve to a regular file — was fixed separately in
`med-high-escalation-bundle-crash` T1). This ticket is about output fidelity,
not evidence loss: a user who mistypes `--rri-table` gets a bundle with a
nonsense "RRI table" section instead of a diagnosable error.

## Out of scope carried over

The read-failure case (an existing regular file that cannot be read or
decoded) is already fixed and is not reopened here.

## T1 — Disambiguate the CLI contract

- **Status:** [ ] Pending
- **Type:** development
- **Depends on:** —

### Goal

Decide and implement one of: (a) require `--rri-table` to always be a path
(breaking the "or literal text" convenience), (b) add a separate
`--rri-table-text` flag so the two input modes are never ambiguous, or (c)
keep both modes but make a nonexistent path fail loudly instead of silently
becoming text. Pick (c) unless a maintainer has a reason to prefer (a) or
(b) — it preserves the existing convenience for genuine literal-text callers
while turning a likely typo into a visible error.

### Acceptance criteria

- The chosen behavior is documented in `escalation_packet.py`'s
  `--rri-table` `--help` text.
- A path that looks like a path (contains `/` or ends in a known
  markdown/text extension) but does not exist is distinguished from a
  genuine literal-text argument, OR the ambiguity is removed entirely via a
  second flag.
- Existing callers (`run_med_high_task.py`, any CLI invocation in scripts or
  docs) are checked for reliance on the current silent-fallback behavior
  before the fix lands.
- `escalation_packet_test.py`'s existing
  `RriTableAsInlineStringVsFile`/`EC6RriTableUnreadableExistingFile` tests
  continue to pass unmodified, or are updated with an explicit note if the
  chosen fix changes their expected behavior.

### Evidence to emit

Unit tests for the new disambiguation behavior, plus a regression test
proving genuine literal-text `--rri-table` arguments still work.

## Out of scope

- The read-failure case on an existing regular file (fixed in
  `med-high-escalation-bundle-crash` T1).
- Any other `escalation_packet.py`/`run_med_high_task.py` behavior not
  related to `--rri-table` argument resolution.
