---
type: Playbook
title: "P2.T3d — guía para el orquestador humano con Claude Code"
status: active
task: P2.T3d
---

# P2.T3d — tus pasos como orquestador

Empieza en el **paso 1**. Esta guía te permite conducir `P2.T3d` aunque Codex
no tenga tokens. Tú decides y apruebas; Claude Code te ayuda a analizar,
preparar los paquetes, coordinar ejecutores y mantener las evidencias.

**Precondición para usar esta guía:** `P2.T3c` está cerrada y su implementación
y evidencias están disponibles. El procedimiento está preparado para ese momento
futuro, por indicación del propietario. Resolver esa dependencia se hará aparte;
no es un paso pendiente de la preparación de este documento ni una declaración
de que T3c ya esté cerrada hoy.

El resultado de T3d son pruebas de certificación del Availability Node: publicación
real de ciphertext, mTLS, replay tras reconstrucción, conflictos, contención de
rutas y ausencia de secretos. No es una comprobación de este documento.

## 1. Abre la terminal y prepara la sesión

```bash
cd /Users/matias/dubbridge
export PATH="/opt/homebrew/opt/python@3.11/libexec/bin:$PATH"
export DUB_TASK_ID='P2.T3d'
export DUB_CALLER='claude-code'
export DUB_TASK_LEDGER='docs/tasks/mvp0-p2p-p2-encrypted-publication.md'
export DUB_TASK_PLAN='docs/plan/mvp0-p2p-p2-encrypted-publication.md'
export DUB_TASK_AUDIT_DIR='.agent/p2-t3d'
mkdir -p "$DUB_TASK_AUDIT_DIR"
set -o pipefail

python3 --version
node --version
git status -sb
git diff --stat
git worktree list
```

En esta máquina, Python 3.11 permite cargar los scripts de revisión; Python 3.9
falla al importarlos. Comprueba Node contra `apps/availability-node/package.json`.
Anota los cambios preexistentes y consérvalos. No limpies el worktree.

Abre Claude Code en este repositorio con tu método habitual. Comprueba primero
su disponibilidad si quieres iniciarlo por terminal:

```bash
command -v claude
claude
```

Mantén otra terminal para los comandos de esta guía. Las variables pertenecen
a cada terminal: repite el bloque inicial al abrir una nueva.

## 2. Entrega este encargo a Claude Code

Copia este texto en su conversación:

```text
Vamos a conducir exclusivamente P2.T3d usando
docs/playbooks/P2_T3D_ORCHESTRATOR_RUNBOOK.md.

Yo soy el orquestador principal y propietario de las decisiones y aprobaciones.
Tú eres mi ayudante de orquestación y arquitectura y el agente de registro:
caller=claude-code. Lee AGENTS.md, el workflow canónico, las políticas aplicables,
el plan P2 y las entradas vigentes de T3c/T3d antes de actuar.

Esta guía se ejecuta con T3c cerrada como precondición. Toma su implementación
y evidencias como entrada y prepara el preflight y la matriz de certificación.
Usa la tarea existente: no crees otro plan paralelo ni renombres su ID.

Mantén un checklist visible y docs/audit/mvp0-p2p-p2-t3d-handoff.md actualizado
después de cada fase, con evidencia, aprobación vigente, bloqueos y próximo paso.
Solo una fase in_progress normalmente. Conserva el trabajo preexistente.

Prepara decisiones concretas para mí; no me atribuyas aprobaciones ni verificación
final. Resuelve ejecutores y revisores con el RRI real y el workflow vigente.
Tu ayuda arquitectónica no sustituye el rol local obligatorio de ADR-038,
la revisión independiente, ni autoriza ampliar alcance. Si escribes pruebas,
no puedes ser su revisor independiente.

No hagas commit, push, PR, despliegue, borrados ni empieces T4c por este encargo.
```

Tú controlas las aprobaciones y la decisión final. Claude prepara y registra los
pasos; el implementador escribe las pruebas dentro del alcance; el revisor
independiente emite su veredicto. Los nombres concretos se resuelven al calcular
el RRI, conservando cualquier pin vigente de la tarea.

## 3. Carga el resultado de T3c como entrada

En la terminal:

```bash
sed -n '/^## P2.T3c /,/^## P2.T4 /p' "$DUB_TASK_LEDGER"
rg -n 'T3c|T3d|T4c' "$DUB_TASK_PLAN" docs/plan/roadmap.md
rg --files docs/audit .agent | rg 't3c|t3d'
```

Pide a Claude incorporar al preflight la referencia del cierre de T3c, su
artefacto de fase 2 y disposición, pruebas y verificación final del propietario.
El propósito de este paso es identificar el contrato y snapshot que certificará
T3d. Usa la implementación final de T3c, no las rutas o APIs propuestas antes
de que se implementara. Si quedan referencias antiguas de estado, sincronízalas
con el cierre existente. Si al ejecutar esta guía la precondición aún no se
cumple, pospón T3d hasta que se cumpla; su resolución pertenece al proceso T3c.

## 4. Congela con Claude el paquete de certificación

Pídele:

```text
Prepara docs/audit/mvp0-p2p-p2-t3d-preflight.md sobre la implementación cerrada
de T3c. Registra: referencias del cierre, snapshot base y cambios preexistentes;
API real del publisher y cierre; layout real del paquete; construcción de fixture
ciphertext; acceso al Hyperdrive por identificador público; transporte local
determinista; forma de reconstruir el publisher conservando su raíz; inyección
de fallos mediante seams existentes; captura de logs y limpieza de recursos.

Mapea cada criterio de T3d a una prueba y aserciones concretas. Si falta un seam
imprescindible, identifica el bloqueo y la tarea propietaria; no inventes una API
ni modifiques producción desde T3d. Conserva C0 y las decisiones de T3c.
```

Las únicas rutas de implementación permitidas son:

```text
apps/availability-node/test/publication-contract.test.js
apps/availability-node/test/fixtures.js
```

Los registros de auditoría y estado se actualizan aparte como evidencia del
proceso. No añadas `.ts`, dependencias ni reparaciones a `src/` dentro del diff
de certificación. El sobre `.js` del ledger vigente corrige las rutas históricas
del documento C0; no cambia su contrato.

Revisa esta matriz con Claude antes de congelar el paquete:

| Caso | Evidencia que deben producir las pruebas |
|---|---|
| HP-T3d-1 | Cliente mTLS permitido + paquete ciphertext real → `201`; cerrar y reconstruir publisher con la misma raíz → replay `200`, mismos identificadores, digest y `confirmed_at`. |
| HP-T3d-2 | Abrir/leer el Hyperdrive por el identificador público; comparar inventario y bytes con manifiesto y ciphertext declarados. Sin decisión de readiness ni autorización de negocio. |
| EC-T3d-1 | Sin certificado o con certificado no confiable → fallo TLS; identidad autenticada no permitida → `403`; cero llamadas al publisher. |
| EC-T3d-2 | Conflicto de lineage/digest → `409`. Traversal, escape por symlink, manifiesto corrupto, tamaño/hash incorrecto o contenido plaintext/secreto del paquete → `422`; sin segundo drive ni nueva evidencia de éxito. |
| EC-T3d-3 | Fallos inyectados de almacenamiento/red → `503`; inspección de logs, respuestas, metadatos persistidos y archivos del drive sin elementos de la deny-list C0. |
| Integración | Todas las pruebas T3a/T3b/T3c/T3d pasan juntas; cierre y limpieza deterministas; sin dependencia de Internet público. |

Respeta también el `400 invalid_contract` de C0 para peticiones malformadas:
la expectativa `422` del paquete no convierte cualquier campo HTTP prohibido
en un error de paquete. Usa valores secretos sintéticos como canarios y comprueba
su ausencia en las salidas, no solo la ausencia de nombres de campos.

Un stub no demuestra publicación ni lectura real de Hyperdrive. Los dobles de
prueba sirven para provocar fallos mediante interfaces existentes. Congela cómo
se conecta el lector al escritor local sin depender de descubrimiento público.
No exijas señales de PostgreSQL o `P2P_READY`: pertenecen a tareas posteriores.

## 5. Calcula el RRI y fija quién hace cada fase

Pide a Claude medir/justificar `C`, `T`, `A`, `X`, `D`, `K`, `P` con la rúbrica,
puntuar el padre e intentar hojas Low que se puedan verificar por separado.
Que sean pruebas en dos archivos no las convierte automáticamente en Low.
No separes la invariante de certificación para reducir artificialmente el RRI.

Claude debe entregarte el comando ya rellenado. Este es su formato, **no lo
ejecutes con los marcadores**:

```bash
python3 scripts/rri.py \
  --touches apps/availability-node/test/publication-contract.test.js \
  --touches apps/availability-node/test/fixtures.js \
  --C <C> --T <T> --A <A> --X <X> --D <D> --K <K> --P <P> \
  | tee "$DUB_TASK_AUDIT_DIR/parent-rri.md"
```

Usa `--cc` si existe medida de complejidad bruta; `--C` requiere justificación.
Aplica las penalizaciones que correspondan. Conserva el informe completo en
`docs/audit/mvp0-p2p-p2-t3d-rri.md`, incluidas las puntuaciones de hojas si existen.
Después define `export DUB_PARENT_RRI='valor-final-real'` en la terminal.

| Resultado | Tu siguiente paso |
|---|---|
| 0–25 | Ruta Low sin tarjeta completa; fase 1 y revisión de código siguen aplicando. |
| 26–40 | Fase 1 y tarjeta para tu aprobación; implementación local-first según elegibilidad. |
| 41–55 | Fase 1, aprobación y ruta de refinamiento ADR-038; Claude prepara el paquete requerido. |
| 56+ | Descomponer antes de implementar; aprobación y revisión independiente entre proveedores. |

El padre gobierna aprobación, revisión integrada y Reflection aunque haya hojas
Low. Claude registra responsables concretos, límites de reparación y rutas de
fallback según el workflow, sin sustituirlos por «Claude lo hace todo».

**Si Codex no tiene tokens:** Claude puede seguir ayudándote, pero si la banda
exige un revisor Codex independiente debe registrarse su indisponibilidad y
aplicarse la cadena D14. Una sesión que implementó no puede autoaprobarse.
Un D14 del mismo proveedor solo cabe como fallback degradado documentado,
contextualmente aislado, después de fallar la alternativa entre proveedores.

## 6. Prepara el entorno local que necesita T3d

T3d debe construir su publisher, servidor mTLS temporal y transporte local desde
las pruebas. No requiere levantar PostgreSQL, Redis, MinIO ni el stack de producto
completo. `npm start` por sí solo no configura ni acredita este escenario.

Después de cerrar T3c, comprueba las dependencias instaladas:

```bash
npm --prefix apps/availability-node ls --depth=0
```

Si falta la instalación, usa el lockfile cerrado de T3c y su procedimiento
registrado; no cambies versiones ni permitas regenerar el lockfile como atajo.

Ollama es un entorno distinto: se necesita cuando la ruta use modelos locales.
Antes de su primera llamada en T3d, pide a Claude registrar el precheck y comprueba:

```bash
pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|gemma-code-review|peer-workflow-review' || true
pgrep -fl ollama || true
lsof -nP -iTCP:11434 -sTCP:LISTEN || true
```

Si no hay otra tarea usando Ollama, reinícialo una vez para T3d:

```bash
export DUB_OLD_OLLAMA_PID="$(pgrep -x ollama | paste -sd, -)"
osascript -e 'quit app "Ollama"'
```

Espera a que ese servidor termine; después:

```bash
open -a Ollama
pgrep -fl ollama
lsof -nP -iTCP:11434 -sTCP:LISTEN
curl -fsS http://127.0.0.1:11434/api/ps | jq .
curl -fsS http://127.0.0.1:11434/api/tags | jq -r '.models[].name'
```

Claude compara PID anterior/nuevo y calienta cada modelo efectivo con el perfil
de producción, usando el ejemplo de
[Step 0 del archivo de detalle](../audit/agent-workflow-guide-detail-archive.md).
Exige respuesta no vacía y `done_reason: stop`; registra modelo, contexto,
presupuesto de salida y resultado. Si hay contenido vacío, sigue la recuperación
de recursos del workflow; no repitas indefinidamente el mismo perfil.
No reinicies ni mates una ejecución ajena.

## 7. Obtén fase 1 y aprueba el alcance cuando corresponda

Pide a Claude crear `.agent/p2-t3d/phase1-packet-v1.md` con el preflight, alcance,
matriz, RRI, rutas de implementación/revisión, comandos y condiciones de parada.
Para RRI 26+, debe evaluar la hipótesis CWE aplicable para Antares o registrar
el skip tipado; Antares es asesor y no sustituye al revisor.

Comprueba la ruta sin invocar modelos:

```bash
python3 scripts/peer-workflow-review.py \
  --phase task --rri "$DUB_PARENT_RRI" --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase1-packet-v1.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase1-review-v1.json" --dry-run
```

Claude debe contrastar el resultado con la guía canónica y la rama efectiva del
script: existen etiquetas antiguas en su ayuda y referencias distintas a modelos
en documentos secundarios. El `dry-run` orienta, pero no demuestra qué modelo
ejecutó la revisión. Registra el modelo del artefacto real y resuelve discrepancias
antes de atribuir un PASS a la ruta exigida.

Después del precheck local, si aplica, ejecuta la revisión quitando `--dry-run`:

```bash
python3 scripts/peer-workflow-review.py \
  --phase task --rri "$DUB_PARENT_RRI" --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase1-packet-v1.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase1-review-v1.json"
jq . "$DUB_TASK_AUDIT_DIR/phase1-review-v1.json"
```

Lee el veredicto y todos los hallazgos. Claude registra su disposición y la línea
`Task-analysis review: <revisor-real> <artefacto> - <PASS|BLOCKED>`.
Un paquete cambiado necesita nueva versión y revisión; conserva el anterior.

Si aparece `awaiting_fallback_selection`, Claude te muestra el paquete y recibo
`fallback-selection-v1`. Tú eliges modelo, esfuerzo y selector; Claude valida
el hash y prepara la invocación exacta conforme a ADR-039. No inventes PASS ni
confundas un recibo autorizado con una revisión ejecutada. La preautorización
solo vale si esos datos estaban congelados en la tarjeta o preflight.

Con fase 1 resuelta y RRI 26+, pide la tarjeta de seis bloques de
`docs/templates/compact-approval-task-card.md`, incluidos ambos diagramas de
desarrollo y los responsables reales. Debe terminar con:

```text
Execution has not started. Approve this task to proceed.
```

Cuando hayas revisado la tarjeta, puedes responder así, sustituyendo la versión:

```text
Apruebo ejecutar P2.T3d con el alcance, RRI, hojas y rutas de la tarjeta <versión>.
Registra esta aprobación y continúa hasta la verificación final que me corresponde.
```

Si ya existe aprobación explícita para ese mismo alcance, Claude la cita y
reanuda desde allí; no necesitas aprobarla otra vez por cambiar de sesión.

## 8. Da la orden de implementar las pruebas

Copia esto una vez satisfechas las puertas anteriores:

```text
Ejecuta P2.T3d por la ruta aprobada. Entrega a cada implementador su paquete
revisado, rutas exactas, interfaz congelada y pruebas. Mantén el checklist.
Solo se escriben los dos archivos de certificación autorizados.

No debilites aserciones para obtener verde. Si una prueba descubre un defecto
del producto o falta un seam indispensable, conserva la reproducción, registra
T3d blocked y prepara la reapertura de T3a, T3b o T3c con alcance/RRI propios.
No repares producción en esta tarea. Tras reparar y cerrar su propietaria,
vuelve a verificar el snapshot y reanuda T3d desde la fase afectada.
```

Las pruebas pueden pasar desde su primera ejecución porque certifican código
ya implementado. No fabriques RED modificando producción. Acredita que las
aserciones distinguen fixtures válidas e inválidas y registran el comportamiento
real, incluidos los fallos inyectados.

Después de cada hoja inspecciona los dos archivos, incluidos los nuevos que
`git diff` todavía no muestre, y contrasta con el inventario preexistente:

```bash
git status --short
git diff --check
git diff -- apps/availability-node
```

## 9. Ejecuta las pruebas y conserva las salidas

Define esta función en tu terminal para guardar salida y código de retorno:

```bash
p2_t3d_run() (
  set -o pipefail
  dub_log_label="$1"
  shift
  dub_log_path="$(mktemp "$DUB_TASK_AUDIT_DIR/${dub_log_label}.XXXXXX")" || exit 1
  "$@" 2>&1 | tee "$dub_log_path"
  dub_command_rc=$?
  printf '\nexit_code=%s\n' "$dub_command_rc" | tee -a "$dub_log_path"
  printf 'log=%s\n' "$dub_log_path"
  exit "$dub_command_rc"
)
```

Ejecuta en orden y detente ante el primer fallo; no uses una compilación anterior
para certificar fuentes nuevas:

```bash
p2_t3d_run typecheck npm --prefix apps/availability-node run typecheck
p2_t3d_run build npm --prefix apps/availability-node run build
p2_t3d_run focused node --test apps/availability-node/test/publication-contract.test.js
p2_t3d_run integrated node --test apps/availability-node/test/*.test.js \
  docs/audit/mvp0-p2p-p2-t3a-contract.test.js \
  docs/audit/mvp0-p2p-p2-t3a-http.test.js
p2_t3d_run diff-check git diff --check
p2_t3d_run qa-docs make qa-docs
```

Los dos tests bajo `docs/audit/` son preexistentes; los nuevos permanecen en
`apps/availability-node/test/`. Pide a Claude adjuntar también el inventario y
comparación del drive, replay tras reconstrucción y escaneo negativo de secretos.
Una suite verde no sustituye esas aserciones. No registres valores secretos reales.

## 10. Obtén revisión independiente y completa el cierre

T3d es **desarrollo/certificación**, por tanto requiere fase 2 aunque solo añada
tests. Pide a Claude preparar `.agent/p2-t3d/phase2-packet-v1.md` con el diff
completo (también archivos nuevos), aceptación aprobada, transcripciones,
inspección del drive, prueba de replay, escaneo negativo y riesgos pendientes.
Incluye el touchpoint Antares posterior o su skip tipado cuando corresponda.

```bash
python3 scripts/peer-workflow-review.py \
  --phase code --rri "$DUB_PARENT_RRI" --caller "$DUB_CALLER" \
  --task-id "$DUB_TASK_ID" \
  --content "$DUB_TASK_AUDIT_DIR/phase2-packet-v1.md" \
  --artifact "$DUB_TASK_AUDIT_DIR/phase2-review-v1.json"
jq . "$DUB_TASK_AUDIT_DIR/phase2-review-v1.json"
```

Claude debe aplicar el contrato completo de la banda, incluidos N-pass cuando
corresponda, lectura de todos los buckets de findings, disposición y recibo
`Review artifact:`. Ejecutar el wrapper una vez no dispensa esos requisitos.
La línea del cierre será
`Code-solution review: <revisor-real> <artefacto> - <PASS|BLOCKED>`.

Después de resolver fase 2, completa en este orden:

1. Claude registra Reflection: 2 pasadas para Moderate, 3 para Med-high,
   4 para Complex; bandas superiores siguen sus requisitos tras descomponer.
   Cada pasada incluye Draft → Critique → Revise y los hallazgos del revisor.
   Low registra la disposición de la revisión sin exigir este bloque separado.
2. Claude completa la tabla `Behavioral coverage certification`: Case ID,
   Type, Behavior, Layer, Executable evidence, Result. Debe cubrir ambos HP,
   los tres EC y todas sus subcondiciones con pruebas reales `passed`.
3. Tú revisas la tabla, los resultados y las pruebas que los sustentan.
   Repite los comandos que necesites y confirma la verificación final.

Si Reflection causa una modificación material, repite las pruebas afectadas
y fase 2 sobre el nuevo paquete antes de certificar. No cierres con revisión
antigua, fallos o casos omitidos.

Tu confirmación final puede ser:

```text
He verificado los happy paths y edge cases de P2.T3d contra la evidencia
ejecutable de la tabla y los resultados indicados. Registra mi nombre, fecha
y los comandos exactos comprobados en Owner final verification.
```

Claude conserva esa confirmación real en `### Owner final verification` con
los campos y declaración exigidos por el workflow. Nunca la anticipa.

## 11. Ordena sincronizar el estado y termina T3d

Copia esto cuando todos los pasos de cierre estén completos:

```text
Sincroniza el ledger P2, el plan P2 y el resumen de T3 en el roadmap con las
evidencias de cierre. Actualiza la dependencia de T4c según sus dependencias
completas, sin iniciarla. Conserva artefactos RRI, aprobación, implementación,
revisiones y cierre en docs/audit; .agent por sí solo no es el archivo duradero.
Marca T3d Done solo con sus puertas satisfechas. Comprueba que T3a, T3b y T3c
siguen cerradas antes de marcar T3 como completo. No cierres el padre P2.
Ejecuta git diff --check y make qa-docs después de sincronizar.
Entrégame el estado final y el enlace al handoff. Detente al cerrar T3d.
```

No hace falta repetir pruebas de producto por un cambio exclusivamente textual
de estado si el snapshot de código certificado no cambió. Un cambio material
sí obliga a renovar la evidencia afectada. Commit/push requieren su autorización
propia. Al terminar puedes compactar la conversación de Claude cuando hayas
comprobado que el relevo está guardado.

## 12. Monitoriza y retoma si se agotan tokens

Define esta función una vez por terminal:

```bash
p2_t3d_status() {
  git status -sb
  git diff --stat
  sed -n '/^## P2.T3d /,/^## P2.T4 /p' "$DUB_TASK_LEDGER"
  if test -f docs/audit/mvp0-p2p-p2-t3d-handoff.md; then
    sed -n '1,220p' docs/audit/mvp0-p2p-p2-t3d-handoff.md
  fi
  pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|gemma-code-review|peer-workflow-review' || true
  lsof -nP -iTCP:11434 -sTCP:LISTEN || true
  rg --files --hidden "$DUB_TASK_AUDIT_DIR" | sort
}

p2_t3d_status
```

Este comando inspecciona; no lanza otro ejecutor ni avanza puertas. Para seguir
una ejecución, usa `tail -n 60 -f <ruta-real-del-log>` en otra terminal.
`Ctrl-C` termina ese seguimiento, no el proceso observado. No dupliques un
runner que aún está trabajando.

Claude debe mantener este formato de relevo, rellenado con hechos tras cada fase:

```text
Task: P2.T3d
Principal: <tu nombre>
Assistant / caller: Claude Code / claude-code
Updated: <fecha y hora>
Branch / HEAD / snapshot: <valores reales>
Pre-existing changes: <inventario o referencia>
T3c closure evidence: <rutas del cierre usado como entrada>
Frozen scope / packet version: <rutas y versión>
Parent / leaf RRI: <informes>
Approval: <texto real, autor, fecha y paquete; o pending/n/a>
Checklist: <cada fase con responsable y estado; una in_progress>
Latest tests / reviews: <comandos, códigos de retorno y artefactos>
Active runner: <PID, directorio, log; o none>
Fallback selection: <recibo y estado; o not triggered>
Blocker: <motivo concreto; o none>
Next step: <número de esta guía y comando o mensaje exacto>
```

Si cambia la sesión, vuelve al paso 1 y pega este mensaje en Claude:

```text
Retoma P2.T3d como mi ayudante de orquestación y arquitectura. Lee su runbook,
docs/audit/mvp0-p2p-p2-t3d-handoff.md y la entrada vigente del ledger.
Contrasta el registro con HEAD, cambios sin commit, procesos activos y evidencias.
Reanuda en el primer paso incompleto; conserva aprobaciones válidas del mismo
alcance y no repitas una ejecución activa. Si falta evidencia, registra pendiente.
Yo sigo siendo el orquestador principal y quien confirma el cierre.
```

## Referencias rectoras

1. [Ledger P2](../tasks/mvp0-p2p-p2-encrypted-publication.md) — definición vigente T3d.
2. [Plan P2](../plan/mvp0-p2p-p2-encrypted-publication.md) — dependencias y alcance.
3. [Workflow](AGENT_WORKFLOW_GUIDE.md) — puertas, resolución de modelos y cierre.
4. [HITL](../policies/HITL_AUTONOMY_POLICY.md) y [RRI](../policies/RRI_POLICY.md).
5. [Contrato C0](../audit/mvp0-p2p-p2-c0-contract-freeze.md) — semántica de publicación.
6. [ADR-039](../adr/ADR-039-human-selected-fallback-model-checkpoint.md) — selección de fallback.
7. [Tarjeta compacta](../templates/compact-approval-task-card.md).

La creación de esta guía es documentación: fase 1 y fase 2 `n/a` por exención.
Esa exención no se aplica a ejecutar P2.T3d.
