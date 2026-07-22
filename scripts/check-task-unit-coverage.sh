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
    && printf '%s\n' "$section" | grep -Eiq 'Type:.*development'
}

section_rri_value() {
  local section="$1"
  printf '%s\n' "$section" | sed -n 's/.*RRI:\*\{0,2\}[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -n1
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

validate_gemma_reviewer_evidence() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"

  if ! printf '%s\n' "$section" | grep -q 'Gemma Reviewer evidence'; then
    add_violation "$task_file: $section_title: missing Gemma Reviewer evidence section"
    return
  fi

  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Command:[[:space:]]*[^[:space:]].*'; then
    add_violation "$task_file: $section_title: Gemma Reviewer evidence missing exact Command"
  fi
  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Quorum:[[:space:]]*(met|failed)[[:space:]]*$'; then
    add_violation "$task_file: $section_title: Gemma Reviewer evidence must record Quorum as met or failed"
  fi
  if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-[[:space:]]*Primary-agent disposition:[[:space:]]*[^[:space:]].*'; then
    add_violation "$task_file: $section_title: Gemma Reviewer evidence missing Primary-agent disposition"
  fi
}

# GEG-1e cutover: sections closed [x] Done before this date are grandfathered
# out of validate_review_evidence and keep the legacy RRI<=40-only check
# above instead. See docs/tasks/gemma-evidence-artifact-gate.md (GEG-1e AC 1).
REVIEW_EVIDENCE_CUTOVER_DATE="2026-07-22"

section_done_date() {
  local section="$1"
  printf '%s\n' "$section" \
    | sed -n 's/.*-[[:space:]]*Date:[[:space:]]*\([0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}\).*/\1/p' \
    | head -n1
}

section_predates_cutover() {
  local section="$1"
  local done_date
  done_date="$(section_done_date "$section")"
  [[ -n "$done_date" ]] && [[ "$done_date" < "$REVIEW_EVIDENCE_CUTOVER_DATE" ]]
}

# Band-agnostic artifact-or-override gate (GEG-1b/1c). Applies to every
# completed development section at or after the cutover, regardless of RRI.
# A `Review artifact:` line must resolve to a real, matching, reachable
# receipt (docs/audit/gemma-evidence/<task_id>.json); a `REVIEW-OVERRIDE:`
# line must be one of the three typed exceptions with its companion field and
# a matching row in docs/audit/gemma-review-overrides.md. Absence of both
# fails the gate — silence is not a pass.
OVERRIDES_LEDGER="docs/audit/gemma-review-overrides.md"

extract_task_id() {
  local section_title="$1"
  printf '%s\n' "$section_title" | awk '{print $1}' | sed 's/:$//'
}

validate_review_artifact_line() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"
  local artifact_task_id="$4"

  local receipt_path="docs/audit/gemma-evidence/${artifact_task_id}.json"
  if [[ ! -f "$receipt_path" ]]; then
    add_violation "$task_file: $section_title: Review artifact points at missing receipt '$receipt_path'"
    return
  fi

  if ! python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$receipt_path" >/dev/null 2>&1; then
    add_violation "$task_file: $section_title: Review artifact '$receipt_path' is not valid JSON"
    return
  fi

  local receipt_task_id
  local receipt_commit_sha
  receipt_task_id="$(python3 -c "import json; print(json.load(open('$receipt_path')).get('task_id',''))" 2>/dev/null || true)"
  receipt_commit_sha="$(python3 -c "import json; print(json.load(open('$receipt_path')).get('commit_sha',''))" 2>/dev/null || true)"

  if [[ "$receipt_task_id" != "$artifact_task_id" ]]; then
    add_violation "$task_file: $section_title: Review artifact task_id '$receipt_task_id' does not match section '$artifact_task_id'"
  fi

  if [[ -z "$receipt_commit_sha" ]]; then
    add_violation "$task_file: $section_title: Review artifact missing commit_sha"
  elif ! git merge-base --is-ancestor "$receipt_commit_sha" HEAD 2>/dev/null && [[ "$receipt_commit_sha" != "$(git rev-parse HEAD 2>/dev/null)" ]]; then
    add_violation "$task_file: $section_title: Review artifact commit_sha '$receipt_commit_sha' is not reachable from reviewed history"
  fi
}

validate_review_override_line() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"
  local override_line="$4"

  local override_type
  override_type="$(printf '%s\n' "$override_line" | sed -nE 's/.*REVIEW-OVERRIDE:[[:space:]]*([A-Za-z-]+)[[:space:]]*—.*/\1/p')"

  case "$override_type" in
    urgency)
      if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-?[[:space:]]*Waiver-by:[[:space:]]*[^[:space:]].*'; then
        add_violation "$task_file: $section_title: REVIEW-OVERRIDE: urgency missing companion Waiver-by: <human name>"
      fi
      ;;
    pipeline-failure)
      if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-?[[:space:]]*Failed-attempt:[[:space:]]*[^[:space:]].*'; then
        add_violation "$task_file: $section_title: REVIEW-OVERRIDE: pipeline-failure missing companion Failed-attempt: <evidence>"
      fi
      ;;
    not-applicable)
      if ! printf '%s\n' "$section" | grep -Eq '^[[:space:]]*-?[[:space:]]*Scope-note:[[:space:]]*[^[:space:]].*'; then
        add_violation "$task_file: $section_title: REVIEW-OVERRIDE: not-applicable missing companion Scope-note: <why>"
      fi
      ;;
    *)
      add_violation "$task_file: $section_title: REVIEW-OVERRIDE type '$override_type' is not one of urgency, not-applicable, pipeline-failure"
      return
      ;;
  esac

  local task_id
  task_id="$(extract_task_id "$section_title")"
  if [[ -z "$task_id" ]] || [[ ! -f "$OVERRIDES_LEDGER" ]] || ! grep -qF "$task_id" "$OVERRIDES_LEDGER"; then
    add_violation "$task_file: $section_title: REVIEW-OVERRIDE has no matching row in $OVERRIDES_LEDGER"
  fi
}

validate_review_evidence() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"

  local artifact_line
  artifact_line="$(printf '%s\n' "$section" | grep -E '^[[:space:]]*-?[[:space:]]*Review artifact:' | head -n1 || true)"
  local override_line
  override_line="$(printf '%s\n' "$section" | grep -E '^[[:space:]]*-?[[:space:]]*REVIEW-OVERRIDE:' | head -n1 || true)"

  if [[ -n "$artifact_line" ]]; then
    local artifact_task_id
    artifact_task_id="$(extract_task_id "$section_title")"
    if [[ -z "$artifact_task_id" ]]; then
      add_violation "$task_file: $section_title: cannot derive task_id from section title for Review artifact check"
      return
    fi
    validate_review_artifact_line "$task_file" "$section_title" "$section" "$artifact_task_id"
    return
  fi

  if [[ -n "$override_line" ]]; then
    validate_review_override_line "$task_file" "$section_title" "$section" "$override_line"
    return
  fi

  add_violation "$task_file: $section_title: missing Review artifact or REVIEW-OVERRIDE evidence"
}

validate_reflection_log() {
  local task_file="$1"
  local section_title="$2"
  local section="$3"

  if ! printf '%s\n' "$section" | grep -q 'Reflection log'; then
    add_violation "$task_file: $section_title: missing Reflection log section"
    return
  fi

  if ! printf '%s\n' "$section" | grep -Eq 'Required passes:[[:space:]]*[0-9]+'; then
    add_violation "$task_file: $section_title: Reflection log missing Required passes header"
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

  local rri
  rri="$(section_rri_value "$section")"

  if [[ -z "$rri" ]]; then
    add_violation "$task_file: $section_title: completed development section must declare numeric RRI"
  fi

  if section_predates_cutover "$section"; then
    if [[ -n "$rri" ]] && (( rri <= 40 )); then
      validate_gemma_reviewer_evidence "$task_file" "$section_title" "$section"
    fi
  else
    validate_review_evidence "$task_file" "$section_title" "$section"
  fi

  if [[ -n "$rri" ]] && (( rri >= 26 )); then
    validate_reflection_log "$task_file" "$section_title" "$section"
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
  printf 'Task completion evidence check failed:\n'
  old_ifs="$IFS"
  IFS=$'\n'
  for violation in $violations; do
    [[ -n "$violation" ]] || continue
    printf ' - %s\n' "$violation"
  done
  IFS="$old_ifs"
  exit 1
fi

printf 'Task completion evidence check passed.\n'
