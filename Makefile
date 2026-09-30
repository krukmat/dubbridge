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


.PHONY: do-state-bootstrap

do-state-bootstrap:
	bash scripts/do-bootstrap-state.sh


.PHONY: do-inventory-freeze

do-inventory-freeze:
	bash scripts/do-freeze-inventory.sh


.PHONY: do-adoption-matrix

do-adoption-matrix:
	python3 scripts/do-classify-adoption.py


.PHONY: do-preserve-wordpress

do-preserve-wordpress:
	bash scripts/do-preserve-wordpress.sh


.PHONY: do-reuse-resize do-import-reused-base

do-reuse-resize:
	bash scripts/do-reuse-resize.sh

do-import-reused-base:
	bash scripts/do-import-reused-base.sh


.PHONY: infra-plan infra-state-backup

infra-plan:
	bash scripts/infra-plan.sh

infra-state-backup:
	bash scripts/infra-state-backup.sh


.PHONY: r2-state-bootstrap contabo-c2-preflight

r2-state-bootstrap:
	bash scripts/r2-state-bootstrap.sh

contabo-c2-preflight:
	bash scripts/contabo-c2-preflight.sh
