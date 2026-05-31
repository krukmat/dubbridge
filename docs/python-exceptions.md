# Python Exception Boundary

DubBridge is intentionally Rust-first. Python exists only where established machine-learning tooling makes a full Rust replacement disproportionately expensive or less maintainable.

## Allowed worker categories

- ASR and alignment
- Translation
- TTS and dubbing

## Constraints

- Python workers must remain isolated behind explicit JSON schemas.
- Business rules, rights checks, consent checks, and publication gates stay in Rust.
- Python workers should be replaceable without changing the core domain model.

## Operational model

Each Python worker directory includes:

- `input.schema.json`
- `output.schema.json`
- `error.schema.json`
- `Dockerfile`
- `README.md`

This keeps the contract stable while allowing implementation changes behind the worker boundary.
