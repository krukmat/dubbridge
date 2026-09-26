#!/usr/bin/env bash
# Regression for preflight log matching under `set -o pipefail`.
# The matcher must consume the full stream so an early match cannot SIGPIPE
# upstream producers, while genuine producer failures still propagate.
set -euo pipefail

needle="p2p_publication_enabled=true"

strip_ansi() {
  sed -E $'s/\033\\[[0-9;]*m//g'
}

match_early_then_continue() {
  printf '\033[32m%s\033[0m\n' "${needle}"
  for ((i = 0; i < 20000; i++)); do
    printf 'tail-%05d lorem ipsum dolor sit amet\n' "${i}"
  done
}

no_match() {
  for ((i = 0; i < 100; i++)); do
    printf 'tail-%05d\n' "${i}"
  done
}

match_then_fail() {
  printf '%s\n' "${needle}"
  printf 'tail\n'
  return 42
}

fail() {
  echo "FAIL - $*" >&2
  exit 1
}

if match_early_then_continue | strip_ansi | grep -F "${needle}" >/dev/null; then
  echo "OK   - early match with trailing output stays successful under pipefail"
else
  fail "early match incorrectly failed under pipefail"
fi

if no_match | strip_ansi | grep -F "${needle}" >/dev/null; then
  fail "missing match incorrectly succeeded"
else
  echo "OK   - missing match remains a failure"
fi

if match_then_fail | strip_ansi | grep -F "${needle}" >/dev/null; then
  fail "producer failure after a match was hidden"
else
  echo "OK   - producer failure after a match still propagates"
fi

echo "PREFLIGHT PIPELINE REGRESSION: PASS"
