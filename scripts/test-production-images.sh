#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# Case registry - bash 3.2 compatible (no associative arrays)
CASE_LIST="self-check"

# Cleanup machinery
TEMP_DIR=""
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

usage() {
    cat >&2 <<'EOF'
Usage: scripts/test-production-images.sh <mode> <case>

Modes:
  contract - Validate image contract
  run - Run image test

Cases:
  self-check - Verify script preconditions (no Docker)
EOF
    exit 1
}

# Case membership check
case_exists() {
    local needle="$1"
    local c
    for c in $CASE_LIST; do
        if [[ "$c" == "$needle" ]]; then
            return 0
        fi
    done
    return 1
}

# self-check case functions
contract_self-check() {
    echo "Contract check for self-check case"
    # Verify required commands are on PATH
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "ERROR: jq not found on PATH" >&2
        return 1
    fi
    # Print bash version
    echo "Bash version: $BASH_VERSION"
    return 0
}

run_self-check() {
    echo "Run check for self-check case"
    # Verify required commands are on PATH
    if ! command -v docker >/dev/null 2>&1; then
        echo "ERROR: docker not found on PATH" >&2
        return 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "ERROR: jq not found on PATH" >&2
        return 1
    fi
    # Print bash version
    echo "Bash version: $BASH_VERSION"
    return 0
}

# Main execution
main() {
    # Validation 1: Exactly 2 positional arguments
    if [[ $# -ne 2 ]]; then
        usage
    fi

    local mode="$1"
    local case_name="$2"

    # Validation 2: Mode must be contract or run
    if [[ "$mode" != "contract" && "$mode" != "run" ]]; then
        usage
    fi

    # Validation 3: Case must exist in registry
    if ! case_exists "$case_name"; then
        usage
    fi

    # Dispatch to function
    local func_name="${mode}_${case_name}"
    if ! declare -F "$func_name" >/dev/null 2>&1; then
        echo "ERROR: Function $func_name not found" >&2
        exit 1
    fi

    "$func_name"
}

main "$@"
