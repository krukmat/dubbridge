# Daily — YYYY-MM-DD

**Branch:** main · **Sync:** ⇡N ⇣M vs origin/main · **Gates:** `fmt:❓ docs:❓`
**Foco del día:** <una línea: el resultado concreto que importa hoy>

---

## 1. Roadmap pulse

- **Fase activa:** S-1XX — \<título\> · \<n\>/\<total\> tasks done
- **Desbloquea al cerrar:** \<fase o task siguiente\>
- **Gates de fundación en riesgo:** ninguno | \<Sxxx / Hx\>
- **X-items que se movieron:** — | \<X## cerrado/abierto\>

---

## 2. Pipelines GH rotos

| Workflow | Último fallo | Estado | Acción |
|---|---|---|---|
| `ci.yml` | ninguno \| \<run / job roto\> | limpio \| abierto \| mitigado | vigilar \| \<task / owner\> |

> Registrar aquí cualquier GitHub Actions en rojo que siga roto al abrir o cerrar
> el día. Si no hay ninguno, dejar `ninguno` explícito.

---

## 3. Push-review post-pipeline

| Run / SHA | Conclusión pipeline | Estado push-review | RRI / routing | Acción |
|---|---|---|---|---|
| `<run-id o sha corto>` | success \| failure \| cancelled \| timed_out | PASS \| FINDINGS \| BLOCKED \| ninguno | pure Low dispatched \| non-Gemma review \| HITL \| n/a | `<reporte / task / owner>` |

> Revisar el report más reciente en `docs/reports/push-review/` o el artifact
> local correspondiente. Registrar findings no-pure-Low, patches delegados que
> siguen `in_review`, y cualquier `BLOCKED` que requiera seguimiento.

---

## 4. Ayer → Hoy

| Estado | Task | Banda RRI | Nota |
|---|---|---|---|
| [x] cerró ayer | S-XXX-TN | Low \| Moderate \| … | \<resumen en 5 palabras\> |
| [~] sigue hoy | S-XXX-TN | … | \<dónde quedó\> |
| [ ] nuevo hoy | S-XXX-TN | … | requiere aprobación \| Gemma local |

> **Regla de banda:** Low (0–25) → Gemma local (sin aprobación explícita).
> Moderate+ (26+) → presentar task + esperar aprobación antes de implementar.

---

## 5. Issues ledger

| ID | Sev | Tipo | Descripción | Estado | Acción |
|---|---|---|---|---|---|
| D-01 | 🔴 | BLOCKER | \<qué rompe\> | abierto | \<task / owner\> |
| D-02 | 🟡 | DRIFT | \<docs ↔ realidad\> | resuelto | \<qué se hizo\> |
| D-03 | 🟡 | DEBT | \<atajo técnico\> | abierto | \<candidato O-xx\> |
| D-04 | 🟢 | RISK | \<latente\> | mitigado | \<evidencia\> |

---

## 6. Optimizaciones y mejoras

| ID | Tipo | Propuesta | Impacto | Esfuerzo | → Task? |
|---|---|---|---|---|---|
| O-01 | CI/DX | \<qué\> | Alto \| Medio \| Bajo | S \| M \| L | candidato Low \| Moderate |
| O-02 | test-infra \| perf \| arch \| docs | \<qué\> | … | … | … |

> Las entradas O-xx son ideas, no compromisos. Solo se vuelven tasks cuando
> se computa RRI y se presenta al humano.

---

## 7. Decisiones pendientes (HITL gate)

- [ ] \<decisión que espera al humano — p.ej. aprobar plan S-120, elegir ADR para X20\>

---

## 8. Cierre del día ✓

- [ ] `git status` limpio — sin trabajo declarado "done" sin commitear
- [ ] Roadmap ↔ ledgers ↔ git consistentes (drift-check emite 0 🔴)
- [ ] Pipelines GH rotos revisados; si existe alguno, quedó con owner o task
- [ ] Push-review más reciente revisado; findings no-pure-Low y patches `in_review` registrados o referenciados
- [ ] 8 gates verdes (`qa-fmt`, `qa-lint`, `qa-test`, `qa-check`, `qa-deny`, `qa-config-secrets`, `qa-coverage`, `qa-docs`) — o issue BLOCKER explicando cuál y por qué
- [ ] X-items tocados hoy reflejados en roadmap
- [ ] Daily de mañana sembrado con lo `[~]` que queda
