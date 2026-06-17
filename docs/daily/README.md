# Daily Planning — Convención

Un daily plan es un artefacto fechado que conecta el estado del repo con el
trabajo del día. Vive en `docs/daily/YYYY-MM-DD.md`.

## Ritmo

| Momento | Acción | Quién |
|---|---|---|
| Apertura | `bash scripts/daily-open.sh` → genera el día | orquestador |
| Apertura | Completar §1 foco + §5 decisiones pendientes | humano |
| Cierre | Completar §3 issues + §4 mejoras + §6 reconciliación | orquestador/humano |
| Cierre | Commitear la entrada del día | orquestador |

El script rellena lo mecánico (git, roadmap pulse, drift-check, semáforo de
gates). El juicio lo aporta el humano o el orquestador, no el script.

## Drift-check

Cruza cada fase `✅ done` del roadmap contra evidencia en git y archivos. Emite:

- 🔴 **DRIFT** — fase marcada done sin plan/task ni commit asociado (bloqueante)
- 🟡 **AVISO** — fase done con archivos pero sin commit confirmado (revisar)

Fases `🟡 REPLANNED`, `⬜`, `cancelled`, `superseded` se omiten: no exigen cierre.

## Taxonomía de issues (§3)

| Tipo | Cuándo usarlo |
|---|---|
| **BLOCKER** | Rompe un gate o bloquea la fase activa |
| **DRIFT** | Docs, roadmap o ledger desincronizados del estado real |
| **DEBT** | Atajo técnico que hay que revisar; no bloquea ahora |
| **RISK** | Latente — podría volverse BLOCKER si se deja crecer |

## Severidad

🔴 = crítico/activo · 🟡 = moderado/vigilar · 🟢 = resuelto ese día

## Mapeo a RRI

Los issues de §3 y las mejoras de §4 pueden escalar a tasks:

- **O-xx** con esfuerzo S y dominio bajo → candidato Low → Gemma local
- **D-xx** BLOCKER → abre task con RRI explícito antes de la siguiente sesión

## Límite

El daily plan es un control **detectivo** (caza lo que ya pasó). El control
**preventivo** (que nada pase sin plan aprobado) es responsabilidad de CI y
branch protection — no de este documento.
