#!/usr/bin/env bash
# daily-open.sh — genera la entrada del día en docs/daily/YYYY-MM-DD.md
# Uso: bash scripts/daily-open.sh [--force]
# --force sobreescribe si el archivo ya existe.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
TODAY="$(date +%Y-%m-%d)"
OUT="$REPO_ROOT/docs/daily/${TODAY}.md"
ROADMAP="$REPO_ROOT/docs/plan/roadmap.md"
TEMPLATE="$REPO_ROOT/docs/daily/TEMPLATE.md"

FORCE=0
for arg in "$@"; do [[ "$arg" == "--force" ]] && FORCE=1; done

if [[ -f "$OUT" && "$FORCE" -eq 0 ]]; then
  echo "Ya existe $OUT — usa --force para sobreescribir." >&2
  exit 1
fi

# ── 1. Header git ─────────────────────────────────────────────────────────────
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
SYNC="$(git status -sb 2>/dev/null | head -1 | grep -oE '\[.*\]' || echo 'synced')"
[[ -z "$SYNC" ]] && SYNC="synced"

# ── 2. Gates rápidos (inline; pesados se referencian) ─────────────────────────
gate_run() {
  local label="$1" cmd="$2"
  if $cmd &>/dev/null 2>&1; then echo "${label}:✅"; else echo "${label}:❌"; fi
}
FMT_GATE="$(gate_run  "fmt"  "make qa-fmt")"
DOCS_GATE="$(gate_run "docs" "make qa-docs")"
GATES="${FMT_GATE} ${DOCS_GATE} (lint/test/cov/build → manual)"

# ── 3. Roadmap pulse — fase activa ────────────────────────────────────────────
# Última fase en progreso (🟡) o la última done (✅) si no hay en progreso
ACTIVE_LINE="$(grep -E '^\| \*\*S-' "$ROADMAP" | grep '🟡' | tail -1)"
if [[ -z "$ACTIVE_LINE" ]]; then
  ACTIVE_LINE="$(grep -E '^\| \*\*S-' "$ROADMAP" | grep '✅ done' | tail -1)"
fi
ACTIVE_SID="$(echo "$ACTIVE_LINE" | grep -oE 'S-[0-9]+' | head -1)"
ACTIVE_TITLE="$(echo "$ACTIVE_LINE" | awk -F'|' '{print $3}' | sed 's/^ *//;s/ *$//'  | cut -c1-60)"
ACTIVE_PULSE="${ACTIVE_SID} — ${ACTIVE_TITLE}"

# ── 4. Drift-check ────────────────────────────────────────────────────────────
# Procesa cada fila de la tabla de fases; solo reporta ✅ done cerradas.
DRIFT_ROWS=""

while IFS= read -r line; do
  # extraer phase id (S-000, S-010, S-020/H1, …)
  sid="$(echo "$line" | grep -oE 'S-[0-9]+' | head -1)"
  [[ -z "$sid" ]] && continue

  # ¿la fila tiene cierre real? (✅ done)
  if echo "$line" | grep -qE '✅ done'; then
    # evidencia 1: algún archivo en docs/plan o docs/tasks menciona el phase id
    files_count=0
    uncommitted=0
    while IFS= read -r f; do
      [[ -z "$f" ]] && continue
      files_count=$((files_count + 1))
      rel="${f#$REPO_ROOT/}"
      if git -C "$REPO_ROOT" status --porcelain -- "$rel" 2>/dev/null \
          | grep -q .; then
        uncommitted=$((uncommitted + 1))
      fi
    done < <(grep -rl "$sid" "$REPO_ROOT/docs/plan/" \
        "$REPO_ROOT/docs/tasks/" 2>/dev/null)

    if [[ "$files_count" -eq 0 ]]; then
      DRIFT_ROWS="${DRIFT_ROWS}| $(date +%H:%M) | 🔴 | DRIFT | ${sid} done en roadmap sin plan/task | abierto | crear docs + commit |\n"
    elif [[ "$uncommitted" -gt 0 ]]; then
      DRIFT_ROWS="${DRIFT_ROWS}| $(date +%H:%M) | 🔴 | DRIFT | ${sid} done con ${uncommitted} archivo(s) sin commitear | abierto | git add + commit |\n"
    fi
  fi
  # 🟡 REPLANNED / ⬜ / cancelled / superseded → omitir (no exigen cierre)
done < <(grep -E '^\| \*\*S-' "$ROADMAP")

if [[ -z "$DRIFT_ROWS" ]]; then
  DRIFT_ROWS="| — | 🟢 | — | 0 filas DRIFT (roadmap ↔ git consistentes) | — | — |\n"
fi

# ── 5. Renderizar ─────────────────────────────────────────────────────────────
{
cat <<HEADER
# Daily — ${TODAY}

**Branch:** ${BRANCH} · **Sync:** ${SYNC} · **Gates:** ${GATES}
**Foco del día:** <completar>

---

## 1. Roadmap pulse

- **Fase activa:** ${ACTIVE_PULSE}
- **Desbloquea al cerrar:** <completar>
- **Gates de fundación en riesgo:** ninguno
- **X-items que se movieron:** —

---

## 2. Ayer → Hoy

| Estado | Task | Banda RRI | Nota |
|---|---|---|---|
| [~] sigue | <completar> | — | — |

---

## 3. Issues ledger

| Hora | Sev | Tipo | Descripción | Estado | Acción |
|---|---|---|---|---|---|
HEADER
printf "%b" "$DRIFT_ROWS"
cat <<FOOTER

---

## 4. Optimizaciones y mejoras

| ID | Tipo | Propuesta | Impacto | Esfuerzo | → Task? |
|---|---|---|---|---|---|
| O-01 | — | <completar> | — | — | — |

---

## 5. Decisiones pendientes (HITL gate)

- [ ] <completar>

---

## 6. Cierre del día ✓

- [ ] \`git status\` limpio — sin trabajo declarado "done" sin commitear
- [ ] Roadmap ↔ ledgers ↔ git consistentes (drift-check emite 0 🔴)
- [ ] Gates verdes: fmt, lint, test, check, deny, secrets, cov, docs — o BLOCKER abierto
- [ ] X-items tocados hoy reflejados en roadmap
- [ ] Daily de mañana sembrado con lo \`[~]\` que queda
FOOTER
} > "$OUT"

echo "Creado: $OUT"
echo ""
echo "Drift-check:"
printf "%b" "$DRIFT_ROWS" | column -t -s'|' 2>/dev/null || printf "%b" "$DRIFT_ROWS"
