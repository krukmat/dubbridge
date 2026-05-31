# Architecture Overview

DubBridge is a Rust-first platform for processing authorized audiovisual media into localized outputs.

## Core principles

- Rust owns the API surface, orchestration, persistence boundaries, governance rules, and quality gates.
- Python is isolated to worker implementations where the ML ecosystem justifies an exception.
- Media artifacts are treated as immutable records with explicit lineage.
- Publication remains blocked until rights, consent, processing, and quality checks succeed.

## Runtime surfaces

- `apps/api` exposes HTTP endpoints and operational health checks.
- `apps/worker-runner` executes background jobs and coordinates external media tools.
- `apps/cli` hosts local operational commands for development and administration.
- `workers/*-py` execute AI workloads behind typed input and output schemas.

## Shared crates

- `domain`: Core entities and invariants.
- `db`: SQLx persistence wiring and repositories.
- `storage`: Object storage abstractions and path conventions.
- `jobs`: Background job types and scheduling adapters.
- `media`: Media probing and process orchestration boundaries.
- `providers`: Worker and provider-facing contracts.
- `qc`: Deterministic quality checks.
- `auth`: Authentication and authorization policy boundaries.
- `audit`: Audit events and lineage metadata.
- `config`: Typed runtime configuration.
- `observability`: Logging, tracing, and health-reporting helpers.

## Local development topology

Local development uses PostgreSQL for primary state, Redis for job coordination, MinIO for object storage, and containerized service entrypoints where needed.

## API identity boundary

`apps/api` is an OAuth 2.0 resource server. Protected routes consume a verified
JWT bearer principal through `crates/auth`; handlers never trust caller-supplied
uploader identity. Live RTMP/SRT source credentials are a separate recording-edge
concern (ADR-022).
