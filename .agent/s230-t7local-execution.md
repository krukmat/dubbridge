# S-230-T7local — compact execution packet

## Goal

Certify the **base S-230 mobile flow** against the local Docker Compose
**gateway** and emit durable evidence.

Scope:

```text
login
→ upload
→ rights
→ finalize
→ preparation
→ review
→ publish
→ normal HLS playback
```

Out of scope: My Content/Invite/Claim, P2P Sync/Verify, loopback P2P playback,
HPKE/Keystore certification and P5.T3.

Pinned DEV-HANDOFF: `84ea5edc` (P6 PASS, 15/15 CI).

## Minimal-context contract

For normal execution, read **this file only** before starting.

Do not read the full S-230 plan, roadmap, historical P6 evidence, ADRs, or old
audit artifacts unless a concrete failure requires them. Do not browse the web.

Only inspect source when a command or runtime behavior contradicts this packet:

- `infra/local/docker-compose.yml` — Compose/runtime mismatch;
- `infra/local/p2p/preflight.sh` — preflight mismatch;
- `mobile/app.config.ts` / `mobile/src/config/env.ts` — config mismatch;
- `Makefile.base` — QA target mismatch;
- `mobile/maestro/t7local/` — real-stack certification harness mismatch.

`DESIGN.md` is required only if a separately approved repair changes product
UI/source. T7local itself should not modify product source.

If a product defect is found, record **BLOCKED** and stop. Do not repair it
inside T7local.

## Execution map

```text
A Runtime
├─ A1 Compose + gateway
├─ A2 gateway live/ready
└─ A3 mobile gateway config

B Mobile
├─ B1 make qa-mobile
├─ B2 normal Android build/install/launch
└─ B3 real login/session

C Base E2E
├─ C1 upload + rights + finalize
├─ C2 preparation + artifacts
├─ C3 review + approved decision
└─ C4 publish + normal HLS playback

D Negative controls
├─ D1 missing gateway config → ConfigErrorScreen
└─ D2 rejected credentials → generic login error / remains unauthenticated

E Certification
├─ E1 downstream-state evidence
├─ E2 audit artifact + exact HEAD
├─ E3 PASS/BLOCKED
└─ E4 freshness vs 84ea5edc
```

Order: **A → B → C → D → E**.

Session-expiry behavior is intentionally **not D2**; it belongs to
`S-230-T7c`.

## Fast path

Run from repo root.

### A — runtime

```bash
git rev-parse HEAD
git diff --name-only 84ea5edc...HEAD

bash infra/local/p2p/generate-mtls.sh
mkdir -p tmp/p2p-ciphertext tmp/p2p-availability-drive tmp/p2p-availability-index

docker-compose -f infra/local/docker-compose.yml --profile app up -d
docker-compose -f infra/local/docker-compose.yml --profile app ps

bash infra/local/p2p/preflight.sh --with-runtime
curl -fsS http://localhost:8082/health/live
curl -fsS http://localhost:8082/health/ready
```

A3 contract:

```text
Android emulator → http://10.0.2.2:8082
physical Android → http://<host-LAN-IP>:8082
host :8080       → API only; never use as the mobile gateway
```

### B — mobile

Run the aggregate QA **once**; do not separately repeat typecheck/lint/Jest:

```bash
make qa-mobile
```

Build the normal app, not a P2P harness. The Android build must run on JDK 17;
verify the Gradle launcher/daemon JVM before debugging native build failures.

```bash
cd mobile
DUBBRIDGE_ENV=local \
EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082 \
npm run android
```

After B2, return to repo root. The canonical low-touch driver for **B3 through
C4** is:

```bash
bash mobile/maestro/t7local/run-real.sh
```

**Bootstrap prerequisite (verified 2026-09-25, HEAD `13c353b`):** keep Metro
running in a separate terminal throughout Maestro. If the build terminal has
exited, restart it from `mobile/`:

```bash
DUBBRIDGE_ENV=local \
EXPO_PUBLIC_E2E_ENABLED=false \
EXPO_PUBLIC_P2P_DEV_HARNESS=false \
EXPO_PUBLIC_P2P_VISUAL_FIXTURES=false \
EXPO_PUBLIC_P5_DEVICE_CERT_HARNESS=false \
EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082 \
npx expo start --lan --port 8081
```

Before launching Maestro, `curl -fsS http://127.0.0.1:8081/status` must return
`packager-status:running`; confirm the server belongs to this `mobile/` project.
The installed debug APK at this head does **not** include `expo-dev-client` or
`expo-dev-launcher`. Its normal `.MainActivity` launch loads Metro from the
Android emulator default `10.0.2.2:8081`, including after `clearState: true`.
An `expo-development-client` deep link reaching `.MainActivity` does not prove
that its nested URL was interpreted or that a bundle loaded. Do not install a
new native dependency just to work around a missing Metro process.

`adb reverse tcp:8081 tcp:8081` provides Android loopback transport, but does not
change this APK's Metro host selection. `--localhost` bound only to host `::1`
in the observed environment; the `--lan` start above restored emulator access.
Keep the gateway on `8082`. Verify the bundle load and `login-screen`, rather
than treating an `openLink` success as bootstrap evidence.

**Current stop point:** bootstrap passes; B3 Maestro submit remains blocked.
The 2026-09-25 diagnostic reached `home-screen` through real authentication
after a normal ADB UI tap, but Maestro's submit tap left `Login phase: idle`;
`retryTapIfNoChange` also failed. Until B3 passes automatically, run only the
bootstrap/login prefix, not the full B3–C4 driver above. See
`docs/audit/s-230-t7local-2026-09-25.md` for exact evidence and limitations.

The runner uses the real gateway, a real account, supported workspace/project
APIs, a fresh local MP4, the normal mobile UI, and read-only PostgreSQL probes.
It must not use the mock gateway, `/e2e/seed`, seeded IDs, or the E2E upload
fast-path. It fails closed if the normal backend does not produce the expected
durable state, including a real review task. Its `/tmp` summary is evidence
input only; it does not certify T7local or edit status documents.

If running B3-C4 manually instead, a missing local account may be created
through the supported gateway endpoint `POST /auth/register` on
`localhost:8082`. Never write a password or returned bearer token into
evidence.

### C — one fresh asset

Use one short local MP4. Do not download media from the internet. If no suitable
fixture exists, generate a tiny synthetic clip locally and push it to the
emulator/device.

Drive the normal UI through C1–C4. Capture the `asset_id` once and reuse it.

Do not accept HTTP 2xx as evidence. Verify downstream state directly. The
relevant PostgreSQL tables are:

```text
assets
rights_records
artifact_records
asset_preparation_status
review_tasks
review_decisions
publications
audit_events
```

Use:

```bash
docker-compose -f infra/local/docker-compose.yml exec -T postgres \
  psql -U dubbridge -d dubbridge
```

Query only the rows for the current `asset_id` / review task. Do not dump whole
tables.

Minimum evidence:

- C1: asset finalized + rights row + source artifact;
- C2: preparation authoritative state + expected derived/HLS artifacts;
- C3: review task exists + latest decision is `approved`;
- C4: publication row is `published` + normal mobile HLS playback renders.

### D — negative controls

Do not rebuild twice just to prove negatives if canonical tests already prove
them.

`make qa-mobile` must include and pass the existing RootNavigator coverage for:

- missing gateway configuration → `config-error-screen`;
- login failure → generic `Invalid email or password.` while unauthenticated.

Record those test results as D1/D2 evidence. Only execute an extra device
negative if the canonical tests are missing or fail.

### E — certification

Create one task-scoped artifact:

```text
docs/audit/s-230-t7local-<YYYY-MM-DD>.md
```

It must contain only:

- exact HEAD;
- device/emulator;
- gateway target;
- A1–D2 PASS/BLOCKED results;
- asset/review/publication identifiers needed for traceability;
- concise downstream-state evidence;
- `make qa-mobile` result;
- no credentials, bearer tokens, raw secrets, CK/KEK material.

Then compute E4:

```bash
git diff --name-only 84ea5edc...HEAD
```

If there is no relevant runtime/config change, record:

```text
T7LOCAL_FRESHNESS=PASS_NO_RERUN
```

If relevant paths changed, first map them to the A–D evidence already executed
on this HEAD. Run only uncovered bounded regression; do not rerun P2P
certification.

Finally synchronize:

- `docs/tasks/s-230-poc-v1-digitalocean.md` T7local status;
- `docs/audit/go-live-octubre-2026-mirror.html`;
- roadmap only if the roadmap state actually changes.

Commit the evidence/status documentation once at the end. Do not create
intermediate docs-only commits unless execution is BLOCKED and the finding must
be preserved.

## PASS / BLOCKED rule

**PASS** only when A1–E4 are satisfied.

**BLOCKED** immediately when:

- gateway/Compose provenance is ambiguous;
- a base-flow stage returns success but downstream state does not advance;
- normal base flow requires a P2P-only action;
- product behavior needs a source-code repair;
- required device/runtime access is unavailable.

A blocker is evidence, not permission to expand scope.

## Final response

Keep the chat response compact:

```text
T7local: PASS | BLOCKED
HEAD:
A:
B:
C:
D:
E4:
artifact:
commit:
next: T7c | blocker
```
