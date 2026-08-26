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

For structured settings, `AppConfig::load()` uses `__` as the nesting separator, so
`auth.jwt_expiry_hours` maps to `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS`. The legacy
`AuthSettings::from_env()` helper still accepts flat aliases such as
`DUBBRIDGE_AUTH_JWT_SECRET`, `DUBBRIDGE_AUTH_JWT_EXPIRY_HOURS`, and
`DUBBRIDGE_AUTH_CLOCK_SKEW_LEEWAY_SECONDS` until that path is removed.

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
| `auth.issuer` | — | — (optional) | — (secret) | `https://poc.iotforce.es` | `DUBBRIDGE_AUTH__ISSUER` |
| `auth.audience` | — | — (optional) | — (secret) | `dubbridge-api` | `DUBBRIDGE_AUTH__AUDIENCE` |
| `auth.rsa_public_key_path` | — | — (optional) | — (secret) | `unused-legacy-field-hs256-only` (dead field, see note) | `DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH` |
| `auth.jwt_secret` | — | — (optional) | — (secret) | — (secret) | `DUBBRIDGE_AUTH__JWT_SECRET` |
| `auth.jwt_expiry_hours` | — | — (optional, default `24`) | — (secret/profile) | `8` (S-230-T5a frozen) | `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` |
| `auth.clock_skew_leeway_seconds` | — | — | — | `30` | `DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS` |
| `storage.access_key_id` | — | — | — (secret) | — (secret) | `DUBBRIDGE_STORAGE__ACCESS_KEY_ID` (double underscore; nested under `[storage]`) |
| `storage.secret_access_key` | — | — | — (secret) | — (secret) | `DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY` (double underscore; nested under `[storage]`) |
| `gateway.oauth.client_secret` | — | — (optional) | — (secret) | — (secret) | `DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET` |
| translation provider (worker, not `AppConfig`) | — | `fake` | — | `http` (S-230-T5a frozen) | `DUBBRIDGE_TRANSLATION_PROVIDER` |
| translation API URL/key (worker, not `AppConfig`) | — | — | — (secret) | — (secret) | `DUBBRIDGE_TRANSLATION_API_URL` / `DUBBRIDGE_TRANSLATION_API_KEY` |

**`auth.rsa_public_key_path` is a dead field:** structurally required by
`AuthSettings` but functionally unused since auth became HS256-only
(ADR-031/S-200, `crates/auth/src/issuer.rs`). `production.toml` sets a fixed
placeholder string rather than a real key path; removing this field entirely
is deferred cleanup, not S-230-T5b scope.

**Vars read outside `AppConfig`/figment:** `DUBBRIDGE_TRANSLATION_PROVIDER`,
`DUBBRIDGE_TRANSLATION_API_URL`, and `DUBBRIDGE_TRANSLATION_API_KEY` are read
directly via `os.environ` by `workers/translation-worker-py/main.py` — they
share the `DUBBRIDGE_` prefix by convention only and are never passed through
figment's `__` nesting. Similarly, `crates/config/src/lib.rs`'s deprecated
`AppConfig::from_env()` (Task 4 removal pending) reads single-underscore
names (`DUBBRIDGE_AUTH_ISSUER`, `DUBBRIDGE_STORAGE_ACCESS_KEY_ID`, etc.) that
look adjacent to the double-underscore names above but are dead for the
`AppConfig::load()` production boot path every service actually uses — do
not set the single-underscore forms expecting them to reach production.

## DATABASE_URL alias rule (ADR-026 §2, F2)

`DATABASE_URL` is a **tooling alias only** — used by sqlx-cli and migration scripts.
The application and all its tests use `DUBBRIDGE_DATABASE_URL` as the single
authoritative name. Never read `DATABASE_URL` from application code.

## Adding a new variable

1. Add it to `AppConfig` (or the relevant sub-struct) in `crates/config/src/lib.rs`.
2. Add non-secret defaults to `default.toml` and/or the relevant `<env>.toml`.
3. Document it in the parity table above.
4. If it is a secret, add a `<REPLACE_ME>` entry to `/.env.example`.
