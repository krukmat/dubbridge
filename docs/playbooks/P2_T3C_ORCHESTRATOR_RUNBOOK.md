---
type: Playbook
title: "Runbook del orquestador para P2.T3c"
status: active
task: P2.T3c
---

# Runbook del orquestador para P2.T3c

Este documento es la secuencia operativa exclusiva para `P2.T3c`. Empieza en
el análisis previo y termina al cerrar `P2.T3c`. No autoriza ni inicia
`P2.T3d`, `P2.T4` ni otra tarea.

La situación inicial correcta es:

```text
P2.T3c = PREPARADA PARA ORQUESTAR
P2.T3c != ALCANCE CONGELADO
P2.T3c != AUTORIZADA
P2.T3d = BLOQUEADA POR P2.T3c
```

Por tanto, el primer trabajo no es modificar TypeScript: es resolver y registrar
las cinco decisiones de preflight de `P2.T3c`.

## 0. Reglas que no debes saltar

- Trabaja únicamente sobre `P2.T3c`.
- Conserva cualquier cambio previo del usuario en el worktree.
- No modifiques código antes de congelar alcance, calcular RRI, superar fase 1
  y obtener aprobación si la banda es RRI 26+.
- No amplíes las rutas permitidas durante la implementación. Vuelve a análisis,
  actualiza el alcance y recalcula el RRI si aparece una ruta nueva.
- No inventes el formato o la ubicación de `package_ref`. Si no existe un
  productor del paquete materializado, detén `P2.T3c` y define un predecesor.
- No hagas commit, push, PR, despliegue, borrado ni `docker compose down -v`
  sin autorización explícita para esa acción.
- Un fallo de revisión o de aceptación deja la fase `blocked`; nunca lo
  conviertas en `completed` por conveniencia.

## 1. Inicializar la sesión

Desde la raíz del repositorio:

```bash
cd /Users/matias/dubbridge

export DUB_TASK_ID='P2.T3c'
export DUB_TASK_SLUG='p2-t3c'
export DUB_TASK_LEDGER='docs/tasks/mvp0-p2p-p2-encrypted-publication.md'
export DUB_TASK_PLAN='docs/plan/mvp0-p2p-p2-encrypted-publication.md'
export DUB_TASK_AUDIT_DIR='.agent/p2-t3c'
export DUB_CALLER='codex' # usa claude-code si ese es el orquestador real

mkdir -p "$DUB_TASK_AUDIT_DIR"
git status -sb
git diff --stat
rg -n -F "$DUB_TASK_ID" \
  "$DUB_TASK_LEDGER" \
  "$DUB_TASK_PLAN" \
  docs/plan/roadmap.md
```

Si el worktree ya contiene cambios, anótalos como preexistentes. No los limpies,
reviertas ni incluyas automáticamente en el alcance.

Guarda este checklist en la herramienta de tareas del orquestador y mantenlo
visible. Solo debe haber una fase `in_progress` normalmente:

```text
[in_progress] Resolver preflight y congelar alcance — orquestador
[pending] Calcular parent/leaf RRI y resolver ruta — orquestador
[pending] Reiniciar Ollama y comprobar modelos locales — orquestador, si aplica
[pending] Revisión de fase 1 — revisor resuelto por RRI
[pending] Aprobación humana — propietario, si RRI 26+
[pending] Implementar hojas aprobadas — implementador resuelto
[pending] Integrar, reflexionar y verificar — orquestador
[pending] Revisión de fase 2 — revisor resuelto por RRI
[pending] Verificación final del propietario y cierre — propietario/orquestador
```

## 2. Leer únicamente el contexto rector de T3c

```bash
sed -n '2410,2536p' "$DUB_TASK_LEDGER"
sed -n '175,205p' "$DUB_TASK_PLAN"
sed -n '117,170p' docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
sed -n '253,265p' docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
sed -n '1,220p' docs/audit/mvp0-p2p-p2-t3-readiness-2026-09-09.md
sed -n '1,260p' docs/adr/ADR-044-p2p-audience-delivery-boundary.md
sed -n '1,220p' apps/availability-node/src/contract.ts
sed -n '1,300p' apps/availability-node/src/server.ts
sed -n '1,240p' apps/availability-node/src/mtls.ts
cat apps/availability-node/package.json
cat apps/availability-node/tsconfig.json
```

No leas `P2.T3d` como una instrucción de ejecución. Solo es una dependencia
posterior que permanecerá bloqueada.

## 3. Confirmar que los predecesores están cerrados

Comprueba `T3b`, el contrato C0 y el contrato de paquete T2:

```bash
rg -n "P2.T3b|T3b.*Done|Owner final verification" "$DUB_TASK_LEDGER"
rg -n "P2.C0|C0.*PASS|contract freeze" \
  "$DUB_TASK_LEDGER" \
  docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
rg -n "P2.T2|T2.*Done|sealed package|package_ref" \
  "$DUB_TASK_LEDGER" \
  "$DUB_TASK_PLAN"
```

Continúa solo si la evidencia vigente confirma `T3b Done` y el contrato T2/C0
cerrado. Una mención en el roadmap o una prueba aislada no basta.

## 4. Crear el registro de preflight

Crea `docs/audit/mvp0-p2p-p2-t3c-preflight.md` con esta estructura y completa
cada campo con evidencia verificable:

```md
---
type: Audit
title: "MVP0-P2P P2.T3c preflight"
status: in_progress
task: P2.T3c
---

# P2.T3c — preflight

## Estado de dependencias

- T3b:
- T2 package contract:
- C0 contract:

## D1 — Dependencias directas

- Node comprobado:
- Versiones declaradas propuestas:
- Versiones resueltas propuestas:
- Compatibilidad y fuente:
- Decisión:

## D2 — Materialización de package_ref

- Productor actual:
- Raíz y ruta relativa:
- Nombre del manifiesto canónico:
- Layout de ciphertext:
- Evidencia en código:
- Decisión: RESUELTO | PREDECESOR REQUERIDO

## D3 — Contención de raíces

- Raíz de paquetes ciphertext:
- Raíz persistente del Availability Node:
- Normalización/realpath:
- Política de enlaces simbólicos:
- Decisión:

## D4 — Identidad y evidencia duraderas

- Clave lógica:
- Registro persistente:
- external_publication_id:
- evidence_id:
- confirmed_at:
- Momento de commit:
- Recuperación ante fallo ambiguo:
- Decisión:

## D5 — Concurrencia y ciclo de vida

- Serialización por ID:
- Semántica join/flush:
- Propietario del seed de larga duración:
- Cierre determinista:
- Reintento tras fallo:
- Decisión:

## Alcance congelado

- Rutas escribibles:
- Rutas excluidas:
- Hojas ejecutables y dependencias:
- Pruebas por hoja:
- Integración del padre:

## Resultado

- Estado: ALCANCE CONGELADO | BLOCKED
- Motivo:
```

Mientras lo completas, no cambies `status` a `complete` ni escribas
`ALCANCE CONGELADO` hasta resolver las cinco decisiones.

## 5. Resolver D1: dependencias directas

Comprueba la versión local de Node y las dependencias ya declaradas:

```bash
node --version
npm --version
npm --prefix apps/availability-node ls --depth=0 || true
rg -n 'corestore|hyperdrive|hyperswarm' \
  apps/availability-node/package.json \
  apps/availability-node/package-lock.json \
  mobile/package.json \
  mobile/package-lock.json 2>/dev/null || true
```

Consulta el registro sin instalar ni modificar todavía el lockfile:

```bash
npm view corestore version engines dependencies --json
npm view hyperdrive version engines dependencies peerDependencies --json
npm view hyperswarm version engines dependencies peerDependencies --json
```

Registra versiones compatibles con Node 22. Las tres deben ser dependencias
directas de `apps/availability-node`; no aceptes una resolución transitoria desde
`mobile/` u otro paquete.

## 6. Resolver D2: materialización de package_ref

Busca quién construye y quién escribe realmente el manifiesto y los ciphertext:

```bash
rg -n "package_ref|manifest_digest_sha256|canonical_manifest|sealed_package|ciphertext" \
  apps crates workers docs/fixtures docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
rg -n "write|persist|put|create_dir|File::create|fs::" \
  crates apps/worker-runner apps/api | rg "package|manifest|cipher|p2p"
```

Debes poder nombrar exactamente:

1. el componente que materializa el paquete;
2. la raíz compartida;
3. el significado relativo de `package_ref`;
4. el nombre del archivo de manifiesto canónico;
5. el layout de los ciphertext referenciados por ese manifiesto.

Si no existe ese productor o el contrato no está congelado:

```text
P2.T3c = BLOCKED
motivo = falta un predecesor de materialización del paquete T2 -> T3
acción permitida = definir y presentar ese predecesor
acción prohibida = inventar el layout dentro de P2.T3c
```

Detente aquí. No calcules un RRI de implementación de T3c sobre un contrato
inventado.

## 7. Resolver D3: contención de raíces

Congela dos raíces inyectadas y distintas:

- raíz de lectura de paquetes ciphertext;
- raíz persistente privada de Corestore/Hyperdrive.

Documenta cómo se rechazan antes de publicar:

- `package_ref` absoluto;
- `..`, separadores inversos y colisiones tras normalización;
- rutas cuyo `realpath` queda fuera de la raíz;
- componentes o archivos que son enlaces simbólicos y escapan;
- manifiestos que referencian plaintext, secretos o rutas no declaradas.

Comprueba las utilidades y convenciones existentes:

```bash
rg -n "realpath|symlink|lstat|readlink|normalize|resolve|traversal|package_invalid" \
  apps/availability-node crates docs
```

La decisión debe ser implementable y comprobable sin añadir configuración de
despliegue, listener público ni credenciales a T3c.

## 8. Resolver D4: identidad y evidencia duraderas

Congela un registro persistente que vincule:

```text
(publication_id, lineage_id, manifest_digest_sha256)
    -> external_publication_id + evidence_id + confirmed_at
```

Especifica:

- dónde vive el registro y cómo se recupera tras reiniciar el proceso;
- cómo se conserva byte por byte la evidencia de la primera publicación;
- cómo se detectan conflictos por `publication_id`, lineage y digest;
- cómo se evita confirmar éxito antes de validar, escribir, anunciar y hacer
  flush según el contrato congelado;
- qué estado queda después de un fallo ambiguo y por qué el reintento es seguro.

El Availability Node no puede usar PostgreSQL ni decidir `P2P_READY`.

## 9. Resolver D5: concurrencia y ciclo de vida

Define y registra:

- exclusión/serialización para peticiones concurrentes del mismo ID;
- comportamiento para IDs diferentes;
- cuándo se considera completado `Hyperswarm.join()` y su flush;
- quién mantiene vivos Corestore, Hyperdrive y Hyperswarm mientras se siembra;
- cómo se reconstruye el publisher con la misma raíz persistente;
- orden y carácter idempotente de `close()`;
- tratamiento de fallos de open/write/join/flush/persistencia.

La salida debe mapear cada fallo a `503 publication_unavailable` sin grabar una
confirmación falsa.

## 10. Congelar alcance y descomponer honestamente

Cuando D1–D5 estén resueltas, convierte el sobre candidato en rutas exactas. El
máximo actual es:

```text
apps/availability-node/package.json
apps/availability-node/package-lock.json
apps/availability-node/src/hyperdrive_store.ts
apps/availability-node/src/server.ts
apps/availability-node/test/hyperdrive-store.test.js
apps/availability-node/test/publication-idempotency.test.js
```

Antes del RRI, intenta separar hojas que tengan valor y verificación propios,
por ejemplo dependencias/arranque, almacenamiento e integridad, identidad
duradera, seeding/lifecycle e integración. Estos son temas candidatos, no IDs
preaprobados: registra en el ledger los IDs, rutas, aceptación, `HP`/`EC`, orden
y comandos reales que resulten del preflight.

No dividas una invariante solo para bajar el RRI. El padre conserva su RRI y
gobierna aprobación, revisión integrada y Reflection. Si una hoja no se puede
verificar por separado, déjala en el residuo con su puntuación real.

Marca el preflight `complete` y `ALCANCE CONGELADO` solo después de sincronizar
la definición de `P2.T3c` en el ledger.

## 11. Calcular el RRI del padre y de cada hoja

No calcules el resultado a mano. Asigna `C`, `T`, `A`, `X`, `D`, `K` y `P` con
la rúbrica vigente y ejecuta el script. Para el padre de seis rutas:

```bash
export DUB_RRI_C='<0-5>'
export DUB_RRI_T='<0-5>'
export DUB_RRI_A='<0-5>'
export DUB_RRI_X='<0-5>'
export DUB_RRI_D='<0-5>'
export DUB_RRI_K='<0-5>'
export DUB_RRI_P='<0-5>'

PATH="/opt/homebrew/opt/python@3.11/libexec/bin:$PATH" \
python3 scripts/rri.py \
  --touches apps/availability-node/package.json \
  --touches apps/availability-node/package-lock.json \
  --touches apps/availability-node/src/hyperdrive_store.ts \
  --touches apps/availability-node/src/server.ts \
  --touches apps/availability-node/test/hyperdrive-store.test.js \
  --touches apps/availability-node/test/publication-idempotency.test.js \
  --C "$DUB_RRI_C" \
  --T "$DUB_RRI_T" \
  --A "$DUB_RRI_A" \
  --X "$DUB_RRI_X" \
  --D "$DUB_RRI_D" \
  --K "$DUB_RRI_K" \
  --P "$DUB_RRI_P" \
  | tee "$DUB_TASK_AUDIT_DIR/parent-rri.md"
```

Sustituye los marcadores antes de ejecutar. Añade únicamente las penalizaciones
que correspondan, por ejemplo `--penalty arch_decision`; no suprimas las que el
script detecte. Ejecuta otro informe para cada hoja usando solamente sus rutas y
sus propios valores.

Registra los informes completos en un artefacto versionado de
`docs/audit/` y actualiza `Effort` en el ledger según el RRI real:

| RRI | Banda | Effort | Consecuencia inmediata |
|---|---|---|---|
| 0–25 | Low | S | sin tarjeta completa; revisión y cierre siguen aplicando |
| 26–40 | Moderate | M | fase 1 + aprobación; local-first |
| 41–55 | Med-high | L | fase 1 + aprobación + ruta ADR-038 |
| 56–70 | Complex | L | descomposición obligatoria + aprobación |
| 71–85 | High | XL | descomposición y revisión humana del diff |
| 86–100 | Very high | XL | ADR/riesgo y descomposición; no implementación directa |
| >100 | Excessive | XL | reconfigurar arquitectura/alcance; no implementación |

Si el padre obtiene RRI 56+, no presentes ni implementes el padre monolítico.
Completa primero la descomposición y puntúa cada hoja; el RRI del padre sigue
siendo el sobre de gobierno.

## 12. Preparar el paquete de revisión de fase 1

Crea `.agent/p2-t3c/phase1-packet.md` con:

- objetivo y estado de dependencias;
- decisiones D1–D5;
- alcance exacto del padre y de cada hoja;
- RRI completo del padre y hojas;
- criterios 1–6 y casos `HP-T3c-1/2`, `EC-T3c-1/2`;
- exclusiones;
- comandos de verificación;
- evidencia y documentos de estado que se actualizarán;
- ruta de implementación/fallback resuelta por banda;
- referencias rectoras.

Antes de la revisión comprueba el enrutamiento sin invocar un modelo:

```bash
python3 scripts/peer-workflow-review.py \
  --phase task \
  --rri <RRI_PADRE> \
  --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase1-packet.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase1-review.json" \
  --dry-run
```

## 13. Reiniciar Ollama cuando la ruta vaya a usarlo

Hazlo una vez para `P2.T3c`, justo antes de la primera llamada a un rol local.
No lo hagas si la fase resuelta no usa Ollama.

```bash
pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|gemma-code-review|peer-workflow-review' || true
pgrep -fl ollama || true
lsof -nP -iTCP:11434 -sTCP:LISTEN || true
```

Si no hay una ejecución ajena activa:

```bash
OLD_OLLAMA_PID="$(pgrep -x ollama | paste -sd, -)"
osascript -e 'quit app "Ollama"'
open -a Ollama
sleep 3
NEW_OLLAMA_PID="$(pgrep -x ollama | paste -sd, -)"
printf 'old=%s new=%s\n' "$OLD_OLLAMA_PID" "$NEW_OLLAMA_PID"
lsof -nP -iTCP:11434 -sTCP:LISTEN
curl -fsS http://127.0.0.1:11434/api/ps | jq .
curl -fsS http://127.0.0.1:11434/api/tags | jq -r '.models[].name'
```

Calienta cada modelo que realmente usará la banda con el `num_ctx` de producción
definido en el workflow. Exige `done_reason: stop` y contenido no vacío. Registra
PID anterior/nuevo, listener, modelo, contexto, `num_predict`, resultado y
estado de `/api/ps` en `.agent/p2-t3c/ollama-precheck.md`.

No mates runners ajenos. Ante contenido vacío, aplica el protocolo de
recuperación del workflow; no repitas indefinidamente el mismo perfil.

## 14. Ejecutar fase 1

Después del precheck local, si aplica:

```bash
python3 scripts/peer-workflow-review.py \
  --phase task \
  --rri <RRI_PADRE> \
  --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase1-packet.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase1-review.json"

jq . "$DUB_TASK_AUDIT_DIR/phase1-review.json"
```

Solo un veredicto `PASS` permite avanzar. `BLOCKED`, salida inválida o ausencia
del revisor sigue la cadena de fallback de la banda. Si el fallback llega a D14
o a un implementador cloud, crea primero el recibo `fallback-selection-v1`; el
modo predeterminado es `human-select` y se detiene hasta que el propietario elija
modelo, esfuerzo y selector.

Si corriges materialmente el paquete, crea una nueva versión y repite fase 1 con
otro nombre de artefacto. No sobrescribas la evidencia anterior.

## 15. Presentar la tarjeta y detenerse para aprobación

Para RRI 26+, rellena los seis bloques de
`docs/templates/compact-approval-task-card.md` usando el RRI del padre:

1. cabecera de decisión y enrutamiento;
2. alcance y aceptación;
3. flujo de agentes con responsables reales;
4. diagrama de agentes y diagrama técnico;
5. referencias materiales;
6. punto de aprobación.

La tarjeta debe mostrar las hojas, pero no reemplaza sus definiciones ni sus
informes RRI. Termina exactamente con:

```text
Execution has not started. Approve this task to proceed.
```

Detente. No interpretes preguntas, lectura de la tarjeta ni aprobaciones antiguas
como aprobación de `P2.T3c`. Para RRI 0–25 no se presenta la tarjeta completa y
se sigue la ruta Low canónica.

## 16. Reanudar después de la aprobación

Registra quién aprobó, fecha, alcance/RRI exactos y sesión. Actualiza el checklist:

```text
[completed] Resolver preflight y congelar alcance
[completed] Calcular parent/leaf RRI y resolver ruta
[completed] Reiniciar Ollama y comprobar modelos locales — si aplicó
[completed] Revisión de fase 1
[completed] Aprobación humana
[in_progress] Implementar hojas aprobadas
```

Ejecuta las hojas en su orden de dependencias y mediante la ruta que haya
resuelto el RRI. Cada implementador recibe únicamente:

- su objetivo y rutas permitidas;
- los contratos D1–D5 que no puede reinterpretar;
- sus `HP`/`EC` y pruebas;
- comandos de verificación;
- condición de parada y formato de evidencia.

Después de cada hoja:

```bash
git status --short
git diff --check
git diff -- <rutas-exactas-de-la-hoja>
```

Rechaza cualquier cambio fuera de alcance. Si una hoja descubre una decisión no
congelada, vuelve a preflight/RRI; no permitas que el implementador la invente.

No inicies `P2.T3d` cuando termine la última hoja. Primero debes integrar y cerrar
el padre `P2.T3c`.

## 17. Ejecutar RED, GREEN e integración del padre

Para cada hoja de lógica determinista, registra primero el fallo esperado de la
prueba (`RED`) y después el pase tras implementar (`GREEN`). Finalmente ejecuta
la batería integrada exacta:

```bash
npm --prefix apps/availability-node ci --ignore-scripts --no-audit --no-fund
npm --prefix apps/availability-node audit --omit=dev --json
npm --prefix apps/availability-node run typecheck
npm --prefix apps/availability-node run build
node --test apps/availability-node/test/hyperdrive-store.test.js \
  apps/availability-node/test/publication-idempotency.test.js
node --test apps/availability-node/test/*.test.js \
  docs/audit/mvp0-p2p-p2-t3a-contract.test.js \
  docs/audit/mvp0-p2p-p2-t3a-http.test.js
git diff --check
make qa-docs
```

No cierres con una instalación incompleta, un audit sin revisar o pruebas
saltadas. Registra salida, código de retorno y cualquier excepción aprobada.

## 18. Ejecutar Reflection

Usa el número de pasadas del RRI del padre:

- Moderate: 2;
- Med-high: 3;
- Complex: 4;
- High/Very high: aplica como mínimo el contrato complejo después de la
  descomposición aprobada y cualquier requisito adicional de la banda.

Cada pasada debe ser `Draft -> Critique -> Revise`, con foco ordenado en:

1. contrato C0, replay estable y conflictos;
2. integridad/contención fail-closed y ausencia de plaintext/secretos;
3. concurrencia, fallos ambiguos y ciclo de vida;
4. cobertura e integración completa, cuando la banda exija cuatro pasadas.

Registra cada hallazgo y corrección en `### Reflection log` dentro del registro
de cierre de T3c. Una pasada sin cambios debe decir explícitamente `none`.

## 19. Preparar y ejecutar la revisión de fase 2

Crea `.agent/p2-t3c/phase2-packet.md` con:

- diff exacto y lista de archivos;
- contrato y aceptación aprobados;
- decisiones D1–D5;
- pruebas y sus resultados;
- evidencia RED/GREEN;
- Reflection log;
- incidencias, desviaciones y riesgos restantes.

Ejecuta:

```bash
python3 scripts/peer-workflow-review.py \
  --phase code \
  --rri <RRI_PADRE> \
  --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase2-packet.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase2-review.json"

jq . "$DUB_TASK_AUDIT_DIR/phase2-review.json"
```

Lee y resuelve todos los findings, no solo el veredicto agregado. Tras una
reparación material, vuelve a ejecutar las pruebas afectadas y una revisión de
fase 2 sobre el snapshot nuevo. `BLOCKED` no permite cerrar.

## 20. Certificar comportamiento y pedir verificación final

En la entrada de `P2.T3c`, añade una tabla `Behavioral coverage certification`
que mapee todos estos casos a evidencia ejecutable aprobada:

```text
HP-T3c-1
HP-T3c-2
EC-T3c-1
EC-T3c-2
```

Cada fila debe indicar tipo, comportamiento, capa, prueba/selector ejecutable y
resultado `passed`. `N/A` no es válido.

Después pide al propietario que ejecute o confirme los comandos finales y
registra:

```md
### Owner final verification

- Owner: <nombre-o-handle>
- Date: YYYY-MM-DD
- Statement: I verified every happy path and edge case defined for this task has executable evidence at an appropriate layer that replicates the expected behavior.
- Commands run: <comandos exactos>
```

Sin esta verificación `P2.T3c` no está Done.

## 21. Sincronizar estado y cerrar únicamente T3c

Actualiza en la misma pasada:

- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`;
- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`;
- `docs/plan/roadmap.md`, si cambia materialmente la preparación downstream;
- artefactos RRI, aprobación, implementación y revisiones de T3c;
- el estado de dependencia de `P2.T3d`, pero sin ejecutarla.

Ejecuta una última vez:

```bash
git status --short
git diff --stat
git diff --check
make qa-docs
npm --prefix apps/availability-node run typecheck
npm --prefix apps/availability-node run build
node --test apps/availability-node/test/*.test.js \
  docs/audit/mvp0-p2p-p2-t3a-contract.test.js \
  docs/audit/mvp0-p2p-p2-t3a-http.test.js
```

Marca `P2.T3c` como `Done` únicamente si todo está verde, fase 2 es `PASS`, el
registro de Reflection está completo y existe la verificación final del
propietario.

El estado final permitido es:

```text
P2.T3c = Done
P2.T3d = preparada/desbloqueada, pero no iniciada
```

Detente y entrega el resumen de T3c. El cambio a `P2.T3d` requiere una nueva
orquestación, nuevo RRI y sus propias puertas.

## 22. Comando de monitoreo rápido

Define esta función al comenzar cada sesión:

```bash
p2_t3c_status() {
  printf 'task=%s\nledger=%s\nplan=%s\n' \
    "$DUB_TASK_ID" "$DUB_TASK_LEDGER" "$DUB_TASK_PLAN"
  git status -sb
  git diff --stat
  rg -n -F "$DUB_TASK_ID" \
    "$DUB_TASK_LEDGER" "$DUB_TASK_PLAN" docs/plan/roadmap.md
  pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|gemma-code-review|peer-workflow-review' || true
  lsof -nP -iTCP:11434 -sTCP:LISTEN || true
  find "$DUB_TASK_AUDIT_DIR" -maxdepth 1 -type f -print 2>/dev/null | sort
}

p2_t3c_status
```

Interpretación rápida:

- decisiones D1–D5 incompletas: continúa preflight;
- preflight `BLOCKED`: define el predecesor y no ejecutes T3c;
- alcance congelado sin RRI: calcula padre y hojas;
- fase 1 sin `PASS`: no presentes ni implementes;
- RRI 26+ sin aprobación: detente en la tarjeta;
- implementación con pruebas rojas: no pases a fase 2;
- fase 2 sin `PASS` o sin verificación del propietario: no marques Done;
- T3c Done: detente; no empieces T3d.

## Referencias rectoras

1. `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` — definición de T3c.
2. `docs/plan/mvp0-p2p-p2-encrypted-publication.md` — secuencia del slice.
3. `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` — contrato C0.
4. `docs/adr/ADR-044-p2p-audience-delivery-boundary.md` — frontera de autoridad.
5. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — workflow canónico.
6. `docs/policies/RRI_POLICY.md` — cálculo y banda RRI.
7. `docs/policies/HITL_AUTONOMY_POLICY.md` — aprobación y autonomía.
8. `docs/templates/compact-approval-task-card.md` — tarjeta para RRI 26+.
