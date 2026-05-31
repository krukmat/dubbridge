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

status_token() {
  case "$1" in
    Proposed*) printf 'Proposed\n' ;;
    Accepted*) printf 'Accepted\n' ;;
    Superseded*) printf 'Superseded\n' ;;
    Deprecated*) printf 'Deprecated\n' ;;
    *) printf '\n' ;;
  esac
}

adr_exists() {
  local adr_id="$1"
  local matches
  matches=(docs/adr/"${adr_id}"-*.md)
  [[ ${#matches[@]} -gt 0 ]]
}

file_status_token() {
  local file="$1"
  local status_line
  status_line="$(grep -m1 '^- \*\*Status:\*\*' "$file" || true)"
  if [[ -z "$status_line" ]]; then
    printf '\n'
    return
  fi
  status_token "${status_line#- **Status:** }"
}

index_row_data() {
  local adr_id="$1"
  local line
  line="$(grep -F "| [$adr_id](" docs/adr/README.md || true)"
  if [[ -z "$line" ]]; then
    printf '\n'
    return
  fi

  local target
  local status
  target="$(printf '%s\n' "$line" | sed -n 's/^| \[[^]]*\](\([^)]*\)) | .* | .*$/\1/p')"
  status="$(printf '%s\n' "$line" | awk -F'|' '{print $4}' | sed 's/^ *//; s/ *$//')"
  printf '%s|%s\n' "$target" "$status"
}

check_status_parity_and_completeness() {
  local file
  for file in docs/adr/ADR-*.md; do
    [[ -f "$file" ]] || continue

    local adr_name
    local adr_id
    local file_status
    local row_data
    local row_target
    local row_status
    local successor

    adr_name="${file##*/}"
    adr_id="$(printf '%s\n' "$adr_name" | sed -n 's/^\(ADR-[0-9][0-9][0-9]\).*$/\1/p')"
    file_status="$(file_status_token "$file")"
    successor="$(grep -m1 -oE 'Superseded by ADR-[0-9]{3}' "$file" | sed 's/^Superseded by //' || true)"

    if [[ -z "$file_status" ]]; then
      add_violation "$file: missing or unparseable - **Status:** line"
      continue
    fi

    if [[ -n "$successor" ]] && ! adr_exists "$successor"; then
      add_violation "$file: superseded successor $successor does not exist"
    fi

    row_data="$(index_row_data "$adr_id")"
    if [[ -z "$row_data" ]]; then
      add_violation "$file: missing index row in docs/adr/README.md"
      continue
    fi

    row_target="${row_data%%|*}"
    row_status="${row_data#*|}"
    row_status="$(status_token "$row_status")"

    if [[ -z "$row_status" ]]; then
      add_violation "docs/adr/README.md: could not parse status token for $adr_id"
      continue
    fi

    if [[ "$file_status" != "$row_status" ]]; then
      add_violation "$file: status '$file_status' does not match index status '$row_status' for $adr_id"
    fi

    if [[ ! -f "docs/adr/$row_target" ]]; then
      add_violation "docs/adr/README.md: index row for $adr_id points to missing file '$row_target'"
    fi
  done

  local index_lines
  index_lines="$(grep -E '^\| \[ADR-[0-9]{3}\]\([^)]+\) \|' docs/adr/README.md || true)"
  local old_ifs
  old_ifs="$IFS"
  IFS=$'\n'
  local line
  for line in $index_lines; do
    [[ -n "$line" ]] || continue
    local adr_id
    local target
    adr_id="$(printf '%s\n' "$line" | sed -n 's/^| \[\(ADR-[0-9][0-9][0-9]\)\].*$/\1/p')"
    target="$(printf '%s\n' "$line" | sed -n 's/^| \[[^]]*\](\([^)]*\)) | .*$/\1/p')"

    if ! adr_exists "$adr_id"; then
      add_violation "docs/adr/README.md: index row for $adr_id points to missing ADR file '$target'"
    fi
  done
  IFS="$old_ifs"
}

check_stream_for_dangling_refs() {
  local scope="$1"
  local stream="$2"
  local old_ifs
  old_ifs="$IFS"
  IFS=$'\n'
  local record
  for record in $stream; do
    [[ -n "$record" ]] || continue
    local file
    local line
    local content
    local tokens
    local token
    file="$(printf '%s\n' "$record" | sed -n 's/^\(.*\):[0-9][0-9]*:.*$/\1/p')"
    line="$(printf '%s\n' "$record" | sed -n 's/^.*:\([0-9][0-9]*\):.*$/\1/p')"
    content="${record#"$file:$line:"}"
    tokens="$(printf '%s\n' "$content" | grep -oE 'ADR-0[0-9][0-9]' || true)"

    while IFS= read -r token; do
      [[ -n "$token" ]] || continue
      if printf '%s\n' "$content" | grep -q "${token}\.\."; then
        continue
      fi
      if ! adr_exists "$token"; then
        add_violation "$scope: dangling reference $token in $file:$line"
      fi
    done <<EOF
$tokens
EOF
  done
  IFS="$old_ifs"
}

check_dangling_refs() {
  local docs_stream
  local code_stream

  docs_stream="$(
    grep -R -nH 'ADR-0[0-9][0-9]' docs/adr docs/plan docs/tasks 2>/dev/null || true
    grep -nH 'ADR-0[0-9][0-9]' docs/architecture.md README.md 2>/dev/null || true
  )"
  code_stream="$(
    grep -R -nH --include='*.rs' 'ADR-0[0-9][0-9]' crates apps 2>/dev/null || true
    grep -R -nH --include='*.sql' 'ADR-0[0-9][0-9]' infra/migrations 2>/dev/null || true
  )"

  check_stream_for_dangling_refs "docs" "$docs_stream"
  check_stream_for_dangling_refs "code" "$code_stream"
}

check_superseded_successors() {
  local lines
  lines="$(grep -nH 'Superseded by ADR-[0-9][0-9][0-9]' docs/adr/ADR-*.md || true)"
  local old_ifs
  old_ifs="$IFS"
  IFS=$'\n'
  local record
  for record in $lines; do
    [[ -n "$record" ]] || continue
    local file
    local line
    local content
    local tokens
    local token
    file="$(printf '%s\n' "$record" | sed -n 's/^\(.*\):[0-9][0-9]*:.*$/\1/p')"
    line="$(printf '%s\n' "$record" | sed -n 's/^.*:\([0-9][0-9]*\):.*$/\1/p')"
    content="${record#"$file:$line:"}"
    tokens="$(printf '%s\n' "$content" | grep -oE 'ADR-[0-9]{3}' || true)"

    while IFS= read -r token; do
      [[ -n "$token" ]] || continue
      if ! adr_exists "$token"; then
        add_violation "$file:$line: superseded successor $token does not exist"
      fi
    done <<EOF
$tokens
EOF
  done
  IFS="$old_ifs"
}

check_status_parity_and_completeness
check_dangling_refs
check_superseded_successors

if [[ -n "$violations" ]]; then
  printf 'Documentation consistency check failed:\n'
  old_ifs="$IFS"
  IFS=$'\n'
  for violation in $violations; do
    [[ -n "$violation" ]] || continue
    printf ' - %s\n' "$violation"
  done
  IFS="$old_ifs"
  exit 1
fi

printf 'Documentation consistency check passed.\n'
