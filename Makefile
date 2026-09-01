.PHONY: qa-fmt qa-lint qa-test qa-test-redis qa-test-s3 qa-check qa-local qa-deny qa-config-secrets qa-roadmap-drift qa-coverage qa-build-release qa-maintainability qa-python-complexity qa-review-budget qa-mobile qa-design qa-task-unit-coverage qa-docs qa-docs-review qa-rri qa-ci qa-gemma-review qa-gemma-push-review qa-peer-workflow-review qa-golden-set show-codex-session-model install-hooks

COVERAGE_MIN ?= 90
PEER_REVIEW_RRI      ?= 22
PEER_REVIEW_PHASE    ?= code
PEER_REVIEW_CALLER   ?= claude-code
PEER_REVIEW_TASK_ID  ?=
PEER_REVIEW_ARTIFACT ?= /tmp/dubbridge-peer-review.json
PEER_REVIEW_BASE     ?= HEAD
GEMMA_REVIEW_BASE   ?= HEAD
GEMMA_REVIEW_TASK_ID ?=
# Task-scoped by default so a stale result from a different task id can never
# be mistaken for the current review (GEG-2a / defect D1). Explicit override
# still wins because command-line assignments outrank Makefile '?=' defaults.
GEMMA_REVIEW_RESULT ?= $(if $(GEMMA_REVIEW_TASK_ID),/tmp/dubbridge-gemma-review-$(GEMMA_REVIEW_TASK_ID).json,/tmp/dubbridge-gemma-review.json)
GEMMA_REVIEW_CONTEXT_METADATA ?= $(if $(GEMMA_REVIEW_TASK_ID),/tmp/dubbridge-review-context-$(GEMMA_REVIEW_TASK_ID).json,/tmp/dubbridge-review-context.json)
GEMMA_EVIDENCE_DIR   ?= docs/audit/gemma-evidence
REVIEW_PATHS         ?=
# Optional task/acceptance source and explicit read-authority scope for M3 local
# reviewer enrichment. With no explicit REVIEW_CONTEXT_ALLOWED_PATHS, the local
# reviewer may read only CKG-selected files contained in the current worktree.
# Cross-vendor qa-peer-workflow-review deliberately does not use these variables.
REVIEW_TASK_FILE     ?=
REVIEW_CONTEXT_ALLOWED_PATHS ?=
GOLDEN_SET_MODEL     ?= gemma4:26b-a4b-it-qat
GOLDEN_SET_RESULT    ?= /tmp/dubbridge-golden-set.json
COVERAGE_IGNORE_REGEX ?= (apps/(api|cli|worker-runner)/src/(main|cleanup)\.rs|apps/api/src/(dto/ingestion|lib|routes/ingestion|state)\.rs|crates/(db|jobs|observability)/src/lib\.rs|crates/db/src/(artifact_repo|asset_repo|audit_repo|pending_ingestion_repo|rights_repo)\.rs|crates/(audit|ingestion)/src/lib\.rs)
CARGO ?= $(if $(shell command -v cargo 2>/dev/null),$(shell command -v cargo),$(HOME)/.cargo/bin/cargo)
PYTHON ?= python3
RUFF_VERSION ?= 0.16.5

qa-fmt:
	$(CARGO) fmt --all -- --check

qa-lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

qa-test:
	$(CARGO) test --workspace --all-features

qa-test-redis:
	@if [ -z "$${DUBBRIDGE_REDIS_URL:-}" ]; then \
		echo "DUBBRIDGE_REDIS_URL must be set for qa-test-redis (for example redis://127.0.0.1:6379/15)"; \
		exit 1; \
	fi
	$(CARGO) test -p dubbridge-jobs --all-features redis_ -- --ignored --test-threads=1

qa-test-s3:
	@set -eu; \
		: "$${DUBBRIDGE_STORAGE_TEST_ENDPOINT:?DUBBRIDGE_STORAGE_TEST_ENDPOINT must be set for qa-test-s3}"; \
		: "$${DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID:?DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID must be set for qa-test-s3}"; \
		: "$${DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY:?DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY must be set for qa-test-s3}"; \
		: "$${DUBBRIDGE_STORAGE_TEST_BUCKET:?DUBBRIDGE_STORAGE_TEST_BUCKET must be set for qa-test-s3}"
	$(CARGO) test -p dubbridge-storage --all-features s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint -- --ignored --test-threads=1

qa-check:
	$(CARGO) check --workspace --all-targets --all-features

qa-local: qa-fmt qa-lint qa-test qa-check

qa-deny:
	$(CARGO) deny check

qa-config-secrets:
	bash scripts/check-config-secrets.sh

qa-roadmap-drift:
	bash scripts/check-roadmap-drift.sh

qa-coverage:
	$(CARGO) llvm-cov --workspace --summary-only --fail-under-lines $(COVERAGE_MIN) \
		--ignore-filename-regex '$(COVERAGE_IGNORE_REGEX)' \
		-- --test-threads=1

qa-build-release:
	$(CARGO) build --workspace --release

qa-maintainability:
	python3 scripts/check-maintainability.py

qa-python-complexity:
	@$(PYTHON) -c "import ruff" >/dev/null 2>&1 || { \
		echo "ruff $(RUFF_VERSION) is required; install with: $(PYTHON) -m pip install ruff==$(RUFF_VERSION)" >&2; \
		exit 1; \
	}
	$(PYTHON) -m ruff check workers --config ruff.toml

# Pre-delegation reviewability budget: fail closed when added/changed code lines
# exceed the budget derived from Gemma's context window, so a change handed to
# the local reviewer/developer fits in-context. Documented escape: a
# `D14-OVERRIDE: <reason>` line in the commit body routes the change to a
# non-Gemma (D14) reviewer instead.
qa-review-budget:
	python3 scripts/check-review-budget.py $(if $(REVIEW_PATHS),--files $(REVIEW_PATHS))

# Mobile production-readiness + correctness: strict types, AST lint (no any /
# console / debugger / ts-suppression), and the Jest suite. Replaces the former
# regex production-readiness scan for the mobile surface.
qa-mobile:
	cd mobile && npm run typecheck && npm run lint && npm test
	python3 scripts/check-primary-label-usage.py

# DESIGN.md stays on an explicit opt-in gate for now because the Google CLI and
# spec are still alpha and should not widen the main CI surface by default.
qa-design:
	npx -y @google/design.md lint DESIGN.md

qa-task-unit-coverage:
	python3 scripts/check_task_unit_coverage_test.py
	bash scripts/check-task-unit-coverage.sh

# Deterministic doc gates only (no LLM review). Safe to run on every push.
qa-docs:
	bash scripts/check-doc-consistency.sh
	python3 scripts/check_task_unit_coverage_test.py
	bash scripts/check-task-unit-coverage.sh
	bash scripts/check-roadmap-drift.sh
	python3 scripts/check_okf_frontmatter.py

# qa-docs plus the Gemma Reviewer LLM pass. Reserved for task closure (Phase 2)
# and CI, per docs/playbooks/AGENT_WORKFLOW_GUIDE.md — not for pre-push.
qa-docs-review: qa-docs
	$(MAKE) qa-gemma-review

qa-okf-frontmatter:
	python3 scripts/check_okf_frontmatter.py

qa-rri:
	python3 scripts/rri_test.py
	python3 scripts/check_roadmap_drift_test.py

qa-ci: qa-local qa-docs-review qa-rri qa-deny qa-config-secrets qa-roadmap-drift qa-maintainability qa-python-complexity qa-review-budget qa-mobile qa-coverage qa-build-release

qa-gemma-review:
	@if [ "$${DUBBRIDGE_SKIP_GEMMA_REVIEW:-0}" = "1" ]; then \
		echo "[gemma-review] skipped (DUBBRIDGE_SKIP_GEMMA_REVIEW=1)"; exit 0; \
	fi; \
	code_changes=$$(git diff --name-only $(GEMMA_REVIEW_BASE) -- $(REVIEW_PATHS) 2>/dev/null \
		| grep -vE '^(docs/|[^/]+\.md$$)' || true); \
	if [ -z "$$code_changes" ]; then \
		echo "[gemma-review] no code changes vs $(GEMMA_REVIEW_BASE); skipped"; exit 0; \
	fi; \
	: "AC3 asserts the stale result IS gone, not that removal was attempted -- rm's own status is ignored on purpose; absence is the post-condition."; \
	rm -f "$(GEMMA_REVIEW_RESULT)" 2>/dev/null || true; \
	if [ -e "$(GEMMA_REVIEW_RESULT)" ]; then \
		echo "[gemma-review] could not clear stale result at $(GEMMA_REVIEW_RESULT); aborting rather than risk reusing it" >&2; \
		exit 1; \
	fi; \
	review_status=0; \
	packet_file=$$(mktemp -t dubbridge-review-diff.XXXXXX); \
	trap 'rm -f "$$packet_file"' EXIT HUP INT TERM; \
	git diff $(GEMMA_REVIEW_BASE) -- $(REVIEW_PATHS) > "$$packet_file"; \
	if [ -f scripts/review_context.py ]; then \
		set -- python3 scripts/review_context.py "$$packet_file" --worktree . \
			--task-id "$(GEMMA_REVIEW_TASK_ID)" --metadata-out "$(GEMMA_REVIEW_CONTEXT_METADATA)"; \
		if [ -n "$(REVIEW_TASK_FILE)" ]; then set -- "$$@" --acceptance-file "$(REVIEW_TASK_FILE)"; fi; \
		for review_path in $(REVIEW_CONTEXT_ALLOWED_PATHS); do set -- "$$@" --allowed-path "$$review_path"; done; \
		"$$@" | python3 scripts/gemma-code-review.py --out "$(GEMMA_REVIEW_RESULT)" - || review_status=$$?; \
	else \
		{ echo "# Gemma Reviewer packet (base: $(GEMMA_REVIEW_BASE))"; echo ""; cat "$$packet_file"; } \
		| python3 scripts/gemma-code-review.py --out "$(GEMMA_REVIEW_RESULT)" - || review_status=$$?; \
	fi; \
	if [ "$$review_status" != "0" ]; then \
		echo "[gemma-review] review command failed (exit $$review_status); aborting before receipt, no stale result reused" >&2; \
		exit $$review_status; \
	fi; \
	echo "[gemma-review] result written to $(GEMMA_REVIEW_RESULT)"; \
	findings_status=0; \
	python3 scripts/parse-review-findings.py "$(GEMMA_REVIEW_RESULT)" || findings_status=$$?; \
	if [ -n "$(GEMMA_REVIEW_TASK_ID)" ]; then \
		mkdir -p "$(GEMMA_EVIDENCE_DIR)"; \
		verdict="PASS"; [ "$$findings_status" != "0" ] && verdict="FINDINGS-ACKED"; \
		reviewer=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('model') or 'unknown-reviewer')" "$(GEMMA_REVIEW_RESULT)" 2>/dev/null || echo "unknown-reviewer"); \
		changed_paths_json=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(json.dumps(d.get('changed_paths') or []))" "$(GEMMA_REVIEW_RESULT)" 2>/dev/null || echo "[]"); \
		commit_sha=$$(git rev-parse HEAD); \
		timestamp=$$(date -u +%Y-%m-%dT%H:%M:%SZ); \
		printf '{"task_id":"%s","commit_sha":"%s","reviewer":"%s","verdict":"%s","timestamp":"%s","changed_paths":%s}\n' \
			"$(GEMMA_REVIEW_TASK_ID)" "$$commit_sha" "$$reviewer" "$$verdict" "$$timestamp" "$$changed_paths_json" \
			> "$(GEMMA_EVIDENCE_DIR)/$(GEMMA_REVIEW_TASK_ID).json"; \
		echo "[gemma-review] receipt written to $(GEMMA_EVIDENCE_DIR)/$(GEMMA_REVIEW_TASK_ID).json"; \
	fi; \
	exit $$findings_status

qa-golden-set:
	@if [ "$${DUBBRIDGE_SKIP_GOLDEN_SET:-0}" = "1" ]; then \
		echo "[golden-set] skipped (DUBBRIDGE_SKIP_GOLDEN_SET=1)"; exit 0; \
	fi; \
	python3 scripts/local-agent/golden_set.py \
		--model "$(GOLDEN_SET_MODEL)" \
		--out "$(GOLDEN_SET_RESULT)" \
	&& echo "[golden-set] result written to $(GOLDEN_SET_RESULT)"

qa-gemma-push-review:
	@if [ "$${DUBBRIDGE_SKIP_GEMMA_PUSH_REVIEW:-0}" = "1" ]; then \
		echo "[gemma-push-review] skipped (DUBBRIDGE_SKIP_GEMMA_PUSH_REVIEW=1)"; exit 0; \
	fi; \
	set -- python3 scripts/gemma-push-review.py; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_RUN_ID:-}" ]; then \
		set -- "$$@" --run-id "$${DUBBRIDGE_PUSH_REVIEW_RUN_ID}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_WORKFLOW:-}" ]; then \
		set -- "$$@" --workflow "$${DUBBRIDGE_PUSH_REVIEW_WORKFLOW}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_BRANCH:-}" ]; then \
		set -- "$$@" --branch "$${DUBBRIDGE_PUSH_REVIEW_BRANCH}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_BEFORE:-}" ]; then \
		set -- "$$@" --before "$${DUBBRIDGE_PUSH_REVIEW_BEFORE}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_AFTER:-}" ]; then \
		set -- "$$@" --after "$${DUBBRIDGE_PUSH_REVIEW_AFTER}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_EVENT_PATH:-}" ]; then \
		set -- "$$@" --event-path "$${DUBBRIDGE_PUSH_REVIEW_EVENT_PATH}"; \
	fi; \
	if [ -n "$${DUBBRIDGE_PUSH_REVIEW_OUT_DIR:-}" ]; then \
		set -- "$$@" --out-dir "$${DUBBRIDGE_PUSH_REVIEW_OUT_DIR}"; \
	fi; \
	if [ "$${DUBBRIDGE_PUSH_REVIEW_FORCE:-0}" = "1" ]; then \
		set -- "$$@" --force; \
	fi; \
	if [ "$${DUBBRIDGE_PUSH_REVIEW_COLLECT_ONLY:-0}" = "1" ]; then \
		set -- "$$@" --collect-only; \
	fi; \
	if [ "$${DUBBRIDGE_PUSH_REVIEW_DRY_RUN:-0}" = "1" ]; then \
		set -- "$$@" --dry-run; \
	fi; \
	echo "[gemma-push-review] running $$1"; \
	"$$@"

# Band-routed two-phase peer-workflow review (PPR-3).
# Reads git diff from PEER_REVIEW_BASE and routes to Gemma (RRI 0-40) or
# cross-vendor peer (RRI 41+) per the contract in docs/plan/portable-peer-review-gate.md.
# Set PEER_REVIEW_DRY_RUN=1 to resolve routing without invoking any model.
# Set DUBBRIDGE_SKIP_PEER_REVIEW=1 to skip entirely (e.g. in CI without Ollama).
qa-peer-workflow-review:
	@if [ "$${DUBBRIDGE_SKIP_PEER_REVIEW:-0}" = "1" ]; then \
		echo "[peer-review] skipped (DUBBRIDGE_SKIP_PEER_REVIEW=1)"; exit 0; \
	fi; \
	set -- --phase "$(PEER_REVIEW_PHASE)" --rri "$(PEER_REVIEW_RRI)" \
	       --caller "$(PEER_REVIEW_CALLER)" --artifact "$(PEER_REVIEW_ARTIFACT)"; \
	if [ -n "$(PEER_REVIEW_TASK_ID)" ]; then set -- "$$@" --task-id "$(PEER_REVIEW_TASK_ID)"; fi; \
	if [ "$${PEER_REVIEW_DRY_RUN:-0}" = "1" ]; then set -- "$$@" --dry-run; fi; \
	review_status=0; \
	git diff "$(PEER_REVIEW_BASE)" -- $(REVIEW_PATHS) | python3 scripts/peer-workflow-review.py "$$@" --content - || review_status=$$?; \
	if [ -n "$(PEER_REVIEW_TASK_ID)" ] && [ -f "$(PEER_REVIEW_ARTIFACT)" ]; then \
		mkdir -p "$(GEMMA_EVIDENCE_DIR)"; \
		verdict=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); v=d.get('verdict','unknown'); print('PASS' if v == 'pass' else 'FINDINGS-ACKED')" "$(PEER_REVIEW_ARTIFACT)" 2>/dev/null || echo "FINDINGS-ACKED"); \
		reviewer=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('reviewer') or d.get('model') or 'peer')" "$(PEER_REVIEW_ARTIFACT)" 2>/dev/null || echo "peer"); \
		changed_paths_json=$$(git diff --name-only "$(PEER_REVIEW_BASE)" -- $(REVIEW_PATHS) 2>/dev/null \
			| python3 -c "import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))" 2>/dev/null || echo "[]"); \
		commit_sha=$$(git rev-parse HEAD); \
		timestamp=$$(date -u +%Y-%m-%dT%H:%M:%SZ); \
		printf '{"task_id":"%s","commit_sha":"%s","reviewer":"%s","verdict":"%s","timestamp":"%s","changed_paths":%s}\n' \
			"$(PEER_REVIEW_TASK_ID)" "$$commit_sha" "$$reviewer" "$$verdict" "$$timestamp" "$$changed_paths_json" \
			> "$(GEMMA_EVIDENCE_DIR)/$(PEER_REVIEW_TASK_ID).json"; \
		echo "[peer-review] receipt written to $(GEMMA_EVIDENCE_DIR)/$(PEER_REVIEW_TASK_ID).json"; \
	fi; \
	exit $$review_status

show-codex-session-model:
	python3 scripts/show-codex-session-model.py

install-hooks:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-push
	@echo "Git hooks installed (core.hooksPath=.githooks)."
