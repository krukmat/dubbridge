#!/usr/bin/env bash
set -euo pipefail

repo_root=$(
  cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1
  pwd
)
cd "$repo_root"

roadmap="${ROADMAP_DRIFT_ROADMAP:-docs/plan/roadmap.md}"
plan_dir="${ROADMAP_DRIFT_PLAN_DIR:-docs/plan}"
tasks_dir="${ROADMAP_DRIFT_TASKS_DIR:-docs/tasks}"

violations=""

add_violation() {
  if [[ -n "$violations" ]]; then
    violations="${violations}"$'\n'"$1"
  else
    violations="$1"
  fi
}

trim() {
  sed 's/^[[:space:]]*//; s/[[:space:]]*$//'
}

if [[ ! -f "$roadmap" ]]; then
  printf 'Roadmap drift check failed: missing roadmap file %s\n' "$roadmap"
  exit 1
fi

while IFS= read -r line; do
  sid="$(printf '%s\n' "$line" | grep -oE 'S-[0-9]+' | head -1 || true)"
  [[ -n "$sid" ]] || continue

  status="$(printf '%s\n' "$line" | awk -F'|' '{print $5}' | trim)"

  if ! printf '%s\n' "$status" | grep -q '✅ done'; then
    continue
  fi

  files_count=0
  uncommitted=0
  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    [[ "$file" == "$roadmap" ]] && continue
    files_count=$((files_count + 1))
    if git status --porcelain -- "$file" 2>/dev/null | grep -q .; then
      uncommitted=$((uncommitted + 1))
    fi
  done < <(grep -rl "$sid" "$plan_dir" "$tasks_dir" 2>/dev/null || true)

  if [[ "$files_count" -eq 0 ]]; then
    add_violation "$sid: marked ✅ done in $roadmap without SID evidence in $plan_dir or $tasks_dir"
  elif [[ "$uncommitted" -gt 0 ]]; then
    add_violation "$sid: marked ✅ done with $uncommitted uncommitted plan/task evidence file(s)"
  fi
done < <(grep -E '^\|[[:space:]]*\*\*S-' "$roadmap" || true)

if [[ -n "$violations" ]]; then
  printf 'Roadmap drift check failed:\n'
  old_ifs="$IFS"
  IFS=$'\n'
  for violation in $violations; do
    [[ -n "$violation" ]] || continue
    printf ' - %s\n' "$violation"
  done
  IFS="$old_ifs"
  exit 1
fi

printf 'Roadmap drift check passed.\n'
