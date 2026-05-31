.PHONY: qa-fmt qa-lint qa-test qa-check qa-local qa-deny qa-coverage qa-build-release qa-docs qa-ci

COVERAGE_MIN ?= 90
COVERAGE_IGNORE_REGEX ?= (apps/(api|cli|worker-runner)/src/main\.rs|crates/(db|jobs|observability)/src/lib\.rs)
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

qa-coverage:
	$(CARGO) llvm-cov --workspace --summary-only --fail-under-lines $(COVERAGE_MIN) \
		--ignore-filename-regex '$(COVERAGE_IGNORE_REGEX)'

qa-build-release:
	$(CARGO) build --workspace --release

qa-docs:
	bash scripts/check-doc-consistency.sh

qa-ci: qa-local qa-docs qa-deny qa-coverage qa-build-release
