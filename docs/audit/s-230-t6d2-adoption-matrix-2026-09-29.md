# S-230-T6d.2 — Adoption compatibility matrix

Date: 2026-09-29
Branch: `main`
Status: ACTIVE — READY FOR EXECUTION
Cloud mutation: **NONE**

## Purpose

Convert the frozen T6d.1 inventory into a deterministic resource-by-resource
classification:

- `IMPORT` — an existing resource is structurally compatible with the frozen
  OpenTofu contract;
- `CREATE` — a required resource is unequivocally absent;
- `BLOCKED` — identity is ambiguous or compatibility would require an unsafe
  or owner-reviewed mutation.

## Command

```bash
make do-adoption-matrix
```

Input:

```text
/tmp/dubbridge-t6d1-inventory/
```

Output:

```text
/tmp/dubbridge-t6d1-inventory/adoption-matrix.json
```

Expected terminal shape:

```text
T6D2_RESOURCE=<name> classification=<IMPORT|CREATE|BLOCKED> [id=<id>]
...
T6D2_SUMMARY=import:<n>,create:<n>,blocked:<n>
T6D2_MATRIX_FILE=/tmp/dubbridge-t6d1-inventory/adoption-matrix.json
T6D2_MATRIX=PASS
```

A `BLOCKED` row exits non-zero and prevents T6d.3/T6d.4 progression.

## Safety boundary

This task performs no Digital Ocean API mutation and no OpenTofu import/apply.
It only consumes the sanitized T6d.1 evidence.
