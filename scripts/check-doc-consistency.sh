#!/usr/bin/env bash
set -euo pipefail

# Preserve the historical deterministic documentation checks, then add the
# cross-stack behavioral and BDD traceability gates. qa-docs already invokes
# this entrypoint, so no second CI path is required.
bash scripts/check-doc-consistency-base.sh
python3 scripts/check_behavioral_coverage_test.py
python3 scripts/check-behavioral-coverage.py
python3 scripts/check_bdd_map_test.py
python3 scripts/check-bdd-map.py
