#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob

repo_root=$(
  cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1
  pwd
)
cd "$repo_root"

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

case_ids_for_prefix() {
  local section="$1"
  local prefix="$2"
  printf '%s\n' "$section" \
    | grep -E "^[[:space:]]*-[[:space:]]*(\\*\\*)?${prefix}-[0-9]+\\b" \
    | grep -oE "${prefix}-[0-9]+" \
    | sort -u || true
}

is_completed_development_section() {
  local section="$1"
  printf '%s\n' "$section" | grep -Eq 'Status:.*\[[xX]\].*([Dd]one|DONE)' \
    && printf '%s\n' "$section" | grep -Eq 'Type:.*Development'
}

find_certification_row() {
  local section="$1"
  local case_id="$2"
  printf '%s\n' "$section" | grep -E "^\\|[[:space:]]*(\\*\\*)?${case_id}(\\*\\*)?[[:space:]]*\\|" || true
}

validate_test_ref() {
  local task_file="$1"
  local section_title="$2"
  local case_id="$3"
  local ref="$4"

  local path
  local test_name
  path="$(printf '%s\n' "$ref" | sed -n 's/^\(.*\.rs\)::.*$/\1/p')"
  test_name="${ref##*::}"

  if [[ -z "$path" || -z "$test_name" ]]; then
    add_violation "$task_file: $section_title: $case_id unit test evidence '$ref' must use path/to/file.rs::test_name"
    return
  fi

  if [[ ! -f "$path" ]]; then
    add_violation "$task_file: $section_title: $case_id unit test evidence references missing file '$path'"
    return
  fi

  if ! grep -Eq "(^|[[:space:]])(async[[:space:]]+)?fn[[:space:]]+${test_name}\\b" "$path"; then
    add_violation "$task_file: $section_title: $case_id unit test evidence references missing test function '$test_name' in '$path'"
  fi
}

validate_case_certification() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"
  local case_id="$4"

  local row
  row="$(find_certification_row "$section" "$case_id")"

  if [[ -z "$row" ]]; then
    add_violation "$task_file: $section_title: missing Unit coverage certification row for $case_id"
    return
  fi

  local evidence
  local result
  evidence="$(printf '%s\n' "$row" | awk -F'|' '{print $5}' | trim)"
  result="$(printf '%s\n' "$row" | awk -F'|' '{print $6}' | trim)"

  if [[ -z "$evidence" || "$evidence" =~ ^[Nn]/[Aa] ]]; then
    add_violation "$task_file: $section_title: $case_id has missing or N/A unit test evidence"
    return
  fi

  if [[ "$result" != "passed" ]]; then
    add_violation "$task_file: $section_title: $case_id result must be 'passed'"
  fi

  local refs
  refs="$(printf '%s\n' "$evidence" | grep -oE '`[^`]+\.rs::[A-Za-z_][A-Za-z0-9_:]*`' | tr -d '`' || true)"
  if [[ -z "$refs" ]]; then
    add_violation "$task_file: $section_title: $case_id unit test evidence must include at least one backticked path/to/file.rs::test_name reference"
    return
  fi

  local ref
  while IFS= read -r ref; do
    [[ -n "$ref" ]] || continue
    validate_test_ref "$task_file" "$section_title" "$case_id" "$ref"
  done <<EOF
$refs
EOF
}

validate_owner_verification() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"

  if ! printf '%s\n' "$section" | grep -q 'Owner final verification'; then
    add_violation "$task_file: $section_title: missing Owner final verification section"
    return
  fi

  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Owner:[[:space:]]*[^[:space:]].*'; then
    add_violation "$task_file: $section_title: Owner final verification missing non-empty Owner"
  fi
  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Date:[[:space:]]*[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]]*$'; then
    add_violation "$task_file: $section_title: Owner final verification Date must use YYYY-MM-DD"
  fi
  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Statement:[[:space:]]*.*verified every happy path and edge case.*unit test evidence.*'; then
    add_violation "$task_file: $section_title: Owner final verification Statement must certify every happy path and edge case has unit test evidence"
  fi
  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Commands run:[[:space:]]*[^[:space:]].*'; then
    add_violation "$task_file: $section_title: Owner final verification missing exact Commands run"
  fi
}

validate_section() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"

  if ! is_completed_development_section "$section"; then
    return
  fi

  if ! printf '%s\n' "$section" | grep -q 'Unit coverage certification'; then
    add_violation "$task_file: $section_title: missing Unit coverage certification section"
  fi
  if ! printf '%s\n' "$section" | grep -q 'Happy paths considered'; then
    add_violation "$task_file: $section_title: missing Happy paths considered section"
  fi
  if ! printf '%s\n' "$section" | grep -q 'Edge cases considered'; then
    add_violation "$task_file: $section_title: missing Edge cases considered section"
  fi

  local hp_ids
  local ec_ids
  hp_ids="$(case_ids_for_prefix "$section" "HP")"
  ec_ids="$(case_ids_for_prefix "$section" "EC")"

  if [[ -z "$hp_ids" ]]; then
    add_violation "$task_file: $section_title: Happy paths considered must define at least one stable HP-# case ID"
  fi
  if [[ -z "$ec_ids" ]]; then
    add_violation "$task_file: $section_title: Edge cases considered must define at least one stable EC-# case ID"
  fi

  local case_id
  while IFS= read -r case_id; do
    [[ -n "$case_id" ]] || continue
    validate_case_certification "$task_file" "$section_title" "$section" "$case_id"
  done <<EOF
$hp_ids
$ec_ids
EOF

  validate_owner_verification "$task_file" "$section_title" "$section"
}

validate_task_file() {
  local task_file="$1"

  if ! grep -q 'Behavioral coverage contract: unit-v1' "$task_file"; then
    return
  fi

  local total_lines
  total_lines="$(wc -l < "$task_file" | tr -d ' ')"

  local headings
  headings=()
  local heading
  while IFS= read -r heading; do
    [[ -n "$heading" ]] || continue
    headings+=("$heading")
  done < <(grep -nE '^##[[:space:]]' "$task_file" | cut -d: -f1 || true)
  if [[ ${#headings[@]} -eq 0 ]]; then
    add_violation "$task_file: unit-v1 ledger has no task sections"
    return
  fi

  local i
  for ((i = 0; i < ${#headings[@]}; i++)); do
    local start
    local end
    local section
    local section_title
    start="${headings[$i]}"
    if (( i + 1 < ${#headings[@]} )); then
      end=$((headings[$i + 1] - 1))
    else
      end="$total_lines"
    fi

    section="$(sed -n "${start},${end}p" "$task_file")"
    section_title="$(printf '%s\n' "$section" | sed -n '1s/^##[[:space:]]*//p')"
    validate_section "$task_file" "$section_title" "$section"
  done
}

if [[ "$#" -gt 0 ]]; then
  task_files=("$@")
else
  task_files=(docs/tasks/*.md)
fi

for task_file in "${task_files[@]}"; do
  [[ -f "$task_file" ]] || continue
  validate_task_file "$task_file"
done

if [[ -n "$violations" ]]; then
  printf 'Task unit coverage certification check failed:\n'
  old_ifs="$IFS"
  IFS=$'\n'
  for violation in $violations; do
    [[ -n "$violation" ]] || continue
    printf ' - %s\n' "$violation"
  done
  IFS="$old_ifs"
  exit 1
fi

printf 'Task unit coverage certification check passed.\n'
