# Architecture Decision Records (ADR)

This directory holds the Architecture Decision Records that govern the DubBridge
platform. Each ADR captures one significant, hard-to-reverse decision: its
context, the decision itself, the consequences, and the alternatives that were
rejected.

## Format

ADRs follow a lightweight MADR-style structure:

- **Status** — `Proposed`, `Accepted`, `Superseded by ADR-XXX`, or `Deprecated`.
- **Context** — the forces at play and why a decision is required.
- **Decision** — what we decided and the precise scope of that decision.
- **Consequences** — positive, negative, and neutral effects.
- **Alternatives considered** — options rejected and why.

## Index

Keep this index synchronized with ADR file status changes and related canonical-doc
updates per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` ("ADR change propagation").
Run `make qa-docs-review` before reporting an ADR change complete (includes
the Gemma Reviewer pass; plain `make qa-docs` only runs the deterministic
doc gates).

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-006](ADR-006-postgres-metadata-object-storage-binaries.md) | PostgreSQL for metadata, object storage for binary artifacts | Accepted |
| [ADR-008](ADR-008-rights-ledger-fail-closed-precondition.md) | Rights ledger is a mandatory, fail-closed precondition | Accepted |
| [ADR-018](ADR-018-structured-observability-traceable-events.md) | Structured observability; every event must be traceable | Accepted |
| [ADR-019](ADR-019-stream-recording-engine-ffmpeg-subprocess.md) | Stream recording engine: FFmpeg subprocess orchestration | Proposed (scope: S3b live recording) |
| [ADR-020](ADR-020-recording-session-lifecycle-and-segment-model.md) | Recording session lifecycle and segment model | Accepted (scope: S3b live recording) |
| [ADR-021](ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md) | Intake-to-asset ingestion bridge with fail-closed rights (generalized) | Accepted |
| [ADR-022](ADR-022-source-protocol-support-and-ingest-authentication.md) | Source protocol support (RTMP + SRT) and ingest authentication | Proposed (scope: S3b live recording) |
| [ADR-023](ADR-023-api-client-authentication-and-principal-propagation.md) | API client authentication and principal propagation | Superseded by ADR-031 |
| [ADR-024](ADR-024-low-friction-first-party-api-access-via-session-gateway.md) | Low-friction first-party API access via session gateway | Superseded by ADR-031 |
| [ADR-025](ADR-025-platform-connector-ingest-and-owner-authorized-credentials.md) | Platform connector ingest and owner-authorized credential model | Proposed (primary S3 intake) |
| [ADR-026](ADR-026-layered-fail-closed-configuration-and-environment-separation.md) | Layered fail-closed configuration and environment separation | Proposed (scope: P0 environment separation) |
| [ADR-027](ADR-027-org-membership-authorization.md) | Organization membership authorization | Accepted |
| [ADR-028](ADR-028-voice-consent-ledger.md) | Voice-consent ledger and fail-closed TTS precondition | Accepted |
| [ADR-029](ADR-029-mobile-as-sole-authenticated-product-surface.md) | Mobile as the sole authenticated product surface | Accepted |
| [ADR-030](ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md) | Review-decision ledger and fail-closed publication gate | Accepted |
| [ADR-031](ADR-031-mobile-jwt-credential-auth-fenix-parity.md) | Mobile credential login with backend-issued JWT (FenixCRM parity) | Accepted (supersedes ADR-023/024; amends ADR-029; slice S-200) |
| [ADR-032](ADR-032-hls-playback-delivery-boundary.md) | HLS playback delivery boundary | Accepted |
| [ADR-033](ADR-033-open-knowledge-format-adoption.md) | Adopt the Open Knowledge Format (OKF) for repository knowledge | Accepted |
| [ADR-034](ADR-034-gemma-process-audit-and-reviewer-reconciliation.md) | Gemma process audit log and reviewer multi-pass reconciliation contract | Accepted (scope: gemma-audit-and-triple-pass slice) |
| [ADR-035](ADR-035-mobile-dark-theme-netflix-style.md) | Mobile dark-theme visual identity — Netflix-style dark canvas | Accepted |
| [ADR-036](ADR-036-local-first-agentic-implementation-band.md) | Local-first agentic implementation band (RRI 26–40) and Apple Silicon local model stack | Accepted (scope: agent workflow / local delegation) |
| [ADR-037](ADR-037-qwen36-27b-local-architect-complex-analyst.md) | Qwen3.6-27B as Local Architect and Complex Analyst | Accepted (scope: direct project advisory analysis) |
| [ADR-038](ADR-038-med-high-architect-refined-single-attempt.md) | Architect-refined single local attempt for Med-high tasks | Accepted (amends ADR-036/ADR-037; agent workflow only) |
| [ADR-039](ADR-039-human-selected-fallback-model-checkpoint.md) | Human-selected fallback model checkpoint | Accepted (amends ADR-034/036/038; agent workflow only) |
| [ADR-040](ADR-040-per-module-complexity-split-implementation-routing.md) | Per-module complexity-split implementation routing (RRI 26-55) | Accepted (amends ADR-036/ADR-038; agent workflow only) |
| [ADR-041](ADR-041-pre-approval-med-high-decomposition-local-favoring-granularity.md) | Pre-approval Med-high task decomposition for local-favoring granularity | Proposed (amends RRI_POLICY.md § Decomposition triggers if accepted; agent workflow only) |
| [ADR-042](ADR-042-push-review-remediation-controller-and-escalation-lifecycle.md) | Push-review remediation controller and bounded escalation lifecycle | Proposed (scope: X27; amends ADR-034/ADR-039 if accepted; agent workflow only) |
| [ADR-043](ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md) | Mobile P2P runtime ownership and proof isolation | Accepted (scope: MVP0-P2P P1; does not decide audience delivery) |
| [ADR-044](ADR-044-p2p-audience-delivery-boundary.md) | P2P audience delivery boundary | Proposed (D1 `O3 parallel` and D2 `K1` resolved 2026-09-05; D3 and acceptance still block P2; does not replace ADR-032) |

## Backfill note

ADR-006, ADR-008, and ADR-018 were referenced by `docs/plan/s1-asset-ingestion-rights-ledger.md`
and by the SQL migrations under `infra/migrations/` before any ADR file existed in
the repository. They have been reconstructed here from the implemented behavior of
the S1 slice so that the references resolve and the decisions are auditable. If the
original intent differs from the implemented behavior, update these files and record
the divergence.

The numbering gaps (ADR-001..005, 007, 009..017) are intentionally left open for
decisions that predate or are unrelated to the slices currently in the repository.
