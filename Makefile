BUILD_DIR := build
BINARY := $(BUILD_DIR)/audio-echo

.PHONY: build test run clean lint help

build: ## Compile the audio-echo binary
	@cmake -B $(BUILD_DIR) -DCMAKE_BUILD_TYPE=Release -S .
	@cmake --build $(BUILD_DIR) --parallel

test: ## Run the full integration test suite
	@cmake -B $(BUILD_DIR) -DCMAKE_BUILD_TYPE=Debug -S .
	@cmake --build $(BUILD_DIR) --parallel
	@cd $(BUILD_DIR) && ctest --output-on-failure

run: build ## Build and run audio-echo with auto-detection
	@$(BINARY)

clean: ## Remove all build artifacts
	@rm -rf $(BUILD_DIR)

lint: ## Run static analysis on all source files
	@cppcheck --enable=all --std=c++17 --suppress=missingIncludeSystem \
		--error-exitcode=1 -I src src/ 2>&1 | grep -v "^Checking" || true
	@echo "Lint complete."

help: ## Show all available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'
