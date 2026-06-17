# Plan: S-080 — Object Storage Switchover

> **Status:** In progress. Authored 2026-06-17 after roadmap review and approved
> decomposition. Tasks T0-T5d are complete; T6 final verification/docs sync is
> currently blocked by the docs QA gate in the existing dirty worktree.
> **Roadmap phase:** `S-080` — object storage switchover behind `StorageAdapter`.
> **Tasks ledger:** `docs/tasks/s-080-object-storage-switchover.md`.

## Purpose

DubBridge already separates relational truth from binary storage: PostgreSQL holds
state, rights, and audit history, while `crates/storage` holds binary artifacts behind
`StorageAdapter`. The missing piece is that the runtime still behaves like a
development install: `build_adapter()` always returns `LocalFsAdapter`, upload keys are
still shaped at the API boundary, and the HTTP upload path buffers the full multipart
file into memory before storing it.

`S-080` turns that boundary into the production-like storage path the roadmap expects:
S3-compatible object storage, canonical storage-owned keys, bounded-memory upload
behavior, and recovery rules when object writes and relational writes diverge.

## Objective

Deliver a production-like object-storage path behind `StorageAdapter` that:

- selects between local fs and S3-compatible storage via config;
- stores binaries in MinIO/S3 with canonical keys owned by `crates/storage`;
- avoids whole-file buffering in API memory for large uploads;
- defines and implements orphan cleanup / reconciliation behavior; and
- preserves the existing Postgres-owned metadata, rights, and audit invariants.

This slice closes roadmap `X9`, unblocks `S-120`, and reduces rework risk before
resuming `S-090`.

## Scope

### Included

- A concrete S3-compatible adapter in `crates/storage` validated against local MinIO.
- Config-driven backend selection through the existing `storage.backend` setting.
- Canonical storage-owned key construction for upload and derived-artifact paths.
- A storage API shape that can write without materializing an entire client upload into
  a single `Vec<u8>` in the API process.
- Internal object URL / object reference behavior appropriate for non-public binaries.
- Cleanup semantics for object-write success + metadata-write failure, plus explicit
  reconciliation for orphaned objects.
- Local infrastructure parity for the S3-compatible path in `infra/local/docker-compose.yml`.
- Focused tests for adapter behavior, selector wiring, key layout, and failure cleanup.

### Excluded

- CDN/public delivery, signed client download URLs, or publication distribution.
- Owner-credential secret-store design (`X20`), which belongs to `S-090`.
- Media preparation, probing, transcoding, ASR, subtitle generation, or dubbing.
- A full provider decision between AWS S3, R2, Wasabi, or other production targets.
  `S-080` implements against an S3-compatible contract and validates locally with MinIO.

## Governing ADRs and roadmap constraints

- ADR-006: PostgreSQL metadata + object storage for binaries.
- ADR-018: structured observability for storage-side failures and reconciliation logs.
- ADR-021: the finalize path is producer-agnostic and expects durable binary storage.
- ADR-025: platform ingest will reuse this storage path for downloaded binaries.
- ADR-026: backend selection is env-driven; secrets remain outside committed profiles.
- Roadmap `X9`: object-store adapter, canonical keys, orphan reconciliation, and a
  streaming/presigned strategy that avoids buffering large uploads in API memory.

## Affected components

| Layer | Path | Change |
|---|---|---|
| Storage abstraction | `crates/storage/src/adapter.rs` | extend the trait contract for production-scale writes |
| Storage implementation | `crates/storage/src/local.rs` | keep local parity with any trait changes |
| Storage implementation | `crates/storage/src/s3.rs` (new) | MinIO/S3-compatible adapter |
| Storage selection | `crates/storage/src/lib.rs` | choose backend from `StorageConfig` |
| Storage config | `crates/storage/src/config.rs` | map typed settings needed by the S3 adapter |
| Runtime config | `crates/config/src/lib.rs` | validate S3-oriented settings where needed |
| API upload path | `apps/api/src/routes/ingestion.rs` | stop buffering the whole multipart file into one `Vec<u8>` |
| Runtime wiring | `apps/api/src/main.rs`, `apps/worker-runner/src/main.rs` | construct the configured backend |
| Local infra | `infra/local/docker-compose.yml` | local MinIO path and env wiring stay usable |
| Docs / QA | `README.md`, `docs/architecture.md`, roadmap/task docs | document the storage path and verification |

## Design decisions

### D1 — S3-compatible contract, MinIO local validation

The storage backend contract is S3-compatible, not AWS-specific. Local and CI
validation use MinIO. This keeps the implementation maintainable and portable while
preserving an obvious production path to AWS S3 or another S3-compatible provider.

### D2 — `crates/storage` owns canonical keys

API routes and workers should not hand-roll object keys. Key layout belongs to
`crates/storage` so every producer writes to one authoritative scheme (`assets/...`,
derived artifacts, future platform-ingest artifacts). This closes the remaining ADR-006
implementation note that key ownership still lives too high in the stack.

### D3 — Upload API must stop whole-file buffering

Today `create_ingestion()` reads the multipart file into memory and only then calls
`storage.put(...)`. `S-080` changes the storage write path so large uploads can be
spooled or streamed through a bounded-memory path before object storage persistence.
The exact mechanism is implemented in tasks, but the slice requirement is clear:
production-scale uploads must not require one full in-memory `Vec<u8>` in the API
process.

### D4 — Object write first, metadata second, with explicit cleanup

The system keeps the existing safety posture: write the object, then persist metadata.
If relational persistence fails, the runtime attempts immediate object cleanup and logs
any cleanup failure. Because cross-store writes are not atomic, the slice also includes
an explicit orphan-reconciliation path for leftovers that survive immediate cleanup.

### D5 — Internal references, not public storage URLs

`object_url()` remains an internal reference helper, not a publication mechanism. The
app should not accidentally leak provider-specific public URLs into API responses that
later become product contracts. `S-180` can define publication delivery separately.

### D6 — Keep the rest of the app storage-agnostic

`apps/api`, workers, and downstream pipeline stages continue to depend on
`StorageAdapter`, not on MinIO/AWS SDK details. `S-080` strengthens the boundary
instead of leaking provider logic upward.

## Task decomposition strategy

The approved presentation computed `RRI 77` for the slice, which mandates
decomposition before implementation. This plan therefore splits the work into bounded
tasks:

1. contract and key ownership;
2. S3-compatible adapter implementation;
3. runtime selection and local parity;
4. bounded-memory upload path;
5. immediate cleanup regression coverage and orphan observability;
6. storage-side candidate listing seam for reconciliation;
7. reconciliation planning against relational references;
8. reconciliation execution plus operator docs;
9. verification and status sync.

No code implementation should start outside those decomposed tasks.

## Relationship to adjacent slices

- **Built on:** `S-010` (`StorageAdapter`, upload/finalize flow), `S-020/H1`
  (atomic relational hardening), `S-030` (config-driven backend selector).
- **Unblocks directly:** `S-120` media preparation, which depends on `S-080`.
- **Reduces rework for:** `S-090`, because platform ingest becomes the first sustained
  high-volume writer and should target production-like storage.
- **Unaffected for now:** `S-160` review/publication product surface, which already
  works on relational artifacts and fixtures.

## Open follow-ups

- Provider selection for production deployment remains open: AWS S3, Cloudflare R2,
  Wasabi, or another S3-compatible backend can be chosen later without changing the
  application contract.
- Client-facing signed upload/download flows may become necessary later, but they are
  not required to unblock `S-120`.
- `S-090` still owns the owner-credential secret-store ADR (`X20`); `S-080` must not
  solve that problem by smuggling provider secrets into committed config.
