# Integration Tests

The workspace root is not a package, so runnable integration suites live under the
owning app or crate instead of `tests/` at the repository root.

Current examples:
- `apps/api/tests/ingestion_test.rs` — S1 HTTP + DB ingestion acceptance tests

DB-backed integration tests use `DATABASE_URL` as the execution gate.
