#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob

if (($# > 0)); then
  files=("$@")
else
  files=(config/*.toml)
fi

if ((${#files[@]} == 0)); then
  echo "No config TOML files found to inspect." >&2
  exit 1
fi

has_violations=0

is_secret_like_segment() {
  local segment="$1"
  local normalized
  normalized="$(printf '%s' "$segment" | tr '[:upper:]' '[:lower:]')"

  case "$normalized" in
    password|secret|token|key)
      return 0
      ;;
    *_password|*_secret|*_token|*_key)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

for file in "${files[@]}"; do
  [[ -f "$file" ]] || continue

  line_no=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_no=$((line_no + 1))

    trimmed="${line#"${line%%[![:space:]]*}"}"
    [[ -z "$trimmed" || "$trimmed" == \#* || "$trimmed" == \[* ]] && continue

    if [[ "$trimmed" =~ ^([A-Za-z0-9_.-]+)[[:space:]]*= ]]; then
      key="${BASH_REMATCH[1]}"
      IFS='.' read -r -a path_segments <<< "$key"
      for segment in "${path_segments[@]}"; do
        if is_secret_like_segment "$segment"; then
          printf 'Secret-looking config key found: %s:%d: %s\n' "$file" "$line_no" "$key" >&2
          has_violations=1
          break
        fi
      done
    fi
  done < "$file"
done

if ((has_violations)); then
  echo "Committed config profiles must stay non-secret. Move secrets to injected env vars." >&2
  exit 1
fi

echo "Config profiles contain no secret-looking keys."
