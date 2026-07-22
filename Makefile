.PHONY: qa-fmt qa-lint qa-test qa-check qa-local qa-deny qa-config-secrets qa-roadmap-drift qa-coverage qa-build-release qa-maintainability qa-review-budget qa-mobile qa-design qa-task-unit-coverage qa-docs qa-docs-review qa-rri qa-ci qa-gemma-review qa-gemma-push-review qa-peer-workflow-review show-codex-session-model install-hooks

COVERAGE_MIN ?= 90
PEER_REVIEW_RRI      ?= 22
PEER_REVIEW_PHASE    ?= code
PEER_REVIEW_CALLER   ?= claude-code
PEER_REVIEW_TASK_ID  ?=
PEER_REVIEW_ARTIFACT ?= /tmp/dubbridge-peer-review.json
PEER_REVIEW_BASE     ?= HEAD
GEMMA_REVIEW_BASE   ?= HEAD
GEMMA_REVIEW_RESULT ?= /tmp/dubbridge-gemma-review.json
GEMMA_REVIEW_TASK_ID ?=
GEMMA_EVIDENCE_DIR   ?= docs/audit/gemma-evidence
COVERAGE_IGNORE_REGEX ?= (apps/(api|cli|worker-runner)/src/(main|cleanup)\.rs|apps/api/src/(dto/ingestion|lib|routes/ingestion|state)\.rs|crates/(db|jobs|observability)/src/lib\.rs|crates/db/src/(artifact_repo|asset_repo|audit_repo|pending_ingestion_repo|rights_repo)\.rs|crates/(audit|ingestion)/src/lib\.rs)
CARGO ?= $(if $(shell command -v cargo 2>/dev/null),$(shell command -v cargo),$(HOME)/.cargo/bin/cargo)

qa-fmt:
	$(CARGO) fmt --all -- --check

qa-lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

qa-test:
	$(CARGO) test --workspace --all-features

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

# Pre-delegation reviewability budget: fail closed when added/changed code lines
# exceed the budget derived from Gemma's context window, so a change handed to
# the local reviewer/developer fits in-context. Documented escape: a
# `D14-OVERRIDE: <reason>` line in the commit body routes the change to a
# non-Gemma (D14) reviewer instead.
qa-review-budget:
	python3 scripts/check-review-budget.py

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

qa-ci: qa-local qa-docs-review qa-rri qa-deny qa-config-secrets qa-roadmap-drift qa-maintainability qa-review-budget qa-mobile qa-coverage qa-build-release

qa-gemma-review:
	@if [ "$${DUBBRIDGE_SKIP_GEMMA_REVIEW:-0}" = "1" ]; then \
		echo "[gemma-review] skipped (DUBBRIDGE_SKIP_GEMMA_REVIEW=1)"; exit 0; \
	fi; \
	code_changes=$$(git diff --name-only $(GEMMA_REVIEW_BASE) 2>/dev/null \
		| grep -vE '^(docs/|[^/]+\.md$$)' || true); \
	if [ -z "$$code_changes" ]; then \
		echo "[gemma-review] no code changes vs $(GEMMA_REVIEW_BASE); skipped"; exit 0; \
	fi; \
	{ echo "# Gemma Reviewer packet (base: $(GEMMA_REVIEW_BASE))"; echo ""; \
	  git diff $(GEMMA_REVIEW_BASE); } \
	| python3 scripts/gemma-code-review.py --out "$(GEMMA_REVIEW_RESULT)" - \
	&& echo "[gemma-review] result written to $(GEMMA_REVIEW_RESULT)"; \
	findings_status=0; \
	python3 scripts/parse-review-findings.py "$(GEMMA_REVIEW_RESULT)" || findings_status=$$?; \
	if [ -n "$(GEMMA_REVIEW_TASK_ID)" ]; then \
		mkdir -p "$(GEMMA_EVIDENCE_DIR)"; \
		verdict="PASS"; [ "$$findings_status" != "0" ] && verdict="FINDINGS-ACKED"; \
		commit_sha=$$(git rev-parse HEAD); \
		timestamp=$$(date -u +%Y-%m-%dT%H:%M:%SZ); \
		printf '{"task_id":"%s","commit_sha":"%s","reviewer":"gemma","verdict":"%s","timestamp":"%s"}\n' \
			"$(GEMMA_REVIEW_TASK_ID)" "$$commit_sha" "$$verdict" "$$timestamp" \
			> "$(GEMMA_EVIDENCE_DIR)/$(GEMMA_REVIEW_TASK_ID).json"; \
		echo "[gemma-review] receipt written to $(GEMMA_EVIDENCE_DIR)/$(GEMMA_REVIEW_TASK_ID).json"; \
	fi; \
	exit $$findings_status

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
	args="--phase $(PEER_REVIEW_PHASE) --rri $(PEER_REVIEW_RRI) \
	      --caller $(PEER_REVIEW_CALLER) --artifact $(PEER_REVIEW_ARTIFACT)"; \
	if [ -n "$(PEER_REVIEW_TASK_ID)" ]; then args="$$args --task-id $(PEER_REVIEW_TASK_ID)"; fi; \
	if [ "$${PEER_REVIEW_DRY_RUN:-0}" = "1" ]; then args="$$args --dry-run"; fi; \
	review_status=0; \
	git diff "$(PEER_REVIEW_BASE)" | python3 scripts/peer-workflow-review.py $$args --content - || review_status=$$?; \
	if [ -n "$(PEER_REVIEW_TASK_ID)" ] && [ -f "$(PEER_REVIEW_ARTIFACT)" ]; then \
		mkdir -p "$(GEMMA_EVIDENCE_DIR)"; \
		verdict=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); v=d.get('verdict','unknown'); print('PASS' if v == 'pass' else 'FINDINGS-ACKED')" "$(PEER_REVIEW_ARTIFACT)" 2>/dev/null || echo "FINDINGS-ACKED"); \
		reviewer=$$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('reviewer') or d.get('model') or 'peer')" "$(PEER_REVIEW_ARTIFACT)" 2>/dev/null || echo "peer"); \
		commit_sha=$$(git rev-parse HEAD); \
		timestamp=$$(date -u +%Y-%m-%dT%H:%M:%SZ); \
		printf '{"task_id":"%s","commit_sha":"%s","reviewer":"%s","verdict":"%s","timestamp":"%s"}\n' \
			"$(PEER_REVIEW_TASK_ID)" "$$commit_sha" "$$reviewer" "$$verdict" "$$timestamp" \
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
