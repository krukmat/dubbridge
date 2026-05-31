// T1-T5: crates/audit is a reserved namespace crate — it is NOT the active audit
// implementation.
//
// Real audit architecture (as of S1 / T1):
//   - Types:      crates/domain/src/audit.rs  (AuditEvent, AuditEventKind)
//   - DB writes:  crates/db/src/audit_repo.rs (insert_audit_event, backed by
//                 the `audit_events` PostgreSQL table)
//
// This crate is intentionally empty until S2+ introduces an out-of-process
// audit sink (e.g. S3 log shipping, event-stream fan-out).  When that work
// begins, re-export or wrap the domain types here so callers have a single
// import path.
//
// Do NOT add logic here that duplicates crates/domain or crates/db.
