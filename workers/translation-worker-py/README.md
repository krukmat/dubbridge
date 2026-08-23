# Translation Worker Contract

This directory holds the JSON contract for the subtitle translation worker
(the real Python implementation lands in `S-150-T3b`; this contract and its
Rust typed client, `crates/providers/src/translation.rs`, land in
`S-150-T3a`).

## Contract shape (D3, `schema_version: 1`)

- **Input** (`input.schema.json`): `job_id`, `source_language`,
  `target_language` (BCP-47 tags), and an ordered `segments` array. Each
  segment carries a stable `segment_id` — derived by the Rust caller from
  `(subtitle_artifact_id, zero_based_ordinal)`, never minted by the worker —
  plus `start_ms`, `end_ms`, and `source_text`.
- **Output** (`output.schema.json`): the same envelope shape with each
  segment's `segment_id`, `start_ms`, and `end_ms` preserved unchanged from
  the input, plus a new `translated_text` field and `status: "ok"`.
- **Error** (`error.schema.json`): `job_id`, `error_code`, `message` — no
  `segments` field, since a failed job has no partial output.

All three schemas set `additionalProperties: false`.

## Identity and timing rules

- `segment_id` is opaque to the worker: read it, echo it back unchanged.
- `start_ms`/`end_ms` must be echoed back unchanged. Translation never
  changes timing.
- The worker never sees the legacy S-140 bare-segment shape directly — the
  Rust input adapter (`translation::normalize_legacy_segments`) normalizes
  it into this versioned envelope first, rejecting missing, duplicated,
  reordered-without-identity, or timing-invalid segments before dispatch.

## Transport

- The worker communicates over stdin/stdout as a subprocess, matching the
  existing `workers/asr-worker-py` pattern. Worker-returned URIs (if any
  future revision of this contract adds them) are transport only — Rust
  reads/validates the referenced bytes and uploads them through
  `StorageAdapter` under storage-owned keys; a worker URI is never written
  directly to PostgreSQL as a canonical storage key (D4).
- Non-zero exit, malformed JSON, a missing output file, or a non-file URI
  must be signaled as the typed error envelope, not a partial success.
