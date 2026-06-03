# config/ — Environment profiles

Non-secret, environment-specific configuration for DubBridge.
Governed by ADR-026: Layered fail-closed configuration and environment separation.

## What lives here vs in environment variables

| Kind | Where |
|---|---|
| Non-secret defaults valid everywhere | `default.toml` |
| Non-secret local-dev values | `local.toml` |
| Non-secret staging values | `staging.toml` |
| Non-secret production values | `production.toml` |
| Secrets and per-deploy values | Injected `DUBBRIDGE_*` env vars (never committed) |

## Layered resolution order (lowest → highest precedence)

```
default.toml  ←  <env>.toml  ←  DUBBRIDGE_* env vars
```

An env var always wins. Use env vars for secrets and per-deploy overrides.

## Variable × environment parity table

| Variable | default | local | staging | production | Env var override |
|---|---|---|---|---|---|
| `env` | — | `local` | `staging` | `production` | `DUBBRIDGE_ENV` |
| `api_port` | `8080` | — | — | — | `DUBBRIDGE_API_PORT` |
| `worker_concurrency` | `4` | — | — | — | `DUBBRIDGE_WORKER_CONCURRENCY` |
| `database_url` | — | `postgres://…localhost…` | — (secret) | — (secret) | `DUBBRIDGE_DATABASE_URL` |
| `redis_url` | — | `redis://127.0.0.1:6379` | — (secret) | — (secret) | `DUBBRIDGE_REDIS_URL` |
| `storage.backend` | — | `local_fs` | `s3` | `s3` | `DUBBRIDGE_STORAGE_BACKEND` |
| `storage.base_path` | — | `/tmp/dubbridge-storage` | `""` | `""` | `DUBBRIDGE_STORAGE_BASE_PATH` |
| `storage.bucket` | — | `dubbridge-local` | `dubbridge-staging` | `dubbridge-production` | `DUBBRIDGE_STORAGE_BUCKET` |
| `storage.endpoint_url` | — | — | — | — | `DUBBRIDGE_STORAGE_ENDPOINT_URL` |
| `observability.log_format` | — | `pretty` | `json` | `json` | `DUBBRIDGE_OBSERVABILITY_LOG_FORMAT` |
| `observability.filter` | `info` | — | — | — | `DUBBRIDGE_OBSERVABILITY_FILTER` |
| `auth.issuer` | — | — (optional) | — (secret) | — (secret) | `DUBBRIDGE_AUTH_ISSUER` |
| `auth.audience` | — | — (optional) | — (secret) | — (secret) | `DUBBRIDGE_AUTH_AUDIENCE` |
| `auth.rsa_public_key_path` | — | — (optional) | — (secret) | — (secret) | `DUBBRIDGE_AUTH_RSA_PUBLIC_KEY_PATH` |
| `auth.clock_skew_leeway_seconds` | — | — | — | — | `DUBBRIDGE_AUTH_CLOCK_SKEW_LEEWAY_SECONDS` |

## DATABASE_URL alias rule (ADR-026 §2, F2)

`DATABASE_URL` is a **tooling alias only** — used by sqlx-cli and migration scripts.
The application and all its tests use `DUBBRIDGE_DATABASE_URL` as the single
authoritative name. Never read `DATABASE_URL` from application code.

## Adding a new variable

1. Add it to `AppConfig` (or the relevant sub-struct) in `crates/config/src/lib.rs`.
2. Add non-secret defaults to `default.toml` and/or the relevant `<env>.toml`.
3. Document it in the parity table above.
4. If it is a secret, add a `<REPLACE_ME>` entry to `/.env.example`.
