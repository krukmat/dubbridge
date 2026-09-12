---
type: Playbook
title: "Manual de reemplazo del orquestador humano"
---

# Manual de reemplazo del orquestador humano

## Propósito

Este es el relevo operativo para una persona que sustituye al orquestador
principal cuando el agente actual no puede continuar (por ejemplo, porque ha
agotado su presupuesto de tokens). Convierte el flujo canónico del repositorio
en un procedimiento ordenado y copiable. No sustituye las fuentes rectoras:

1. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — máxima autoridad sobre el proceso.
2. `docs/policies/HITL_AUTONOMY_POLICY.md` — límites de aprobación y autonomía.
3. `docs/policies/RRI_POLICY.md` — detalle del cálculo RRI y enrutamiento.
4. `AGENTS.md` — contrato compacto de presentación de tareas.
5. El plan del segmento activo, su registro de tareas, los ADR aplicables y las
   fuentes de producto/BDD.

Si este manual contradice alguno de esos documentos, sigue primero la guía de
flujo y corrige después este manual. Los modelos locales solo actúan como
implementadores o revisores consultivos. No pueden aprobar el alcance, aceptar
su propio trabajo, crear commits, hacer push ni cerrar una tarea en nombre de la
persona responsable.

## Estado del relevo — 2026-09-09

Esta instantánea está fechada deliberadamente. Vuelve a ejecutar los comandos de
la sección siguiente antes de confiar en ella.

- Copia de trabajo: `feature/p2p-mvp-core` en `6b27c2b`; al capturar este
  estado, el árbol estaba limpio y sincronizado con
  `origin/feature/p2p-mvp-core`.
- Relación con `main`: la rama de funcionalidad está 191 commits por delante y
  cero por detrás. No hay ninguna pull request abierta. No hagas merge, rebase
  ni abras una PR por inferencia.
- CI: la ejecución más reciente de `ci` para `feature/p2p-mvp-core` pasó en
  `6b27c2b`; la ejecución más reciente de CI en `main` también pasó. Había varias
  ejecuciones `push-review` de `main` en cola; inspecciónalas en vez de asumir que
  terminaron.
- Línea activa de producto: MVP0-P2P P2. `P2.T2`, `P2.T3a`, `P2.T3b` y
  `P2.T4a` están terminadas. `P2.T3c` es el siguiente candidato documentado,
  pero aún figura como planificado, muestra `RUN BEFORE EXECUTION` en la columna
  RRI y no está autorizado.
- Objetivo de `P2.T3c`: persistencia de seed/open de Hyperdrive e idempotencia y
  comportamiento ante conflictos para la misma identidad. Las rutas de escritura
  congeladas son
  `apps/availability-node/src/hyperdrive_store.ts` y
  `apps/availability-node/src/server.ts`. Su dependencia `P2.T3b` está terminada.
- El trabajo de S-230 en Digital Ocean sigue teniendo menor prioridad que el
  desarrollo local; `T7local` es la siguiente tarea S-230 sin iniciar. No cambies
  de línea de trabajo sin una decisión explícita del propietario.
- El daily más reciente es `docs/daily/2026-08-31.md`; por tanto, el registro
  diario no está lo bastante actualizado para determinar por sí solo la siguiente
  tarea.

## 1. Iniciar cada sesión de reemplazo

Ejecuta lo siguiente desde una terminal nueva:

```bash
cd /Users/matias/dubbridge

# El /usr/bin/python3 de esta máquina es 3.9 y no puede importar al menos
# scripts/peer-workflow-review.py. Da prioridad al Python 3.11 de Homebrew.
export PATH="/opt/homebrew/opt/python@3.11/libexec/bin:$PATH"

python3 --version
git status -sb
git branch --show-current
git rev-parse --short HEAD
git log -5 --oneline --decorate
python3 scripts/agent-preflight.py --print-summary
```

En esta máquina se espera Python 3.11.x. Detente si `python3` muestra 3.9: el
script de revisión entre pares falla al importarse con 3.9. No lo evites
saltándote la revisión.

Inspecciona el árbol de trabajo antes de modificarlo:

```bash
git status --short
git diff --stat
git diff
git diff --cached
git worktree list
```

Los cambios sin commit y los worktrees existentes pertenecen a sus autores.
Consérvalos. Nunca uses `git reset --hard`, `git checkout -- <ruta>`, `git clean`
ni elimines un worktree como atajo de recuperación.

## 2. Obtener el estado en unos 60 segundos

Pega esta función de diagnóstico una vez por sesión de terminal:

```bash
dub_status() {
  git status -sb
  git log -5 --oneline --decorate
  git rev-list --left-right --count origin/main...HEAD
  gh pr status
  gh run list --branch "$(git branch --show-current)" --limit 6
  gh run list --branch main --limit 6
  pgrep -fl 'ollama|delegate-low-rri|run_local_task|gemma-code-review|peer-workflow-review' || true
  lsof -nP -iTCP:11434 -sTCP:LISTEN || true
  curl -fsS http://127.0.0.1:11434/api/ps | jq '{models: [.models[]? | {name, size, expires_at}]}' || true
  find docs/reports/push-review -type f -print 2>/dev/null | sort | tail -n 5
}

dub_status
```

Para monitorización continua en macOS (deténla con `Ctrl-C`):

```bash
while :; do clear; date; dub_status; sleep 15; done
```

Comandos útiles de GitHub:

```bash
gh run list --branch "$(git branch --show-current)" --limit 20
gh run watch <run-id> --exit-status
gh run view <run-id> --log-failed
gh pr status
```

No expongas `gh auth token`, archivos de entorno, certificados, claves de firma,
URL de bases de datos ni paquetes de Ollama que contengan secretos en logs o
artefactos de revisión.

## 3. Abrir o actualizar el control diario

El daily es un control de detección, no una autorización para implementar. Créalo
solo cuando estés operando activamente el repositorio:

```bash
bash scripts/daily-open.sh
```

No uses `--force` solo porque ya exista un daily: sobrescribiría notas humanas.
Abre `docs/daily/YYYY-MM-DD.md` y completa:

1. el foco del día;
2. pipelines de GitHub rotos o en cola;
3. el resultado más reciente de push-review;
4. bloqueos, desviaciones, deuda y riesgos activos;
5. decisiones humanas pendientes;
6. la reconciliación de cierre.

Al abrir y cerrar el día, inspecciona la salida más reciente de push-review:

```bash
find docs/reports/push-review -type f -print 2>/dev/null | sort | tail -n 20
gh run list --workflow push-review --limit 20
```

Un push-review en cola o fallido no es un veredicto de revisión de código. Un
patch delegado permanece `in_review` hasta que se registre la revisión
independiente posterior al desarrollo.

## 4. Seleccionar la siguiente tarea sin adivinar

Sigue este orden:

```bash
sed -n '1,360p' docs/plan/roadmap.md
sed -n '1,320p' docs/plan/<segmento-activo>.md
sed -n '1,360p' docs/tasks/<segmento-activo>.md
rg -n 'Planned|In progress|BLOCKED|RUN BEFORE EXECUTION|next' \
  docs/plan/<segmento-activo>.md docs/tasks/<segmento-activo>.md
```

Antes de congelar una tarea, comprueba todo lo siguiente contra los archivos, no
contra tu memoria:

- estado de predecesores y puertas de dependencia;
- objetivo exacto, rutas permitidas y exclusiones explícitas;
- criterios de aceptación y, para desarrollo, al menos un `HP-#` y un `EC-#`;
- comandos exactos de verificación;
- evidencias que emitir y artefactos de estado que sincronizar;
- fuentes de arquitectura, seguridad, gobernanza, migraciones, BDD, producto o
  diseño que restrinjan el trabajo;
- `DESIGN.md` en la raíz para cambios de interfaz o presentación móvil.

Para el candidato actual del relevo, empieza aquí:

```bash
sed -n '2137,2188p' docs/tasks/mvp0-p2p-p2-encrypted-publication.md
sed -n '2410,2605p' docs/tasks/mvp0-p2p-p2-encrypted-publication.md
sed -n '175,255p' docs/plan/mvp0-p2p-p2-encrypted-publication.md
sed -n '117,170p' docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
sed -n '253,265p' docs/audit/mvp0-p2p-p2-c0-contract-freeze.md
sed -n '1,220p' docs/audit/mvp0-p2p-p2-t3-readiness-2026-09-09.md
sed -n '1,260p' docs/adr/ADR-044-p2p-audience-delivery-boundary.md
sed -n '1,260p' apps/availability-node/src/server.ts
sed -n '1,220p' apps/availability-node/src/contract.ts
```

Si falta el plan o el registro de tareas, o son insuficientes, complétalos antes
de tocar código fuente. La petición del usuario de ejecutar directamente este
manual solo eximió de crear un plan separado para esta tarea de documentación;
no exime del contrato normal de plan y tareas para el trabajo posterior de
producto.

## 5. Orquestar una tarea existente a partir de su ID

Si el propietario te entrega un ID concreto, ese ID sustituye el paso de elegir
la siguiente tarea: no selecciones otra ni interpretes que mencionar una tarea
autoriza su implementación. Localiza primero su única definición canónica:

```bash
export DUB_TASK_ID='P2.T3c'
export DUB_TASK_LEDGER='docs/tasks/mvp0-p2p-p2-encrypted-publication.md'
export DUB_TASK_PLAN='docs/plan/mvp0-p2p-p2-encrypted-publication.md'

rg -n -F "$DUB_TASK_ID" "$DUB_TASK_LEDGER" "$DUB_TASK_PLAN" docs/plan/roadmap.md
sed -n '1,120p' "$DUB_TASK_LEDGER"
git status --short
```

Después aplica esta puerta de preparación antes del RRI, la revisión o cualquier
llamada a un implementador:

1. Confirma que el ID aparece en una tabla de orden/dependencias y en una sección
   propia del registro; una simple mención en el roadmap no es una definición.
2. Confirma que cada predecesor está realmente `Done`/`PASS`, con su evidencia y
   verificación final. `Planned`, `In progress` o una prueba verde aislada no
   satisfacen una dependencia.
3. Exige objetivo, tipo, Effort, estado, rutas escribibles exactas, exclusiones,
   criterios de aceptación, comandos de verificación y condición de parada.
4. Para desarrollo, exige al menos un `HP-#` y un `EC-#`, la capa y ruta de prueba
   previstas, `Evidence to emit`, `Status artifacts affected` y un handoff breve.
5. Comprueba que las rutas propuestas bastan de verdad: dependencias directas
   requieren sus manifiestos/lockfiles, y un comportamiento nuevo necesita una
   ruta de prueba ejecutable. No uses una lista histórica incompleta como límite
   ficticio.
6. Comprueba que no quedan decisiones de arquitectura, seguridad, persistencia o
   autoridad que el implementador tendría que inventar. Si quedan, crea o ejecuta
   primero una tarea de preflight/contrato solo documental.

Clasifica el resultado con estas expresiones exactas en el relevo o checklist:

- `NO PREPARADA`: falta alguna definición o hay una decisión sin resolver. Solo
  se permite completar plan/ledger/contrato; no ejecutes RRI como si el alcance
  estuviera congelado ni prepares una tarjeta de implementación.
- `PREPARADA PARA ORQUESTAR`: objetivo, dependencias y preguntas de preflight
  están acotados, de modo que se puede iniciar el análisis documental. Si alguna
  pregunta sigue abierta, no pases todavía a RRI ni a implementación.
- `ALCANCE CONGELADO`: rutas, criterios, pruebas, evidencia, exclusiones y
  decisiones necesarias ya están resueltos. Calcula ahora el RRI actual.
- `LISTA PARA PRESENTAR/EJECUTAR`: RRI actual y fase 1 resueltos. Si el RRI es
  26+, presenta la tarjeta y espera aprobación; si es Low, sigue su ruta directa.
- `AUTORIZADA`: existe aprobación válida para esta tarea y esta sesión cuando la
  banda la exige. Aun así, no autoriza commit, push, borrado ni ampliar alcance.

Usa este resumen repetible mientras orquestas el ID seleccionado:

```bash
dub_task_status() {
  printf 'task=%s\nledger=%s\nplan=%s\n' \
    "$DUB_TASK_ID" "$DUB_TASK_LEDGER" "$DUB_TASK_PLAN"
  rg -n -F "$DUB_TASK_ID" "$DUB_TASK_LEDGER" "$DUB_TASK_PLAN" docs/plan/roadmap.md
  git status -sb
  git diff --stat
  pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|peer-workflow-review' || true
  find .agent -maxdepth 2 -type f -iname "*${DUB_TASK_ID//./-}*" -print 2>/dev/null | sort
}

dub_task_status
```

Si la tarea ya estaba `In progress`, reconstruye primero su checklist y compara
artefactos, diff y pruebas según la sección 14; nunca repitas una fase solo porque
la sesión anterior terminó. Si estaba `Planned`, empieza en análisis y pasa a RRI
solo cuando el alcance quede congelado. Si estaba `Done`, no la reabras sin una
orden explícita y un nuevo alcance.

### Cola preparada en este relevo

- `P2.T3c`: `PREPARADA PARA ORQUESTAR`, no aprobada. Su definición ampliada está
  en el registro de P2; el primer trabajo es resolver sus cinco preguntas de
  preflight. Solo después se congelan alcance, RRI y fase 1. Para ejecutar esta
  tarea sin mezclarla con T3d, sigue
  `docs/playbooks/P2_T3C_ORCHESTRATOR_RUNBOOK.md`.
- `P2.T3d`: definición preparada, pero bloqueada por la finalización de `T3c`.
  Sigue [su guía específica con Claude Code como ayudante](P2_T3D_ORCHESTRATOR_RUNBOOK.md)
  cuando exista evidencia de cierre de T3c; tú conservas la orquestación principal.
  No debe presentarse ni ejecutarse antes.
- `P2.T4b`: su dependencia `T4a` está terminada, pero su entrada sigue siendo
  solo una fila resumida; clasifícala `NO PREPARADA` hasta añadir su ficha propia.
  Además permanece fuera de la línea seleccionada mientras se cierre T3. No la
  conviertas en trabajo concurrente implícito.

## 6. Calcular el RRI del padre coherente y de las hojas ejecutables

Primero congela el resultado coherente y sus invariantes. Después realiza el pase
honesto de maximización de banda Low: divide únicamente en límites reales de
comportamiento, propiedad, evidencia o decisión. Nunca reduzcas puntuaciones,
omitas acoplamiento ni fragmentes una invariante solo para alcanzar Low.

Ejecuta la calculadora; no calcules el RRI manualmente:

```bash
python3 scripts/rri.py \
  --touches <ruta-1> --touches <ruta-2> \
  --cc <CC-bruta-mas-alta-de-las-funciones-modificadas> \
  --D <0-5> --K <0-5> --P <0-5> \
  --T <0-5> --A <0-5> --X <0-5> \
  [--penalty refactor_and_behavior] \
  [--penalty arch_decision] \
  [--penalty no_verification]
```

Guarda la salida sin modificar en el registro de tareas o en un artefacto RRI
enlazado. Puntúa tanto el envoltorio padre como cada hoja ejecutable. Vuelve a
calcular si cambian el alcance, las rutas, las invariantes o la verificación.

El enrutamiento resultante es:

| RRI | Banda | Aprobación | Autoría predeterminada | Reflection |
|---|---|---|---|---|
| 0–25 | Low | Sin tarjeta completa de aprobación | Ejecución directa por la persona/agente principal; `qwen3.8:27b-mlx` opcional solo para patches de código simples y acotados | Sin registro separado |
| 26–40 | Moderate | Aprobación humana explícita | `devstral-small-2:24b-instruct-2512-q4_K_M` en un worktree desechable; como máximo dos reparaciones respaldadas por evidencia | 2 pases |
| 41–45 | Med-high | Aprobación humana explícita | Puerta ADR-038; `GO_LOCAL` usa la ruta local Moderate y `CLOUD_REQUIRED` la ruta cloud aprobada | 3 pases |
| 46–55 | Med-high | Aprobación humana explícita | Puerta de evidencia ADR-038; sin intento local de la tarea completa; descomponer hojas Low antes del residuo cloud | 3 pases |
| 56–70 | Complex | Aprobación humana explícita | Descomponer primero y después usar la implementación cloud aprobada | 4 pases |
| 71–85 | High | Aprobación más revisión humana del diff | Descomponer primero; implementación cloud | Seguir la guía canónica |
| 86+ | Very high / Excessive | Decisión humana de arquitectura | ADR, análisis de riesgo y replanteamiento; sin implementación directa | Seguir la guía canónica |

Los disparadores obligatorios de descomposición incluyen RRI final 56+, superficies
grandes y muy acopladas, lógica compleja con riesgo de dominio, refactor mezclado
con cambio de comportamiento y trabajo de alto impacto sin pruebas. Consulta la
lista vigente completa en `docs/policies/RRI_POLICY.md § Decomposition triggers`.

## 7. Ejecutar la revisión de fase 1 y obtener aprobación

El enrutamiento del revisor es independiente de quién escribirá el código:

| RRI | Cadena de fase 1 y fase 2 |
|---|---|
| 0–25 | `gpt-oss:20b` → `gemma4:26b-a4b-it-qat` → D14 |
| 26–55 | `gemma4:26b-a4b-it-qat` → `gpt-oss:20b` → D14 |
| 56+ | Revisor de otro proveedor → D14 |

Prepara un paquete de análisis que contenga el alcance congelado, aceptación,
RRI, rutas permitidas, verificación y referencias rectoras. Después ejecuta:

```bash
python3 scripts/peer-workflow-review.py \
  --phase task \
  --rri <final-rri> \
  --caller unknown \
  --task-id <id-tarea> \
  --content .agent/<id-tarea>-phase1-packet.md \
  --artifact .agent/<id-tarea>-phase1-review.json
```

La revisión de análisis debe indicar PASS antes de presentar o delegar. Si el
paquete cambia de forma material, vuelve a revisar exactamente el paquete nuevo y
conserva un artefacto distinto. Las tareas solo de documentación, configuración,
migración, ADR, plan, registro de tareas o política registran:

```text
Task-analysis review: n/a - <exención exacta>
```

Para RRI 26+, completa los seis bloques de
`docs/templates/compact-approval-task-card.md`, presenta la tarjeta y termina
exactamente con:

```text
Execution has not started. Approve this task to proceed.
```

Ninguna aprobación de una tarea o sesión anterior se hereda. Registra una exención
explícita y acotada si el propietario ordena continuar sin otro punto de control.
La aprobación nunca autoriza borrados, commits, push, despliegues ni ampliar las
rutas salvo que esas acciones estén incluidas explícitamente.

## 8. Mantener la lista de control activa de la tarea

Mantén una lista visible durante toda la ejecución. Normalmente debe haber
exactamente una fase `in_progress`; las puertas fallidas permanecen `blocked`
hasta que se resuelvan.

Para RRI 26+:

```text
[completed] Analizar y acotar — <persona/orquestador>
[completed] Revisión de fase 1 — <revisor real>
[completed] Aprobación — <persona que aprueba>
[in_progress] Implementar — <implementador/modelo real>
[pending] Reflexionar y verificar — <persona/orquestador>
[pending] Revisión de fase 2 — <revisor real>
[pending] Cerrar — <persona/orquestador>
```

Antepon `Restart Ollama + local-stack precheck — <orquestador>` antes de la
primera llamada a un modelo local. El trabajo Low y solo documental puede usar
una lista reducida.

## 9. Reiniciar y comprobar previamente la pila de modelos locales

Hazlo una vez por ID de tarea antes de la primera acción respaldada por Ollama,
incluida la revisión de fase 1. Primero demuestra que ninguna tarea ajena está
usando Ollama:

```bash
pgrep -fl 'delegate-low-rri|run_local_task|run_analysis|gemma-code-review|peer-workflow-review' || true
pgrep -fl ollama || true
lsof -nP -iTCP:11434 -sTCP:LISTEN || true
```

Si hay otra tarea activa, espera a que termine su runner acotado o detenlo según
el contrato de terminación del propio runner. No mates una ejecución ajena.

En este Mac, reinicia la aplicación Ollama y confirma que cambió el PID del
servidor:

```bash
OLD_OLLAMA_PID="$(pgrep -x ollama | paste -sd, -)"
osascript -e 'quit app "Ollama"'
open -a Ollama
sleep 3
NEW_OLLAMA_PID="$(pgrep -x ollama | paste -sd, -)"
printf 'old=%s new=%s\n' "$OLD_OLLAMA_PID" "$NEW_OLLAMA_PID"
lsof -nP -iTCP:11434 -sTCP:LISTEN
curl -fsS http://127.0.0.1:11434/api/ps | jq .
```

Calienta y prueba cada modelo que usará la tarea con un prompt pequeño de revisión
que solo devuelva JSON y con el contexto de producción de la ruta. Ejemplo:

```bash
MODEL='gpt-oss:20b'
NUM_CTX=65536
curl -fsS http://127.0.0.1:11434/api/chat \
  -H 'content-type: application/json' \
  -d "$(jq -n --arg model "$MODEL" --argjson ctx "$NUM_CTX" '{
    model: $model,
    stream: false,
    think: false,
    messages: [{role: "user", content: "Devuelve solo JSON: {\"verdict\":\"PASS\"}"}],
    options: {temperature: 0, num_ctx: $ctx, num_predict: 512}
  }')" \
  | tee .agent/ollama-precheck.json \
  | jq '{done, done_reason, content: .message.content, eval_count}'
```

Exige `done_reason: "stop"` y contenido no vacío. Para Devstral Moderate usa el
contexto predeterminado de 131072 del runner. Si el contenido está vacío, sigue el
orden canónico de recuperación de recursos: descarga el modelo, inspecciona
`/api/ps` junto con `memory_pressure`/`vm_stat`, reintenta una vez con
`think=false`, `temperature=0`, `num_ctx<=16384` y
`num_predict=512..1024`; después reconstruye el paquete real para que quepa en ese
contexto y haz un único reintento acotado. Nunca repitas indefinidamente el mismo
perfil de alto consumo que está fallando.

```bash
ollama stop "$MODEL"
curl -fsS http://127.0.0.1:11434/api/ps | jq .
memory_pressure | head -n 30
vm_stat | head -n 30
```

Registra en el artefacto de la tarea el modelo, contexto, presupuesto de
predicción, modo de razonamiento, motivo terminal, longitud del contenido, estado
de los modelos cargados y decisión de recuperación.

## 10. Implementar mediante la ruta de la banda

### Low: ejecución directa o patch acotado con Qwen

La documentación, planes, registros de tareas, ADR, políticas y trabajo con mucho
peso estructural se ejecutan directamente. Para un patch de código simple, prepara
un paquete con el extracto de la tarea, aceptación, salida RRI, fragmentos
literales relevantes, rutas permitidas, verificación y condiciones de parada. Usa
`full-file` para un archivo nuevo o pequeño y `before-after` para un reemplazo
pequeño dentro de un archivo grande.

```bash
python3 scripts/delegate-low-rri.py \
  --task-id <id-tarea> \
  --attempt 1 \
  --rri <0-25> \
  --mode full-file \
  --allow-path <ruta-relativa-al-repositorio> \
  --out .agent/<id-tarea>-attempt1.json \
  --apply \
  .agent/<id-tarea>-delegation-packet.md
```

Para `before-after`:

```bash
python3 scripts/delegate-low-rri.py \
  --task-id <id-tarea> \
  --attempt 1 \
  --rri <0-25> \
  --mode before-after \
  --target-path <ruta-relativa-al-repositorio> \
  --before-file .agent/<id-tarea>-before.txt \
  --allow-path <ruta-relativa-al-repositorio> \
  --out .agent/<id-tarea>-attempt1.json \
  --apply \
  .agent/<id-tarea>-delegation-packet.md
```

El contenido literal de `--before-file` también debe aparecer en el paquete que
ve el modelo; el wrapper no lo inyecta en el prompt. Inspecciona personalmente las
rutas devueltas y el diff, y después ejecuta los comandos exactos de aceptación.
Permite como máximo un ciclo acotado de reparación. Los códigos de salida
significan: `0`, resultado disponible/aplicado; `124`, bloqueo o límite total de
tiempo; `2`, Ollama no disponible; `1`, fallo de validación, alcance o aplicación.

### Moderate: runner Devstral aislado

Crea un worktree desechable solo después de la aprobación y del PASS de fase 1:

```bash
TASK_ID='<id-tarea>'
TASK_BRANCH="agent/${TASK_ID}"
TASK_WORKTREE="/Users/matias/dubbridge/.agent/worktrees/${TASK_ID}"

git worktree add -b "$TASK_BRANCH" "$TASK_WORKTREE" HEAD
python3 scripts/local-agent/run_local_task.py \
  --card .agent/<id-tarea>-local-card.json \
  --worktree "$TASK_WORKTREE" \
  --out .agent/<id-tarea>-local-run.json
```

Estructura mínima de la tarjeta:

```json
{
  "task_id": "<id-tarea>",
  "spec": "<objetivo, comportamiento, restricciones y condiciones de parada completos y congelados>",
  "acceptance_tests": ["<comando exacto 1>", "<comando exacto 2>"],
  "allowed_paths": ["<ruta-relativa-al-repositorio>"],
  "rri": 30,
  "band": "Moderate"
}
```

Inspecciona el alcance y la verificación antes de integrar nada:

```bash
git -C "$TASK_WORKTREE" status --short
git -C "$TASK_WORKTREE" diff --check
git -C "$TASK_WORKTREE" diff --name-only
git -C "$TASK_WORKTREE" diff
```

Usa como máximo dos reparaciones respaldadas por evidencia. Tras agotar 2/2,
diagnostica y descompón el trabajo restante en hojas Low puntuadas de forma
independiente antes de considerar cloud. No escribas directamente residuo
sustantivo mientras afirmas estar usando la ruta local.

### Med-high: puerta ADR-038

Para RRI 41–55, calcula el hash del paquete aprobado e inmutable, ejecuta el
refinamiento consultivo actual con Qwen3.6, crea un recibo primario con el esquema
exacto que aplica `scripts/local-agent/med_high_gate.py` y después evalúa la
puerta:

```bash
CARD=.agent/<id-tarea>-adr038-packet.json
CARD_HASH="$(shasum -a 256 "$CARD" | awk '{print $1}')"

python3 scripts/local-architect/run_analysis.py \
  --packet "$CARD" \
  --profile med-high-refinement-v1 \
  --expected-packet-sha256 "$CARD_HASH" \
  --output .agent/<id-tarea>-adr038-refinement.json

python3 scripts/local-agent/med_high_gate.py \
  --refinement-artifact .agent/<id-tarea>-adr038-refinement.json \
  --primary-receipt .agent/<id-tarea>-adr038-primary-receipt.json \
  --card-hash "$CARD_HASH" \
  --rri <41-55>
```

El recibo primario debe vincular `card_hash` y el SHA-256 del JSON canónico del
artefacto exacto de refinamiento, y debe contener `primary_id`, `decision`,
`rationale` y `timestamp`. El principal puede degradar `GO_LOCAL` a
`CLOUD_REQUIRED`, pero nunca hacer la promoción inversa. Usa el esquema del script
como autoridad; no copies de un recibo antiguo una etiqueta histórica de modelo.

- RRI 41–45 con `GO_LOCAL`: usa el runner Moderate y su presupuesto de dos
  intentos.
- RRI 41–45 con `CLOUD_REQUIRED`: usa únicamente la ruta cloud congelada en la
  tarjeta.
- RRI 46–55: no hay intento local de la tarea completa. Divide y despacha hojas
  Low honestas; solo el residuo irreducible por encima de Low puede continuar por
  la ruta cloud congelada.

### Complex y superiores

Para RRI 56+, la descomposición y la aprobación explícita preceden a cualquier
edición de código. La autoría cloud usa el modelo concreto y el nivel de
razonamiento congelados en la tarjeta. Las revisiones de fase 1 y 2 deben usar un
proveedor distinto al del orquestador. Si el modelo cloud o sus credenciales no
están disponibles, deja la tarea bloqueada con un paquete de relevo; los revisores
locales no pueden convertirse por sustitución en implementadores.

## 11. Gestionar el fallback terminal sin sustituciones silenciosas

D14 o un implementador cloud requiere un recibo ADR-039
`fallback-selection-v1` vinculado al paquete exacto de fallback. Los runners que
originan el fallo lo emiten. El modo predeterminado es `human-select`; si falta el
modelo, el nivel de razonamiento o la persona que selecciona, se produce
`awaiting_fallback_selection` y el flujo debe detenerse.

Ejemplo de opciones para el runner que encontró el fallo terminal:

```bash
--fallback-mode preauthorized \
--fallback-model <modelo-exacto> \
--fallback-reasoning-effort <low|medium|high|xhigh|max> \
--fallback-selected-by <identidad-humana> \
--fallback-selection-artifact .agent/<id-tarea>-fallback-selection.json
```

Usa `preauthorized` solo si los tres valores ya estaban congelados en la tarjeta
aprobada o en el preflight. En caso contrario, usa `human-select`, informa de la
ruta recomendada y espera. Revalida el digest del paquete, rol, modelo y esfuerzo
justo antes de reanudar. D14 solo revisa; un recibo D14 no autoriza implementación.

## 12. Verificar, revisar y cerrar en orden

Ejecuta primero la aceptación enfocada y después las puertas aplicables del
repositorio. Puertas habituales:

```bash
make qa-fmt
make qa-lint
make qa-test
make qa-check
make qa-docs
make qa-deny
make qa-config-secrets
make qa-roadmap-drift
make qa-maintainability
make qa-python-complexity
make qa-review-budget REVIEW_PATHS='<task paths>'
make qa-mobile
make qa-coverage
make qa-build-release
```

Usa `make qa-ci` únicamente cuando corresponda ejecutar la puerta completa entre
stacks y esté disponible el revisor local requerido. Nunca ocultes un fallo ajeno
o preexistente: reprodúcelo, registra la salida exacta y mantén la tarea bloqueada,
o sepáralo explícitamente por indicación del propietario. Nunca hagas commit con
una prueba obligatoria fallando.

Antes de asignar cualquier estado Done, cierra en este orden:

1. Punto de control Antares posterior a la implementación, solo para tareas RRI
   26+ con una hipótesis CWE justificada, relevante y ya incluida en la watchlist;
   en caso contrario, registra una omisión tipada. Antares es consultivo.
2. Revisión de fase 2 de la solución de código para toda tarea de desarrollo.
3. Registro Reflection: 2 pases para Moderate, 3 para Med-high y 4 para Complex
   56–70; sigue la guía para bandas superiores.
4. Certificación de cobertura de comportamiento que vincule cada `HP-#` y `EC-#`
   a evidencia ejecutable aprobada en la capa `unit`, `component`, `integration`,
   `contract` o `e2e`.
5. Verificación final del propietario con nombre, fecha, declaración y comandos
   exactos.
6. Sincronización del registro de tareas, plan del segmento, roadmap, ADR/índice/
   arquitectura afectados, bloqueos dependientes, artefactos de evidencia y estado
   del relevo.
7. Solo entonces cambia la tarea a `[x] Done` e informa del cierre.

Prepara el paquete de fase 2 con el diff exacto, los criterios de aceptación y los
hechos de pruebas ya verificados, y después ejecuta:

```bash
python3 scripts/peer-workflow-review.py \
  --phase code \
  --rri <final-rri> \
  --caller unknown \
  --task-id <id-tarea> \
  --content .agent/<id-tarea>-phase2-packet.md \
  --artifact .agent/<id-tarea>-phase2-review.json
```

Lee y resuelve cada hallazgo, incluidos los específicos de un pase y los
clasificados como probablemente falsos positivos. Nunca informes de cero hallazgos
basándote únicamente en una etiqueta PASS ni sin comprobar el código de salida del
parser o comando. Las líneas obligatorias del informe son:

```text
Task-analysis review: <gemma|gpt-oss|codex|claude|d14> <artefacto> - <PASS|BLOCKED>
Code-solution review: <gemma|gpt-oss|codex|claude|d14> <artefacto> - <PASS|BLOCKED>
```

El trabajo solo de documentación, configuración, migración, ADR, plan, registro de
tareas o política registra la fase 2 como `n/a` con la exención exacta.

## 13. Crear commits, hacer push y monitorizar solo con autorización explícita

Crear commits y hacer push siempre requiere aprobación explícita. Cuando se
conceda, añade al staging solo las rutas exactas de la tarea e inspecciona el diff
preparado:

```bash
git status --short
git diff --check
git add -- <ruta-exacta-1> <ruta-exacta-2>
git diff --cached --stat
git diff --cached
git commit -m '<tipo>(<alcance>): <resumen>'
git push -u origin "$(git branch --show-current)"
```

Después monitoriza la ejecución exacta:

```bash
gh run list --branch "$(git branch --show-current)" --limit 10
gh run watch <run-id> --exit-status
gh run view <run-id> --log-failed
```

No declares la finalización hasta que coincidan los controles requeridos y los
artefactos de estado. No elimines automáticamente la rama de tarea ni el worktree
desechable. Si más adelante se aprueba la limpieza, determina primero el objetivo
exacto con `git worktree list`.

## 14. Recuperar después de una interrupción o de agotar el contexto

Al reanudar, no vuelvas a ejecutar la implementación a ciegas. Reconstruye el
estado en este orden:

```bash
git status -sb
git diff --stat
git diff
git diff --cached
git worktree list
find .agent -maxdepth 2 -type f -mtime -2 -print 2>/dev/null | sort
find docs/audit -type f -mtime -2 -print | sort
git log -10 --oneline --decorate
gh run list --branch "$(git branch --show-current)" --limit 10
```

Después compara la lista de control activa, el estado del registro de tareas, los
artefactos y el diff real:

- Si falta un artefacto de salida, está mal formado, obsoleto o su digest no
  coincide, la fase correspondiente no ha pasado.
- Si un runner local se detuvo, inspecciona su transcripción JSON y código de
  salida antes de elegir reparación, descomposición o fallback.
- Si las rutas exceden el alcance aprobado, detente y restaura únicamente con una
  decisión explícita del propietario para esas rutas concretas; nunca descartes
  masivamente cambios desconocidos.
- Si falló la aceptación, mantén la fase `blocked` y registra la salida exacta.
- Si la implementación pasó pero falta revisión, cobertura o verificación del
  propietario, la tarea sigue abierta.
- Si una tarea está realmente bloqueada, escribe el bloqueo y la siguiente
  decisión humana requerida en el registro de tareas o daily, en vez de fabricar
  una finalización.

Al entregar la tarea a otra persona o agente, usa únicamente:

1. ID de tarea y objetivo en una línea;
2. rutas del registro de tareas y del plan;
3. archivo y rango exacto de líneas que modificar;
4. criterios de aceptación en viñetas;
5. condición de parada e instrucción explícita de no iniciar la tarea siguiente.

## 15. Cierre del día

```bash
git status -sb
make qa-docs
make qa-roadmap-drift
gh run list --branch "$(git branch --show-current)" --limit 10
gh run list --workflow push-review --limit 10
```

Completa las secciones del daily sobre incidencias, mejoras, decisiones pendientes
y reconciliación. Cada pipeline rojo necesita una persona responsable o una tarea
siguiente. Todo elemento `[~]` debe trasladarse al siguiente día. No hagas commit
del daily ni de ningún código sin aprobación explícita.

En una sesión respaldada por IA, recuerda al operador ejecutar `/compact` después
de cada tarea cerrada o `/clear` cuando ya no necesite su contexto. Esos comandos
los activa la persona o el harness; un agente no puede invocarlos por el usuario.

## Condiciones de parada

Detente y pregunta al propietario del repositorio, en vez de inferir permiso,
cuando ocurra cualquiera de estas situaciones:

- deben ampliarse el alcance, las invariantes, las rutas permitidas o el
  comportamiento de producto;
- una tarea que requiere aprobación carece de aprobación explícita vigente;
- no se autorizó explícitamente un borrado, sobrescritura, commit, push, PR,
  despliegue, migración o escritura externa;
- se agotó la cadena de revisores o está incompleta la selección D14/cloud;
- falla una prueba o puerta obligatoria y la causa no puede separarse con
  seguridad;
- hay decisiones sin resolver sobre seguridad, derechos, consentimiento,
  gobernanza, esquema o arquitectura;
- los archivos actuales contradicen el contrato del plan, tarea o ADR;
- el trabajo sin commit de otro autor se solapa con la tarea.

Un estado bloqueado comunicado fielmente es un resultado aceptable. No lo son una
ampliación silenciosa del alcance, la autorrevisión, reutilizar evidencia obsoleta
ni una escritura externa no aprobada.
