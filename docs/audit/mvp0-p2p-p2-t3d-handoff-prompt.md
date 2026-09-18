---
type: Audit
title: "P2.T3d (CONS-T3) — handoff prompt for a follow-up session"
status: active
task: P2.T3d
---

# P2.T3d (CONS-T3) — handoff prompt

Copy the block below verbatim into a new Claude Code session in this
repository to resume implementation.

---

Vamos a ejecutar P2.T3d (= CONS-T3 en el ledger de remediación), ya
**aprobado por el owner el 2026-09-18** ("CONST-T3 Aprobado", Matias).

**Lee primero, en este orden:**
1. `docs/audit/mvp0-p2p-p2-t3d-preflight.md` — preflight completo: mapeo de
   cada criterio a un test concreto (§9), RRI 70 Complex con su honest-low-
   band-maximization pass (§11), fase-1 review PASS (§12).
2. `docs/tasks/mvp0-p2p-s230-consistency-remediation.md` § CONS-T3 — el
   registro de aprobación y el estado exacto al momento del handoff.
3. `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § P2.T3d — la
   definición original de la tarea (acceptance criteria HP-T3d-1, HP-T3d-2,
   EC-T3d-1, EC-T3d-2, EC-T3d-3, criterio 6; exact writable paths).
4. `docs/playbooks/P2_T3D_ORCHESTRATOR_RUNBOOK.md` — el procedimiento
   humano-conducido original; ya no hace falta repetir sus pasos 1-7
   (sesión, entrada T3c, paquete congelado, RRI, aprobación) porque ese
   trabajo ya está hecho y aprobado. Retoma desde su **paso 8** (dar la
   orden de implementar).

**No recalcules RRI ni repitas la fase-1 review** — el alcance está
congelado y aprobado. Solo si descubrís que el alcance real necesita
cambiar (un archivo adicional, un criterio no cubierto) volvé a correr
`scripts/rri.py` sobre el paquete revisado y una nueva fase-1 antes de
continuar.

**Rutas exactas a escribir (únicas autorizadas):**
- `apps/availability-node/test/publication-contract.test.js`
- `apps/availability-node/test/fixtures.js`

**No toques `src/`.** Si una prueba descubre un defecto de producto,
detenete, documentá la reproducción, y marcá T3d como blocked — reabrir
T3a/T3b/T3c es una tarea aparte con su propio RRI, no se repara acá.

**Qué implementar (resumen del preflight §9, ver el archivo para el mapeo completo):**
- Reusar el harness mTLS de `apps/availability-node/test/private-publication-ingress.test.js` (genera CA/server/client certs con `openssl` vía `execFileSync`, limpia con `mkdtemp`/`finally`) en vez de inventar uno nuevo.
- Reusar `createPublicationExecutor` (el executor real) y el binario Rust de fixtures `package_build_and_materialize_fixture` (ya usado en `package-publication-integration.test.js` de T3c-Integ) para producir ciphertext real, no fixtures JS hardcodeadas.
- HP-T3d-1: cliente mTLS permitido publica un paquete real → `201`; reconstruir el servidor sobre las mismas raíces persistentes, replay → `200`, evidencia idéntica.
- HP-T3d-2: abrir el Hyperdrive por `external_publication_id`/`publicKeyHex`, comparar bytes con lo que `verifyPackage` aceptó.
- EC-T3d-1: reusar el patrón de `private-publication-ingress.test.js` pero con el executor real como publisher (no un stub) — sin cert/cert rogue → fallo TLS; cliente no listado → `403`; cero invocaciones al executor en ambos casos.
- EC-T3d-2 (5 sub-casos): conflicto de lineage (mismo patrón que `EC-T3c-1a`) → `409`; conflicto de digest (`EC-T3c-1b`) → `409`; ciphertext tampereado (`EC-T3c-2`) → `422`; symlink-escape vía `containment.ts` → `422`; JSON malformado / content-type incorrecto / body sobredimensionado → `400` `invalid_contract` (NO `422` — es una distinción real del contrato, confirmada en el preflight §9).
- EC-T3d-3: usar `hyperswarmJoinTimeoutMs` muy bajo para forzar `503` `publication_unavailable` (seam ya confirmado por lectura de código en `publication_executor.ts`/`server.ts`, ver preflight §7 y §10) + escaneo negativo de los 8 campos de `secret_deny_list` (`docs/fixtures/mvp0-p2p-publication-contract-v1.json`) contra: respuesta HTTP, logs de consola, el registro de índice persistido, y los bytes crudos del drive.
- Criterio 6: correr junto con el resto del suite (`apps/availability-node/test/*.test.js` + los dos tests bajo `docs/audit/mvp0-p2p-p2-t3a-*.test.js`), sin dependencia de red pública, limpieza determinista de todos los `mkdtemp`.

**No debilitar aserciones para obtener verde.** Las pruebas certifican código
ya implementado (T3a-T3c cerrados) — es normal y esperado que pasen en su
primer intento; eso no invalida la certificación, siempre que las
aserciones realmente distingan fixtures válidas de inválidas (usar el mismo
patrón que `package-publication-integration.test.js` ya usa: comparar
`core.length` antes/después para probar que no hubo segunda escritura,
comparar bytes exactos, etc.).

**Routing aprobado:** RRI 70 Complex, cloud-primary (Claude Opus 5,
escalado desde Sonnet 5 por la tabla de capacidad Claude para RRI 56-70),
sin ruta local-first. 4 pasadas de Reflection obligatorias antes del
cierre.

**Fase-2 (code-solution) review — IMPORTANTE, defecto conocido:**
`scripts/peer-workflow-review.py` tiene un bug confirmado para RRI 56+: su
`main()` salta el reviewer primario (`gpt-oss:20b` perfil Complex) e invoca
directamente el fallback cross-vendor (ver
`docs/audit/peer-workflow-review-complex-band-defect-2026-09-18.md`, no
corregido aún). **No uses ese script para fase-2.** Invocá `gpt-oss:20b`
directamente contra `http://127.0.0.1:11434/api/chat` con el perfil Complex
(`num_ctx=49152`, `num_predict=10240`, `think="medium"`, `temperature=1.0`,
`top_p=1.0`), y si falla/da `BLOCKED`, recién ahí escalá al par cross-vendor
(codex para caller=claude-code) como fallback, luego D14 como último
recurso — exactamente el orden documentado en
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`. Antes
de la primera llamada a Ollama de esta sesión, hacé el precheck de reinicio
por-tarea (Step 0 de la guía) si no se hizo ya en esta misma sesión.

**Reporte de cierre exigido (formato del report-line contract):**
```
Task-analysis review: gpt-oss .agent/p2-t3d/phase1-review-v1.json - PASS
Code-solution review: <gpt-oss|codex|d14> <artefacto real> - <PASS|BLOCKED>
```

**Al terminar:**
1. Reflection log (4 pasadas), tabla de Behavioral coverage certification
   (Case ID | Type | Behavior | Layer | Executable evidence | Result)
   cubriendo los 2 HP y los 3 EC con todas sus sub-condiciones.
2. Pedime a mí (el owner) la verificación final — no la anticipes ni la
   inventes.
3. Sincronizá en la misma pasada: `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
   § P2.T3d (marcar Done solo si todas las puertas están satisfechas),
   `docs/tasks/mvp0-p2p-s230-consistency-remediation.md` § CONS-T3, el
   resumen de T3 en `docs/plan/roadmap.md`, y el estado de dependencia de
   T4c. No inicies T4c. No cierres el padre P2.
4. Corré `git diff --check` y `make qa-docs` después de sincronizar.
5. Recordá el /compact al final.

**Detenete** después de cerrar T3d y sincronizar estado — no continúes a
T4c ni a otra tarea del ledger de remediación sin nueva instrucción mía.
