CONFIG ?= config.toml

.PHONY: build test run clean lint format dev dev-gsm dev-sip docker-build coverage help

build: ## Compile all binaries (release mode)
	@cargo build --workspace --release

test: ## Run the full test suite
	@cargo test --workspace --all-features

run: build ## Build and run the GSM-SIP bridge
	@cargo run --release --bin gsm-sip-bridge -- --config $(CONFIG)

clean: ## Remove all build artifacts
	@cargo clean

lint: ## Run formatting check, clippy, and cargo-deny
	@cargo fmt --check
	@cargo clippy --workspace -- -D warnings
	@if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; fi
	@if [ -f tools/count-unsafe.sh ]; then bash tools/count-unsafe.sh; fi

format: ## Auto-format all Rust source files
	@cargo fmt

dev: ## Run in debug mode with verbose logging
	@RUST_LOG=debug,gsm_sip_bridge=trace cargo run --bin gsm-sip-bridge -- --config $(CONFIG) --verbose

dev-gsm: ## [Debug] Run GSM-only audio loopback
	@cargo run --bin gsm-echo

dev-sip: ## [Debug] Run SIP-only audio loopback
	@cargo run --bin sip-echo -- --config $(CONFIG) --verbose

docker-build: ## Build the production Docker image
	@docker compose -f docker/docker-compose.yml build

coverage: ## Generate code coverage report
	@cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
	@cargo llvm-cov report --workspace --all-features

help: ## Show all available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
