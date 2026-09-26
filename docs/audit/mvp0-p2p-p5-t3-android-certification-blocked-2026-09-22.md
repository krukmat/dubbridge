---
type: Audit
title: "P5.T3 Android certification — BLOCKED, gateway-URL routing defect"
status: active
---

# P5.T3 Android certification — attempt record, 2026-09-22

**Result:** BLOCKED at CLAIM stage. Not PASS. Not eligible to advance P5 aggregate status.

## Environment

- Branch: `feature/p2p-mvp-core`
- HEAD: `584b22c2515d5120419c605d606a4ec05858f609`
- Device: Android emulator `emulator-5554`, `sdk_gphone64_arm64`, Android 14 (API 34), ABI `arm64-v8a`
- Backend: `infra/local/docker-compose.yml` local stack (`api`, `availability-node`, `worker-runner`, `postgres`, `redis`, `minio`) — **no `gateway` service in this Compose file**
- A separate standalone `dubbridge-gateway` process was found already running outside Compose (PID 51009, long-lived, started prior to this session) on port **8082**

## Preflight

`bash infra/local/p2p/preflight.sh --with-runtime` initially reported `FAIL` on the
`worker P2P publication flag` check. Root-caused as an **environmental/preflight-script
limitation, not a P2P capability defect**: the worker-runner container had been running
for 3 days and accumulated high-frequency `apalis_redis::storage: consumer not registered`
warnings (368 of the last 500 log lines at time of check), which scrolled the one-time
`p2p_publication_enabled=true` boot line out of the script's `docker-compose logs --tail=500`
window. `docker-compose restart` does not reset the log buffer (same container, Docker
retains history); `docker-compose up -d --force-recreate worker-runner` does. After
recreate, preflight passed cleanly: **PREFLIGHT: PASS**. No code was changed to reach
this state — only container lifecycle (recreate), which is standard local-dev housekeeping.

**Residual observation (not fixed, not in scope to fix under this task):** the preflight
script's log-tail-based check is fragile against any long-lived worker-runner container
once the Redis consumer-registration warning accumulates. Worth a follow-up task to either
reduce the warning's frequency or make the preflight check more robust (e.g. `--since`
with a bounded time window instead of a fixed line-count tail).

## Certification attempts

### Attempt 1 — default gateway URL (`http://10.0.2.2:8081`)

Launched via the playbook's exact documented command: `npm run android:p2p-cert`
(no env override). App opened directly to the P5 certification harness (a persisted
auth session from a prior manual login was present — `auth.status === "authed"`
gates the harness's render). Typed the invitation token from `tmp/p5-invitation-token.txt`
into the masked field, tapped "RUN P5 CERTIFICATION".

**Result: `CLAIM_FAILED`.** No corresponding request appeared in `apps/api` container
logs at all (checked `docker-compose logs api` around the attempt timestamp — zero P2P-
related lines, zero HTTP request evidence of any kind).

### Attempt 2 — corrected `apps/api` port (`http://10.0.2.2:8080`)

Hypothesis: `10.0.2.2:8081` is Metro Bundler's port (confirmed: `expo run:android`
itself logs `Waiting on http://localhost:8081`), not the backend. Relaunched via
`EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8080 npx expo run:android`
(env override supported natively by `mobile/app.config.ts`, not a code change).
Retried the identical claim.

**Result: `CLAIM_FAILED` again.** Still zero corresponding requests in `apps/api` logs.
This ruled out the simple port-typo hypothesis as sufficient.

### Root cause — client/server route-prefix mismatch, and no gateway in local Compose

- `mobile/src/api/p2p.ts` calls paths prefixed `/api/...` (e.g.
  `/api/p2p/invitations/claim`), consistent with the documented mobile-client
  contract of always talking to the gateway (ADR-031), never `apps/api` directly.
- `apps/gateway/src/lib.rs:28` — `.nest("/api", proxy_router())` — the gateway is
  the component that owns the `/api` prefix and strips it
  (`apps/gateway/src/proxy.rs:144-145`) before proxying to `apps/api`.
- `apps/api/src/lib.rs` merges `routes::p2p_audience::router()` etc. with **no**
  `/api` prefix — its real route is `/p2p/invitations/claim`.
- Verified directly:
  - `curl -X POST http://localhost:8080/p2p/invitations/claim` → `401` (route
    exists on `apps/api`, unauthenticated as expected).
  - `curl -X POST http://localhost:8080/api/p2p/invitations/claim` → `404`
    (dead route on `apps/api` directly — this is exactly what the mobile client
    calls when pointed at `apps/api`'s port).
  - `curl -X POST http://localhost:8082/api/p2p/invitations/claim` → `401`
    (the standalone gateway process, found running outside Compose on 8082,
    correctly proxies and strips the prefix).
- **`infra/local/docker-compose.yml` has no `gateway` service.** The only way
  this path works locally is if a developer separately runs
  `cargo run -p dubbridge-gateway` outside Compose, on whatever port their local
  `config/local.toml`/env resolves — which happened to already be running in this
  environment (pre-existing process, not started by this session) on port 8082.
- `mobile/app.config.ts`'s compiled-in default `gatewayBaseUrl` is
  `http://10.0.2.2:8081`, which matches **neither** the local Compose `api`
  container (8080) **nor** the actual standalone gateway (8082) **nor** any
  gateway service inside Compose (none exists). `npm run android:p2p-cert`
  (`mobile/package.json`) does not set `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL`.

### Attempt 3 — correct gateway port (`http://10.0.2.2:8082`)

Relaunched via `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082 npx expo run:android`.
This rebuild/reinstall **invalidated the previously-persisted auth session** — the
app opened to the login screen instead of the harness (`auth.status !== "authed"`
correctly hides the harness). No credentials for the intended viewer account were
available in this task's instructions, and none were guessed or fabricated. Stopped
here per the task's explicit rule: do not work around a blocker, and do not proceed
without required input that only the human owner can supply.

**Operator incident during this attempt:** tap coordinates calibrated for the harness
screen were reused blindly on the login screen (different layout) and the invitation
token was briefly typed into the plaintext EMAIL field. It was immediately cleared
character-by-character before any submission, and the one screenshot that captured
partial token text was deleted rather than kept as evidence. No submission occurred
with that value in the email field. Recorded here for transparency; the underlying
invitation token file (`tmp/p5-invitation-token.txt`) was not modified.

## Classification

**`CLAIM_FAILED`**, but not the class of `CLAIM_FAILED` anticipated by the task
(expired invitation). This is a **P5/local-dev-environment defect**: the P5.T3
playbook's documented launch path (`npm run android:p2p-cert` against
`infra/local/docker-compose.yml`) has no way to reach a working backend, because:

1. The harness's compiled-in default gateway URL points at the wrong port for any
   known local topology.
2. The local Docker Compose stack that the mandatory preflight script targets does
   not include a `gateway` service at all — a developer must separately run one
   out-of-band, and nothing documents this requirement or its port.

This is **not** a P3 (invitation/authorization) or P4 (sync/package) defect — the
request never left the device to reach any P3/P4 boundary. It is a P5 harness/local-
environment wiring gap between `mobile/app.config.ts`'s default, `mobile/package.json`'s
`android:p2p-cert` script, and the actual local backend topology described by
`infra/local/docker-compose.yml` + the playbook.

## Not fixed

Per the task's explicit rule ("NO modifiques código salvo que encuentres un defecto
real reproducible" + "detente antes de modificar código y repórtalo"), no source,
config, or script file was modified. All corrected URLs used above were passed as
launch-time environment variable overrides only (an existing, designed escape hatch
in `mobile/app.config.ts`), never as a code/config edit.

## Attempt 4 — confirmed re-verification of the port-8082 relaunch (same-session continuation)

Re-verified rather than assumed: relaunched via the documented `android:p2p-cert`
script with `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082` set. Confirmed
the standalone gateway was still live first (`curl -X POST
http://localhost:8082/api/p2p/invitations/claim` → `401`, `lsof` showed the
long-lived PID 51009 still listening). Explored a lighter-weight alternative first —
`expo start --android` (Metro-only, no native reinstall) — to test whether the
gateway URL could be swapped without invalidating the session; this was **not
viable**: this app has custom native modules (`DubBridgeP2PKeyStore`, bare-kit
worklets) that Expo Go cannot run, and `expo start --android` targeted Expo Go
instead of the installed dev client, installing the wrong app entirely. Killed that
process and uninstalled the stray `host.exp.exponent` package immediately.

Proceeded with the only viable path, `npm run android:p2p-cert` (full native
rebuild via `expo run:android`, the same mechanism used in Attempt 3). Build
succeeded (`BUILD SUCCESSFUL in 4s`), app installed and opened on the emulator.
Screenshot confirms: **Sign in screen**, empty email/password fields — the
persisted auth session was invalidated by the rebuild, identically to Attempt 3.
This is not a new blocker; it is the same one, now directly re-confirmed rather
than inferred. No credentials were available or fabricated. Stopped here again per
the task's explicit rule.

**Conclusion: the port-8082 gateway routing hypothesis cannot be verified against
CLAIM in this environment without either (a) valid viewer credentials to log back
in after the required rebuild, or (b) a way to change `gatewayBaseUrl` without a
native rebuild** (not available today — `expo-constants`/`app.config.ts` `extra`
is read at bundle-load time but this app cannot run under Expo Go's bundle, and no
runtime-configurable override exists in the current code). This second point is a
new, concrete, minor recommendation added below.

## Infrastructure fix — `gateway` added to local Compose (2026-09-22)

Per explicit owner instruction ("arregla el compose"), added a `gateway` service
to `infra/local/docker-compose.yml`: `cargo run -p dubbridge-gateway` under the
`app` profile, host port `8082` mapped to the container's internal `8081`
(matching `config/local.toml`'s `[gateway] port = 8081`), with
`DUBBRIDGE_GATEWAY__UPSTREAM_API_BASE_URL=http://api:8080` overriding
`local.toml`'s `localhost:8080` default (which does not resolve from inside the
gateway's own container) and `DUBBRIDGE_DATABASE_URL`/`DUBBRIDGE_REDIS_URL`
pointed at the `postgres`/`redis` service names for the same reason (`AppConfig`
requires these fields to deserialize even though the gateway binary itself never
queries them). Also updated `mobile/app.config.ts`'s `gatewayBaseUrl` default
from `http://10.0.2.2:8081` to `http://10.0.2.2:8082`, so a plain
`npm run android:p2p-cert` (no manual env override) now targets the right
service, and documented the new precondition in
`docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md`. The previously-manual
standalone `dubbridge-gateway` process (PID 51009) was stopped to free port 8082
for the Compose-managed instance.

**Verified end-to-end:** container compiled (`cargo build`, first run, no cached
image layer — took 52m13s from cold, entirely expected for a fresh
`rust:1.98.1-bookworm`-based service with no pre-warmed target dir) and started
cleanly: `starting gateway port=8081 upstream_api_base_url=http://api:8080`.
`curl -X POST http://localhost:8082/api/p2p/invitations/claim` → `401` — proves
the Compose gateway correctly strips `/api` and proxies to the real `api`
container route, identically to the previously-manual standalone process this
session had been depending on. This closes the **infrastructure/routing** half
of the Attempt 1-4 root cause.

**Not resolved by this fix:** the CLAIM retry itself is still blocked. Testing
the corrected default still requires either (a) a fresh install that picks up
`mobile/app.config.ts`'s new default without a manual env override — a normal
`npm run android:p2p-cert` should now work with no override needed, since the
Compose gateway is durable across app rebuilds (unlike the earlier standalone
process, which was itself the reason a rebuild was tried in Attempt 3) — or (b)
valid viewer credentials to log back in if the persisted session was already
invalidated by an earlier rebuild in this same device/session lineage. Neither
was re-attempted as part of this infrastructure fix; it was scoped to the
Compose/config gap only, per the user's explicit request.

## Recommended follow-up (not actioned)

- Either add a `gateway` service to `infra/local/docker-compose.yml`, or document
  the required standalone `cargo run -p dubbridge-gateway` step and its port
  explicitly in `docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` preconditions.
- Fix `mobile/package.json`'s `android:p2p-cert` (and `android:p2p-dev`) scripts to
  either set a correct default `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` for local
  Compose, or fail loudly/require the operator to pass one.
- Consider hardening `infra/local/p2p/preflight.sh` to also check gateway
  reachability (`GET /health/live` on the resolved gateway URL), since the current
  preflight has no gateway-awareness at all and would not have caught this gap.
- Consider a runtime-configurable gateway URL (e.g. a dev-only in-app settings
  field, or reading from `AsyncStorage`/a debug menu) so a local topology fix can
  be tested without a full native rebuild, which unavoidably invalidates the
  persisted auth session on this device/build configuration.
- A negative-evidence CLAIM_FAILED run against a genuinely expired invitation (the
  scenario this task originally expected to reproduce) still needs to be captured
  once the gateway-routing gap above is resolved and a valid authenticated viewer
  session is available.

## Evidence

- `docs/audit/mvp0-p2p-p5-t3-android-certification-blocked-2026-09-22.md` (this file)
- Screenshots (local only, not committed — contain only UI chrome/stage text, no
  secrets): `tmp/p5_state_0_launch.png` through `tmp/p5_state_6_cleared.png` in the
  working tree at the time of this session (not part of the git-tracked evidence set;
  regenerate if needed for a follow-up session).

---

# Continuación 2026-09-22 — el bloqueo se movió de CLAIM a SYNC

El arreglo de infraestructura anterior (servicio `gateway` en Compose + default
`8082` en `mobile/app.config.ts`) **resolvió efectivamente el bloqueo de CLAIM**.
El registro de arriba, que clasifica P5.T3 como "BLOCKED at CLAIM stage", queda
**superado** por esta sección.

## CLAIM: confirmado OK

Dos invitaciones frescas e independientes fueron reclamadas con éxito desde el
harness: `p2p_invitations` quedó con `claimed_by_subject_id`/`claimed_at`
poblados, con timestamps coincidentes con los taps de "RUN P5 CERTIFICATION".
La semántica single-use quedó confirmada empíricamente (el segundo intento
requirió acuñar una invitación nueva).

## Nuevo resultado: `SYNC_FAILED`, reproducible 2/2

Dos corridas independientes (invitaciones distintas, ~10 min de separación)
terminaron ambas en `stage=idle` con `SYNC_FAILED`. Falla rápido, ~2-3 s — **no**
a los 30 s de `DEFAULT_IO_TIMEOUT_MS`, lo que descarta el timeout de discovery
como explicación.

### Cómo se obtuvo detalle sin violar el contrato de no-logging

`P5DeviceCertification.ts` colapsa deliberadamente cualquier excepción de
`startSync()` a `SYNC_FAILED`, sin detalle. No se agregó logging. En su lugar se
leyó el campo `lastError` que `P2pProductSync` **ya persiste** en el cache
on-device de estado de sync, documentado explícitamente como metadata no-secreta:

```
adb shell run-as com.dubbridge.mobile cat \
  /data/data/com.dubbridge.mobile/files/p2p-packages/<scope>/<pub>/<lineage>/sync-state.json
```

Resultado, idéntico en ambas corridas:

```
phase:     RETRYING
lastError: P2P source open failed: Product package could not be opened
```

### Cadena de propagación

`P5DeviceCertification.ts` → `P2PSyncController.startSync` →
`P2pProductSync.ts:166` (envuelve como *"P2P source open failed"*) →
`P2PServicePackageSource.open` → `P2PService.openProductPackage` →
`BareRuntimeClient` → worklet `product-package-runtime.ts::openPackage`.

### Localización del defecto

El código de error es `PRODUCT_PACKAGE_OPEN_FAILED`. En
`product-package-runtime.ts:121` ese código corresponde a la **rama catch-all**,
que solo se alcanza cuando el error capturado **no** es un `RuntimeProtocolError`
tipado. Un fallo de descubrimiento habría producido `REPLICATION_DISCOVERY_FAILED`
(líneas 111/115). Por lo tanto: **algo dentro del bloque `try` lanza una excepción
cruda**, y el primer candidato es `await drive.ready()` (línea 106).

Precisión metodológica: el texto del mensaje proviene del mapa código→mensaje de
`protocol-codec.ts:38`, no del `throw` original — lo que cruza el puente RPC es el
**código**. La conclusión se sostiene sobre el código, no sobre el string.

## P3 descartado — verificado en disco, no solo en DB

No se aceptó el estado de la DB como prueba suficiente. Se inspeccionó el
almacenamiento real del Availability Node (bind-mounted al host bajo `tmp/`):

| Evidencia | Resultado |
|---|---|
| `tmp/p2p-availability-index/59229676-….json` | Presente, contrato `availability-publication-v1` |
| `tmp/p2p-availability-drive/` | Corestore real, **136 MB** |
| `tmp/p2p-ciphertext/` | 70 MB |
| `external_publication_id` (index) vs (DB) | **Idénticos**: `5b726a6d…1f8e`, 64 hex = 32 bytes válidos |
| `p2p_publications.state` | `ready`, `external_confirmed_at=2026-09-18 22:28:09` |
| `p2p_publication_outbox.delivery_state` | `delivered` |

**El lado P3 está correcto y verificado end-to-end.** La drive key que el móvil
usa es exactamente la que el Availability Node publicó.

## Evidencia adicional del lado móvil

El directorio del paquete en el device contiene **únicamente** `sync-state.json`
(escrito por el lado RN/Expo). **No hay ningún Corestore ni Hyperdrive.** El
worklet Bare nunca llegó a escribir nada en disco, consistente con un fallo
inmediato en `drive.ready()`.

## Clasificación

**Defecto P4 (runtime P2P del cliente móvil).** No es P3, no es P5, no es el
entorno de red. Reproducible 2/2. Bloquea P5.T3.

## Hipótesis abiertas (NO confirmadas — requieren instrumentación)

1. **Esquema `file:` pasado a Corestore.** `runtimeStorageUri`
   (`product-package-runtime.ts:142-148`) exige que `runtime.argv[0]` empiece con
   `file:` y devuelve el URI **con el esquema incluido**. Ese string se pasa
   directo a `new Corestore(storageUri)` (línea 100), que espera un path de
   filesystem, no un URI. Contra-evidencia: `transient-drive.ts:47` usa el mismo
   patrón y P1 pasó — habría que confirmar si P1 ejercitó realmente esa ruta con
   Corestore.
2. **`this.active` residual entre corridas.** `product-package-runtime.ts:57`
   lanza `PRODUCT_PACKAGE_OPEN_FAILED` con mensaje *"already open"*, pero el codec
   lo renderiza como *"could not be opened"* — indistinguible desde el host.
   Requeriría un open exitoso previo sin close.

## No modificado

Ningún archivo fuente fue tocado en esta continuación. Solo lecturas, consultas
SQL, inspección de volúmenes e interacción UI. No se imprimió ningún token,
CK, envelope ni payload de runtime.

---

# Corrida diagnóstica instrumentada 2026-09-22 — causa confirmada

**Resultado:** hipótesis (1) confirmada como causa del `SYNC_FAILED` observado.
Hipótesis (2) descartada **solo para esta corrida**. P5.T3 sigue **BLOCKED por P4**.
Ninguna corrección permanente aplicada.

## Procedimiento (una sola corrida)

- Instrumentación temporal de `mobile/src/p2p/runtime/product-package-runtime.ts`
  (entrada de `open`, etapa, campos redactados del error antes del catch-all);
  `worklet.bundle.js` reconstruido; recarga vía menú Reload de Metro, sin reinstalar.
- Invitación fresca acuñada con una sesión owner vigente de un login real
  (HTTP 201, id `38bda18d-…`); token guardado sin imprimir. **Ya consumida.**
- Un solo tap en `Run P5 certification`. CLAIM confirmado en DB. Las dos aperturas
  registradas son el reconnect interno (`P2pProductSync` `executeWithReconnect`),
  no dos intentos manuales.
- Fuente y bundle restaurados byte a byte y re-verificados de forma independiente:
  `2794a3b3…` / `5d99caee…`, `git diff` vacío, `npm run check:bare-worklet` PASS.

## Evidencia decisiva (ambas aperturas)

```
branch=open-entry active=false
branch=catch stage=drive.ready protocolError=false code=ENOENT
message=ENOENT: no such file or directory, stat "file:"
messageHasFileScheme=true storageHasFileScheme=true
```

## Mecanismo, verificado en dependencias instaladas y reproducido fuera del device

- `Corestore(storage)` → `Hypercore.defaultStorage` → `hypercore-storage`
  `CorestoreStorage`: si `storage` es string, lo usa como **path**
  (`path.join(this.path, 'db')`, `fs.existsSync`, RocksDB). No hay conversión URI→path.
- Sonda en Node con Corestore/Hyperdrive/RocksDB reales (scratchpad, fuera del repo):
  - URI crudo `file://…/accounts/viewer-a`: "abre", pero crea el store en un directorio
    **relativo** `./file:/…` bajo el cwd. En Android (cwd `/`) eso produce el
    `ENOENT stat "file:"` observado.
  - `fileURLToPath(uri)`: store en la ruta absoluta correcta, incluso con espacio (`%20`).
  - `uri.replace(/^file:\/\//, '')`: deja `%20` literal → directorio equivocado.
    **Un `replace` no es suficiente.**
- Origen: `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` congeló pasar el URI
  "unchanged … avoiding an unsafe URI/path conversion", citando la guía oficial de
  Bare mobile. Esa guía sí convierte en el worklet:
  `join(URL.fileURLToPath(Bare.argv[0]), '…')`. El contrato omitió ese paso y ningún
  test lo detectó: la evidencia de P1 mockea Corestore/`ready()`, X29 verificó
  ping/packaging, y el único test de producto sobre storage compara strings de URI.
- Expo produce URIs correctamente codificados (Android `Uri.fromFile(...)`; JS
  `Paths.join` vía `URL` + `encodeURLChars`, autoridad vacía): se debe decodificar
  **exactamente una vez**.

## Opciones evaluadas

| Criterio | A — convertir en la frontera con Corestore | B — helper compartido con `transient-drive` | C — contrato worklet pasa a paths |
|---|---|---|---|
| Bare/Android | `bare-url@2.5.2` ya empaquetado y su addon ya carga en device (lo requiere `bare-fs`) | igual | requiere cambios en host (`BareRuntimeClient`, `ProofRuntimeFactory`, `SeedProofRunner`) |
| Espacios / percent-encoding | `fileURLToPath` decodifica una vez | igual | conversión en host; Expo JS no documenta un equivalente |
| Autoridad | rechaza host ≠ vacío/`localhost` | igual | a implementar |
| Absoluto | pathname de file URL siempre absoluto; rechaza `%2F`/`%00` | igual | nueva validación |
| Aislamiento por cuenta | intacto: scope validado se agrega antes de convertir; limpiador del host sin cambios | igual | host necesitaría ambas formas |
| Doble decodificación | un único punto (worklet) | igual | riesgo host+worklet |
| Alcance | 1 fuente + bundle + tests | + camino proof P1 (dev-only, mismo defecto latente, sin evidencia device propia) | reabre contrato congelado P1 |

**Recomendación: A**, alineada con la guía oficial y sin tocar el contrato
host→worklet. B queda como residual del camino proof P1 (fuera de la ruta de P5.T3).
C no justificado. Decisión abierta: declarar `bare-url@^2.5.2` como dependencia
directa en `mobile/package.json` (hoy transitiva vía `bare-fs`; el lockfile resuelve
la misma 2.5.2). Tarea definida: `docs/tasks/mvp0-p2p-p4-mobile-sync.md` § P4.T1-r1;
prompt de continuación: `docs/prompts/p4-t1-r1-storage-uri-fix.md`.

Viabilidad de evidencia real verificada: Corestore/Hyperdrive/RocksDB reales abren
bajo el preset `jest-expo` con `@jest-environment node` (sonda PASS en scratchpad).
Resolver `ENOENT` puede revelar un bloqueo posterior; no se asume PASS.

## Revisión de la corrida diagnóstica — NO cerrada

- Fase 1: `gpt-oss:20b` Complex — PASS (`.agent/p5-t3-diag/phase1-gpt-oss.json`, local, ignorado por git).
- Fase 2: pase 1 PASS; pase 2 devolvió `FINDINGS/BLOCKED` (token de veredicto
  inválido). Su único hallazgo sustantivo son los TS2339 de
  `__tests__/p2p.provider-account-change.test.tsx:64,75,94`, preexistentes al
  diagnóstico, e invoca evidencia incorrecta (`check:bare-worklet` no ejecuta `tsc`).
  Disposición **propuesta**: rechazado como fuera de alcance del diagnóstico,
  registrado como residual de P4.T3. Pase 3 no ejecutado.
- 2026-09-22: por instrucción del owner (uso semanal 94 %, complejidad), se suspende
  el uso de IA local para esta línea de trabajo; la revisión queda pendiente de
  decisión del owner (correr pase 3 o aceptar la disposición).

### Fase 2 — pase 3, 2026-09-22

Por decisión del owner se ejecuta pase 3 en lugar de aceptar directamente la
disposición del pase 2.

**Veredicto del pase 3: PASS.** El único hallazgo sustantivo del pase 2 eran los
tres TS2339 del test de account-change. Ese hallazgo no bloquea este diagnóstico:
era preexistente a la corrida y, además, la evidencia posterior del gate móvil
registra strict typecheck PASS en el source/test head de P4.T1-r1. No queda un
hallazgo diagnóstico bloqueante derivado del pase 2.

Este PASS cierra únicamente la revisión de la corrida diagnóstica. **No** marca
P5.T3 ni el agregado P5 como PASS; el rerun Android con invitación fresca sigue
siendo el gate pendiente.

## Residuales registrados

- `npm run typecheck` falla en HEAD: 3× TS2339 introducidos por `f2fa64c`
  (P4.T3 account-change test). No corregidos.
- `docs/tasks/mvp0-p2p-p4-mobile-sync.md` P4.T1 dice "bounded reconnect not
  implemented", pero `de199ff` lo implementó hoy (revisión pendiente). Estado desactualizado.
- Camino proof P1 (`transient-drive.ts` `proofStorageUri` → `openStoreAndDrive`)
  tiene el mismo defecto latente; dev-only.

---

# Continuación 2026-09-22 — P4.T1-r1 aplicado; rerun Android pendiente

El defecto P4 confirmado en la corrida instrumentada fue corregido en
`e44d00f497fe0f7128ce3a31a797b13840dafb2c`.

La frontera del runtime ahora conserva el contrato host→worklet como `file:` URI,
pero convierte el URI scoped exactamente una vez mediante
`bare-url.fileURLToPath` inmediatamente antes de construir Corestore. Un error de
conversión produce `PRODUCT_STORAGE_CONFIG_INVALID` antes de construir
Corestore/Hyperdrive/Hyperswarm.

Evidencia automatizada:
- GitHub Actions `35735766081`: mobile **PASS**, 61/61 suites y 441/441 tests.
- `product-storage-path.test.ts`: PASS con Corestore/Hyperdrive reales para el
  caso `%20`, aislamiento por cuenta y controles negativos de URI.
- GitHub Actions `35736737748`: RED contra el source exacto pre-fix
  `ed55050e6aaba916f3f4ce26d387a0aff74204ae` (6 fallos/7) y GREEN contra el
  source corregido (7/7 PASS).
- Worklet comprometido: sha256
  `2f1f79cdf1d62a2b7cd8fafd3819b4ccb5039b71b519fbf7e954d1654a7bfd1b`.
- GitHub Actions `35736354778`: `npm run check:bare-worklet` PASS desde
  checkout limpio.

Evidencia completa:
`docs/audit/p4-t1-r1-storage-uri-fix-evidence-2026-09-22.md`.

**Estado P5.T3:** ya no está bloqueado por el defecto conocido
`ENOENT stat "file:"`; queda **pendiente el rerun Android con invitación fresca
sobre la revisión final del branch**. Ese rerun debe registrar el siguiente estado
real de SYNC/VERIFY/PLAYBACK. No se infiere PASS de P5.T3 a partir de CI.

