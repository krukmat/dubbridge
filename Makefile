# Behavioral-testing wrapper. The pre-existing build/QA contract remains byte-for-byte
# in Makefile.base; this file only adds the cross-stack behavioral gates.
include Makefile.base

.PHONY: qa-behavioral-coverage qa-bdd-map

qa-behavioral-coverage:
	python3 scripts/check_behavioral_coverage_test.py
	python3 scripts/check-behavioral-coverage.py

qa-bdd-map:
	python3 scripts/check_bdd_map_test.py
	python3 scripts/check-bdd-map.py
