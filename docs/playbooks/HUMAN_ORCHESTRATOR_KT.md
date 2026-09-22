---
type: Playbook
title: Guía del orquestador humano
---

# Guía del orquestador humano

Esta es la única guía operativa del KT. Tu función es decidir el siguiente paso,
comprobar sus puertas y exigir evidencia. Los agentes implementan o revisan; tú
controlas alcance, aprobación, integración y cierre.

La fuente normativa sigue siendo [Agent Workflow Guide](AGENT_WORKFLOW_GUIDE.md).
Consulta [HITL](../policies/HITL_AUTONOMY_POLICY.md) para aprobación y
[RRI](../policies/RRI_POLICY.md) para puntuación. No memorices modelos ni rutas:
usa siempre sus tablas vigentes.

## Inicio de sesión

```bash
cd /Users/matias/dubbridge
export PATH="/opt/homebrew/opt/python@3.11/libexec/bin:$PATH"
python3 scripts/human-orchestrator.py status
python3 scripts/human-orchestrator.py --execute run preflight -- --print-summary
```

Antes de escribir, identifica la tarea, su plan, su ledger, la rama, los
worktrees y los cambios existentes. Todo cambio previo tiene propietario.

El lanzador es una fachada segura sobre scripts existentes:

```bash
# Mostrar un comando sin ejecutarlo
python3 scripts/human-orchestrator.py run rri -- --help

# Ejecutarlo de forma explícita
python3 scripts/human-orchestrator.py --execute run rri -- --help
```

`--execute` va antes de `run`. Los argumentos después de `--` se envían como
argumentos literales. Los alias disponibles son `preflight`, `rri`, `review`,
`low`, `local`, `architect` y `gate`. El lanzador no aprueba, reintenta,
selecciona modelos ni interpreta veredictos.

## Ciclo de una tarea

| Paso | Decisión humana | Evidencia mínima |
|---|---|---|
| 1. Seleccionar | ¿La tarea y sus dependencias están preparadas? | Plan y entrada vigente del ledger |
| 2. Acotar | ¿Qué resultado, rutas y HP/EC se autorizan? | Contrato verificable y archivos permitidos |
| 3. Puntuar | ¿Cuál es el RRI del padre y de las hojas? | Salida real de `scripts/rri.py` |
| 4. Revisar | ¿Pasó fase 1 por la ruta de la banda? | Artefacto ligado al paquete exacto |
| 5. Aprobar | ¿La política exige aprobación y está registrada? | Aprobación del alcance exacto |
| 6. Preparar | ¿La ruta local necesita reinicio/precheck? | PID nuevo, endpoint y warm-up válidos |
| 7. Implementar | ¿Se usa la ruta de la banda? | Ejecución, diff e intentos registrados |
| 8. Cerrar | ¿Pasaron aceptación y fase 2 antes de `Done`? | Review, Reflection, HP/EC y estado sincronizado |

Reglas prácticas:

- Ejecuta `python3 scripts/rri.py --help` y puntúa cada tarea con sus datos
  reales. RRI 26+ requiere la presentación y aprobación definidas por la guía.
- Si interviene un modelo Ollama, reinicia y prueba la pila una vez por tarea
  antes de la primera llamada. No lo hagas para documentación pura.
- Inspecciona `--help`, el paquete y las rutas autorizadas antes de todo comando
  con efectos. Empieza con dry-run cuando exista.
- Un runner terminado no equivale a una tarea aceptada. Examina el artefacto,
  el diff, las pruebas y los hallazgos.
- Commit, push, PR, despliegue, eliminación y escrituras externas requieren la
  autorización aplicable.

## Comandos de referencia

Los nombres son ejemplos; usa el ID y las rutas reales de la tarea.

```bash
# RRI documental de ejemplo; no reutilices sus valores
python3 scripts/rri.py --touches docs/plan/ejemplo.md \
  --C 0 --D 0 --K 0 --P 0 --T 0 --A 0 --X 1

# Comprobar la ruta de revisión sin llamar al modelo
python3 scripts/peer-workflow-review.py --phase task --rri 25 \
  --caller codex --task-id MI-TAREA \
  --content .agent/MI-TAREA/phase1-packet.md --dry-run

# Preparar una propuesta Low sin aplicarla
python3 scripts/delegate-low-rri.py .agent/MI-TAREA/packet.md \
  --task-id MI-TAREA --attempt 1 --rri 25 \
  --mode full-file --allow-path ruta/autorizada.py --dry-run

# Consultar el contrato del runner Moderate
python3 scripts/local-agent/run_local_task.py --help
```

Conserva paquetes y resultados numerados en `.agent/MI-TAREA/`; no sobrescribas
intentos. El estado duradero vive en `docs/tasks/`.

### Limitación conocida

Verificada el 2026-09-19: `peer-workflow-review.py` para RRI 56+ salta al
revisor cross-vendor, mientras la guía canónica exige GPT-OSS Complex primero.
No uses esa ruta hasta resolver la discrepancia o invoca explícitamente la ruta
canónica y conserva su artefacto. El lanzador no corrige esta lógica.

## Recuperación y relevo

| Situación | Acción |
|---|---|
| Contexto perdido | Lee ledger, diff y último artefacto; identifica el último gate pasado |
| Pruebas fallan | Conserva comando/salida y usa solo el presupuesto de reparación vigente |
| Runner inconcluso | Inspecciona proceso, logs y cambios antes de decidir otro intento |
| Fallback pendiente | Emite el recibo ADR-039 para el paquete y selección humana exactos |
| Diff fuera de alcance | Detén la integración y corrige paquete/gates |
| Modelo devuelve vacío | Aplica el protocolo de recuperación de la guía; no repitas a ciegas |

Reanuda desde hechos verificables, no desde el historial del chat:

```text
Task ID / plan / ledger:
Objetivo y rutas autorizadas:
RRI padre y hojas / aprobación:
Rama / HEAD / worktree / cambios preexistentes:
Última fase completada / evidencia:
Intentos, pruebas y procesos activos:
Bloqueo o decisión pendiente:
Siguiente comando exacto / precondiciones:
Documentos por sincronizar:
```

## Práctica para transferir el rol

1. **Low, 20–30 min:** prepara y puntúa una tarea pequeña; explica dry-run,
   implementación, revisión y cierre.
2. **Moderate, 30–45 min:** prepara una tarea aprobable y su worktree; localiza
   el diff, la aceptación y los límites de reparación.
3. **Recuperación, 20–30 min:** reconstruye una ejecución interrumpida desde el
   ledger y los artefactos; propone el siguiente comando y sus precondiciones.

La transferencia termina cuando puedes conducir los tres casos, justificar
cada gate, preservar cambios ajenos y dejar un relevo que otra persona pueda
reanudar sin la conversación original.

## Mantenimiento

Actualiza esta guía solo cuando cambie una decisión humana o un ejemplo útil.
Las tablas de modelos, parámetros, bandas y fallback pertenecen a la guía
canónica. La evidencia histórica de esta entrega está en el
[ledger](../tasks/human-orchestrator-kt.md) y la
[auditoría](../audit/human-orchestrator-kt.md).
