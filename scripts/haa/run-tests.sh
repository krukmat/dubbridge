#!/bin/sh
set -eu

python3 -c 'import cryptography' >/dev/null 2>&1 || {
  echo "HAA_TEST_DEPENDENCY_MISSING: install scripts/haa/requirements.txt in an isolated environment" >&2
  exit 2
}

exec python3 -m unittest \
  scripts.haa.contracts_test \
  scripts.haa.client_test \
  scripts.haa.authority_test \
  scripts.haa.grants_test \
  scripts.haa.adapter_test \
  scripts.haa.boundary_detector_test \
  scripts.haa.request_checkpoint_test \
  scripts.haa.live_verify_test \
  scripts.haa.authority_lease_test \
  scripts.haa.lease_guard_test \
  scripts.haa.recovery_test
