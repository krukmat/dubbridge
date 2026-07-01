.PHONY: qa-fmt qa-lint qa-test qa-check qa-local qa-deny qa-config-secrets qa-roadmap-drift qa-coverage qa-build-release qa-maintainability qa-review-budget qa-mobile qa-design qa-task-unit-coverage qa-docs qa-rri qa-ci qa-gemma-review qa-gemma-push-review qa-peer-workflow-review install-hooks

COVERAGE_MIN ?= 90
PEER_REVIEW_RRI      ?= 22
PEER_REVIEW_PHASE    ?= code
PEER_REVIEW_CALLER   ?= claude-code
PEER_REVIEW_TASK_ID  ?=
PEER_REVIEW_ARTIFACT ?= /tmp/dubbridge-peer-review.json
PEER_REVIEW_BASE     ?= HEAD
GEMMA_REVIEW_BASE   ?= HEAD
GEMMA_REVIEW_RESULT ?= /tmp/dubbridge-gemma-review.json
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
	bash scripts/check-task-unit-coverage.sh

qa-docs:
	$(MAKE) qa-gemma-review
	bash scripts/check-doc-consistency.sh
	bash scripts/check-task-unit-coverage.sh
	bash scripts/check-roadmap-drift.sh
	python3 scripts/check_okf_frontmatter.py

qa-okf-frontmatter:
	python3 scripts/check_okf_frontmatter.py

qa-rri:
	python3 scripts/rri_test.py
	python3 scripts/check_roadmap_drift_test.py

qa-ci: qa-local qa-docs qa-rri qa-deny qa-config-secrets qa-roadmap-drift qa-maintainability qa-review-budget qa-mobile qa-coverage qa-build-release

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
	python3 scripts/parse-review-findings.py "$(GEMMA_REVIEW_RESULT)"

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
	git diff "$(PEER_REVIEW_BASE)" | python3 scripts/peer-workflow-review.py $$args --content -

install-hooks:
	cp scripts/hooks/pre-commit .git/hooks/pre-commit
	cp scripts/hooks/pre-push .git/hooks/pre-push
	chmod +x .git/hooks/pre-commit .git/hooks/pre-push
	@echo "Git hooks installed."
