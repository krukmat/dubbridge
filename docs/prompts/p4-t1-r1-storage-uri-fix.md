---
type: Prompt
title: "Continuation prompt — P4.T1-r1 product storage URI fix (unblocks P5.T3)"
governs: "docs/tasks/mvp0-p2p-p4-mobile-sync.md § P4.T1-r1"
status: ready
date: 2026-09-22
---

# Continuation prompt — P4.T1-r1

```text
Continúa en /Users/matias/dubbridge, branch feature/p2p-mvp-core.

OBJETIVO
Implementar P4.T1-r1 (opción A): convertir el file: URI del storage de producto a
path, exactamente una vez, en la frontera con Corestore. Desbloquea P5.T3 SYNC.

LEER (solo esto)
- docs/tasks/mvp0-p2p-p4-mobile-sync.md § P4.T1-r1  (criterios, paths, tests)
- docs/audit/mvp0-p2p-p5-t3-android-certification-blocked-2026-09-22.md
  § "Corrida diagnóstica instrumentada 2026-09-22"
- mobile/src/p2p/runtime/product-package-runtime.ts

DECISIONES DEL OWNER PENDIENTES (preguntar antes de implementar)
1. Revisión fase 2 del diagnóstico: correr pase 3 o aceptar la disposición
   propuesta del pase 2 (TS2339 preexistentes → residual P4.T3).
2. Declarar bare-url@^2.5.2 como dependencia directa (recomendado) o no.
3. Modo de revisión: al 2026-09-22 el owner suspendió la IA local para esta línea
   por uso (94 % semanal). Confirmar si sigue suspendida o se reactiva.

PASOS
1. scripts/rri.py sobre los allowed paths; si RRI >= 26, card compacta y esperar
   aprobación.
2. Test primero (RED): mobile/__tests__/p2p/product-storage-path.test.ts con
   Corestore/Hyperdrive reales, @jest-environment node, chdir a tmp,
   jest.mock("bare-url", () => require("node:url")), hyperswarm falso.
3. Fix en openPackage: fileURLToPath de bare-url (require perezoso, patrón
   bare-crypto); error de conversión → PRODUCT_STORAGE_CONFIG_INVALID antes de
   crear Corestore/Hyperswarm. No usar replace de "file:".
4. npm run build:bare-worklet; check:bare-worklet; jest focalizado; lint;
   typecheck comparado con la línea base de 3 TS2339.
5. Enmienda breve en docs/audit/mvp0-p2p-p1-a1b-storage-contract.md.
6. Device: invitación FRESCA (la de 2026-09-22 está consumida), un tap en
   Run P5 certification; registrar la siguiente etapa tal cual.

REGLAS
- Nunca imprimir JWT, CK, envelopes ni tokens de invitación.
- docker-compose (no "docker compose"); Postgres -U dubbridge -d dubbridge.
- Preservar cambios previos del usuario (compose gateway, app.config 8082,
  playbook, ledger P5, audit).
- No marcar P5 aggregate PASS. No tocar transient-drive (residual).
```
