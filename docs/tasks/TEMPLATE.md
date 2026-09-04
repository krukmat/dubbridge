# Development task template

Use this template for new development ledgers. Add normal OKF frontmatter for the specific task file and include:

```yaml
Behavioral coverage contract: behavior-v2
```

For each development task define before implementation:

- acceptance criteria;
- at least one stable `HP-#` happy path;
- at least one stable `EC-#` edge/failure path;
- expected evidence and status artifacts.

Before marking the task Done, add:

```md
### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | ... | unit / component / integration / contract / e2e | `path::selector` | passed |
| EC-1 | Edge case | ... | unit / component / integration / contract / e2e | `path::selector` | passed |
```

For reproducible defects, add the failing regression test before the fix. BDD `.feature` work is required only when the behavior is a stable product/domain contract; do not create Gherkin for trivial implementation details.
