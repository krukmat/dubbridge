---
type: Playbook
title: Manual de reemplazo del orquestador humano
status: superseded
---

# Manual de reemplazo del orquestador humano

Este nombre se conserva para no romper enlaces antiguos. Su contenido se
consolidó el 2026-09-19 en la
[Guía del orquestador humano](HUMAN_ORCHESTRATOR_KT.md), que contiene el inicio
de sesión, el ciclo completo, comandos, recuperación, relevo y ejercicios.

Usa estas fuentes en este orden:

1. [Agent Workflow Guide](AGENT_WORKFLOW_GUIDE.md), autoridad del proceso.
2. [Guía del orquestador humano](HUMAN_ORCHESTRATOR_KT.md), operación diaria.
3. El plan y ledger de la tarea real, para alcance y estado.

Las instantáneas de rama, CI y tareas que antes vivían aquí eran históricas y
podían inducir decisiones obsoletas. Obtén el estado actual con:

```bash
python3 scripts/human-orchestrator.py status
python3 scripts/human-orchestrator.py --execute run preflight -- --print-summary
```
