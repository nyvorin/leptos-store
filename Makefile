# ============================================================================
# leptos-store Makefile
# ============================================================================
# Enterprise-grade, type-enforced state management for Leptos
# ============================================================================

.PHONY: all build check test test-all clippy fmt fmt-check doc doc-open \
        clean example example-build example-release publish publish-dry \
        deps audit outdated help examples-list run showcase build-example build-example-release \
        test-example-pkg check-example clean-example test-all-examples \
        check-all-examples clean-all-examples \
        showcase-css showcase-dev showcase-build docker-build docker-run docker

# Colors for terminal output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

# Default target
all: check test

# ============================================================================
# Building
# ============================================================================

## Build the library in debug mode
build:
	@echo "$(CYAN)Building leptos-store...$(RESET)"
	cargo build

## Build the library in release mode
build-release:
	@echo "$(CYAN)Building leptos-store (release)...$(RESET)"
	cargo build --release

## Check the library compiles without building
check:
	@echo "$(CYAN)Checking leptos-store...$(RESET)"
	cargo check --workspace

## Check all feature combinations
check-features:
	@echo "$(CYAN)Checking all feature combinations...$(RESET)"
	cargo check --no-default-features
	cargo check --features ssr
	cargo check --features hydrate
	cargo check --features csr
	cargo check --all-features

# ============================================================================
# Testing
# ============================================================================

## Run all tests
test:
	@echo "$(CYAN)Running tests...$(RESET)"
	cargo test --workspace

## Run tests with output
test-verbose:
	@echo "$(CYAN)Running tests (verbose)...$(RESET)"
	cargo test --workspace -- --nocapture

## Run only library tests
test-lib:
	@echo "$(CYAN)Running library tests...$(RESET)"
	cargo test --lib

## Run only doc tests
test-doc:
	@echo "$(CYAN)Running doc tests...$(RESET)"
	cargo test --doc

## Run tests for the example
test-example:
	@echo "$(CYAN)Running example tests...$(RESET)"
	cargo test -p auth-store-example

## Run all tests with all features
test-all:
	@echo "$(CYAN)Running all tests with all features...$(RESET)"
	cargo test --workspace --all-features

## Run tests with coverage (requires cargo-tarpaulin)
coverage:
	@echo "$(CYAN)Running tests with coverage...$(RESET)"
	cargo tarpaulin --workspace --out Html --output-dir target/coverage

# ============================================================================
# Code Quality
# ============================================================================

## Run clippy lints
clippy:
	@echo "$(CYAN)Running clippy...$(RESET)"
	cargo clippy --workspace --all-targets -- -D warnings

## Run clippy with fixes
clippy-fix:
	@echo "$(CYAN)Running clippy with fixes...$(RESET)"
	cargo clippy --workspace --all-targets --fix --allow-dirty

## Format code
fmt:
	@echo "$(CYAN)Formatting code...$(RESET)"
	cargo fmt --all

## Check code formatting
fmt-check:
	@echo "$(CYAN)Checking code format...$(RESET)"
	cargo fmt --all -- --check

# ============================================================================
# Documentation
# ============================================================================

## Build documentation
doc:
	@echo "$(CYAN)Building documentation...$(RESET)"
	cargo doc --workspace --no-deps

## Build and open documentation
doc-open:
	@echo "$(CYAN)Building and opening documentation...$(RESET)"
	cargo doc --workspace --no-deps --open

## Build documentation with private items
doc-private:
	@echo "$(CYAN)Building documentation (including private items)...$(RESET)"
	cargo doc --workspace --no-deps --document-private-items

# ============================================================================
# Example Applications (Generic)
# ============================================================================
# Use these commands to work with any example in the examples/ folder.
# Pass NAME=<example-name> to specify which example to use.
#
# Examples:
#   make examples-list                    # List all available examples
#   make run NAME=token-explorer-example  # Run specific example
#   make build-example NAME=auth-store-example  # Build specific example
# ============================================================================

# Default example (for backwards compatibility)
DEFAULT_EXAMPLE := auth-store-example

## Install cargo-leptos if not present
leptos-install:
	@echo "$(CYAN)Installing cargo-leptos...$(RESET)"
	@command -v cargo-leptos >/dev/null 2>&1 || cargo install cargo-leptos

## Install trunk (WASM bundler) if not present
trunk-install:
	@echo "$(CYAN)Installing trunk...$(RESET)"
	@command -v trunk >/dev/null 2>&1 || cargo install trunk

## Add WASM target if not present
wasm-target:
	@echo "$(CYAN)Adding wasm32-unknown-unknown target...$(RESET)"
	rustup target add wasm32-unknown-unknown

## Setup example dependencies
example-setup: wasm-target
	@echo "$(GREEN)Example setup complete!$(RESET)"

## List all available examples
examples-list:
	@echo "$(CYAN)Available examples:$(RESET)"
	@for dir in examples/*/; do \
		name=$$(basename "$$dir"); \
		if [ -f "$$dir/Cargo.toml" ]; then \
			desc=$$(grep '^description' "$$dir/Cargo.toml" 2>/dev/null | head -1 | sed 's/.*= *"\(.*\)"/\1/' || echo ""); \
			port=$$(grep 'site-addr' "$$dir/Cargo.toml" 2>/dev/null | head -1 | sed 's/.*:\([0-9]*\)".*/\1/' || echo "N/A"); \
			echo "  $(GREEN)$$name$(RESET)"; \
			if [ -n "$$desc" ]; then echo "    $$desc"; fi; \
			if [ "$$port" != "N/A" ] && [ -n "$$port" ]; then echo "    Port: $$port"; fi; \
		fi \
	done
	@echo ""
	@echo "$(YELLOW)Usage:$(RESET)"
	@echo "  make run NAME=<example-name>"
	@echo "  make build-example NAME=<example-name>"
	@echo "  make test-example-pkg NAME=<example-name>"

## Run an example by name (SSR mode with cargo-leptos)
## Usage: make run NAME=token-explorer-example
run: example-setup leptos-install
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make run NAME=<example-name>"; \
		echo "Run 'make examples-list' to see available examples"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		echo "Run 'make examples-list' to see available examples"; \
		exit 1; \
	fi
	@port=$$(grep 'site-addr' "examples/$(NAME)/Cargo.toml" 2>/dev/null | head -1 | sed 's/.*:\([0-9]*\)".*/\1/' || echo "3000"); \
	echo "$(CYAN)Starting $(NAME) (SSR mode)...$(RESET)"; \
	echo "$(YELLOW)Open http://127.0.0.1:$$port in your browser$(RESET)"; \
	cd examples/$(NAME) && cargo leptos watch

## Run the Examples Showcase (unified gallery of all examples)
showcase: showcase-css example-setup leptos-install
	@echo "$(CYAN)Starting Examples Showcase...$(RESET)"
	@echo "$(YELLOW)Open http://127.0.0.1:3100 in your browser$(RESET)"
	cd examples/showcase && cargo leptos watch

## Generate scoped CSS for showcase (concatenates all example CSS with .demo-{id} nesting)
SHOWCASE_EXAMPLES := counter auth-store token-explorer middleware persistence composition feature-flags devtools csr selectors
showcase-css:
	@echo "$(CYAN)Generating showcase CSS...$(RESET)"
	@echo "/* ============================================================================" > examples/showcase/style/main.css
	@echo " * leptos-store Showcase — Unified Scoped CSS" >> examples/showcase/style/main.css
	@echo " * Auto-generated by: make showcase-css" >> examples/showcase/style/main.css
	@echo " * DO NOT EDIT — regenerate with: make showcase-css" >> examples/showcase/style/main.css
	@echo " * ============================================================================ */" >> examples/showcase/style/main.css
	@echo "" >> examples/showcase/style/main.css
	@for id in $(SHOWCASE_EXAMPLES); do \
		css_file="examples/$${id}-example/style/main.css"; \
		if [ -f "$$css_file" ]; then \
			echo "/* --- $$id --- */" >> examples/showcase/style/main.css; \
			echo ".demo-$$id {" >> examples/showcase/style/main.css; \
			cat "$$css_file" | sed 's/^/  /' >> examples/showcase/style/main.css; \
			echo "" >> examples/showcase/style/main.css; \
			echo "}" >> examples/showcase/style/main.css; \
			echo "" >> examples/showcase/style/main.css; \
		fi \
	done
	@echo "$(GREEN)Showcase CSS generated: examples/showcase/style/main.css$(RESET)"

## Run showcase in dev mode (regenerate CSS + watch)
showcase-dev: showcase-css example-setup leptos-install
	@echo "$(CYAN)Starting Examples Showcase (dev mode)...$(RESET)"
	@echo "$(YELLOW)Open http://127.0.0.1:3100 in your browser$(RESET)"
	cd examples/showcase && cargo leptos watch

## Build showcase for production (SSR binary + WASM client)
showcase-build: showcase-css example-setup leptos-install
	@echo "$(CYAN)Building showcase (release)...$(RESET)"
	cd examples/showcase && cargo leptos build --release
	@echo "$(GREEN)Showcase built: target/release/showcase + examples/showcase/target/site/$(RESET)"

## Build Docker image for showcase
docker-build: showcase-css
	@echo "$(CYAN)Building Docker image...$(RESET)"
	docker build -t leptos-store-showcase .
	@echo "$(GREEN)Docker image built: leptos-store-showcase$(RESET)"

## Run showcase Docker container locally
docker-run:
	@echo "$(CYAN)Starting showcase container on :8080...$(RESET)"
	@echo "$(YELLOW)Open http://127.0.0.1:8080 in your browser$(RESET)"
	docker run --rm -p 8080:8080 leptos-store-showcase

## Build and run Docker container
docker: docker-build docker-run

## Build an example by name (SSR mode)
## Usage: make build-example NAME=token-explorer-example
build-example: example-setup leptos-install
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make build-example NAME=<example-name>"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Building $(NAME) (SSR)...$(RESET)"
	cd examples/$(NAME) && cargo leptos build

## Build an example in release mode (SSR)
## Usage: make build-example-release NAME=token-explorer-example
build-example-release: example-setup leptos-install
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make build-example-release NAME=<example-name>"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Building $(NAME) (SSR, release)...$(RESET)"
	cd examples/$(NAME) && cargo leptos build --release

## Test an example by name
## Usage: make test-example-pkg NAME=token-explorer-example
test-example-pkg:
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make test-example-pkg NAME=<example-name>"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Testing $(NAME)...$(RESET)"
	cargo test -p $(NAME) --features hydrate

## Check an example compiles (both SSR and hydrate)
## Usage: make check-example NAME=token-explorer-example
check-example:
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make check-example NAME=<example-name>"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Checking $(NAME) (SSR)...$(RESET)"
	cargo check -p $(NAME) --features ssr
	@echo "$(CYAN)Checking $(NAME) (hydrate)...$(RESET)"
	cargo check -p $(NAME) --features hydrate --target wasm32-unknown-unknown

## Clean an example's build artifacts
## Usage: make clean-example NAME=token-explorer-example
clean-example:
	@if [ -z "$(NAME)" ]; then \
		echo "$(RED)Error: NAME is required$(RESET)"; \
		echo "Usage: make clean-example NAME=<example-name>"; \
		exit 1; \
	fi
	@if [ ! -d "examples/$(NAME)" ]; then \
		echo "$(RED)Error: Example '$(NAME)' not found$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Cleaning $(NAME) build artifacts...$(RESET)"
	rm -rf examples/$(NAME)/dist
	rm -rf examples/$(NAME)/target/site

## Run all examples' tests
test-all-examples:
	@echo "$(CYAN)Testing all examples...$(RESET)"
	@for dir in examples/*/; do \
		name=$$(basename "$$dir"); \
		if [ -f "$$dir/Cargo.toml" ]; then \
			echo "$(CYAN)Testing $$name...$(RESET)"; \
			cargo test -p "$$name" --features hydrate || exit 1; \
		fi \
	done
	@echo "$(GREEN)All example tests passed!$(RESET)"

## Check all examples compile
check-all-examples: wasm-target
	@echo "$(CYAN)Checking all examples...$(RESET)"
	@for dir in examples/*/; do \
		name=$$(basename "$$dir"); \
		if [ -f "$$dir/Cargo.toml" ]; then \
			echo "$(CYAN)Checking $$name (SSR)...$(RESET)"; \
			cargo check -p "$$name" --features ssr || exit 1; \
			echo "$(CYAN)Checking $$name (hydrate)...$(RESET)"; \
			cargo check -p "$$name" --features hydrate --target wasm32-unknown-unknown || exit 1; \
		fi \
	done
	@echo "$(GREEN)All examples compile!$(RESET)"

## Clean all examples' build artifacts
clean-all-examples:
	@echo "$(CYAN)Cleaning all example build artifacts...$(RESET)"
	@for dir in examples/*/; do \
		name=$$(basename "$$dir"); \
		rm -rf "$$dir/dist"; \
		rm -rf "$$dir/target/site"; \
	done
	@echo "$(GREEN)All example artifacts cleaned!$(RESET)"

# ============================================================================
# Legacy Example Commands (auth-store-example specific)
# ============================================================================
# These commands are kept for backwards compatibility.
# Consider using the generic commands above instead.
# ============================================================================

## Run the auth-store-example (SSR mode) - LEGACY
example: example-setup leptos-install
	@echo "$(CYAN)Starting auth-store-example (SSR mode)...$(RESET)"
	@echo "$(YELLOW)Open http://127.0.0.1:3000 in your browser$(RESET)"
	cd examples/auth-store-example && cargo leptos watch

## Run the auth-store-example in SSR mode (manual - without cargo-leptos)
example-ssr: example-setup example-ssr-build-wasm
	@echo "$(CYAN)Starting server...$(RESET)"
	@echo "$(YELLOW)Open http://127.0.0.1:3000 in your browser$(RESET)"
	cd examples/auth-store-example && cargo run --features ssr

## Build WASM and assets for manual SSR mode
## Note: Requires wasm-bindgen-cli version to match the wasm-bindgen crate (0.2.100)
example-ssr-build-wasm:
	@echo "$(CYAN)Building WASM for hydration...$(RESET)"
	cd examples/auth-store-example && cargo build --lib --features hydrate --target wasm32-unknown-unknown --release
	@echo "$(CYAN)Running wasm-bindgen (requires v0.2.100)...$(RESET)"
	@command -v wasm-bindgen >/dev/null 2>&1 || cargo install wasm-bindgen-cli --version 0.2.100
	@mkdir -p examples/auth-store-example/target/site/pkg
	wasm-bindgen \
		--target web \
		--out-dir examples/auth-store-example/target/site/pkg \
		--out-name auth-store-example \
		target/wasm32-unknown-unknown/release/auth_store_example.wasm
	@echo "$(CYAN)Copying CSS...$(RESET)"
	cp examples/auth-store-example/style/main.css examples/auth-store-example/target/site/pkg/auth-store-example.css
	@echo "$(GREEN)Assets ready in target/site/pkg/$(RESET)"

## Run the auth-store-example in CSR mode (with trunk)
example-csr: example-setup trunk-install
	@echo "$(CYAN)Starting auth-store-example (CSR mode)...$(RESET)"
	@echo "$(YELLOW)Open http://localhost:8080 in your browser$(RESET)"
	cd examples/auth-store-example && trunk serve --features csr

## Run the auth-store-example on a specific port (CSR mode)
example-port: example-setup trunk-install
	@echo "$(CYAN)Starting auth-store-example on port $(PORT)...$(RESET)"
	cd examples/auth-store-example && trunk serve --features csr --port $(PORT)

## Build the auth-store-example for production (SSR)
example-build: example-setup leptos-install
	@echo "$(CYAN)Building auth-store-example (SSR)...$(RESET)"
	cd examples/auth-store-example && cargo leptos build

## Build the auth-store-example for production (SSR, optimized)
example-release: example-setup leptos-install
	@echo "$(CYAN)Building auth-store-example (SSR, release)...$(RESET)"
	cd examples/auth-store-example && cargo leptos build --release

## Build the auth-store-example for CSR
example-build-csr: example-setup trunk-install
	@echo "$(CYAN)Building auth-store-example (CSR)...$(RESET)"
	cd examples/auth-store-example && trunk build --features csr

## Build the auth-store-example for CSR (optimized)
example-release-csr: example-setup trunk-install
	@echo "$(CYAN)Building auth-store-example (CSR, release)...$(RESET)"
	cd examples/auth-store-example && trunk build --release --features csr

## Clean auth-store-example build artifacts
example-clean:
	@echo "$(CYAN)Cleaning example build artifacts...$(RESET)"
	rm -rf examples/auth-store-example/dist
	rm -rf examples/auth-store-example/target/site

#==============================================================================
# VERSION MANAGEMENT
#==============================================================================

version: ## Show current version
	@./scripts/get-version.sh

bump: ## Auto-bump version based on commits
	@./scripts/bump-version.sh auto

bump-major: ## Bump major version
	@./scripts/bump-version.sh major

bump-minor: ## Bump minor version
	@./scripts/bump-version.sh minor

bump-patch: ## Bump patch version
	@./scripts/bump-version.sh patch

changelog: ## Generate changelog from commits
	@./scripts/changelog.sh

#==============================================================================
# RELEASE
#==============================================================================

release: check ## Full release: auto-bump, tag, push, publish
	@echo "$(BLUE)Starting release process...$(RESET)"
	@./scripts/release.sh auto

release-major: check ## Release with major version bump
	@echo "$(BLUE)Starting major release...$(RESET)"
	@./scripts/release.sh major

release-minor: check ## Release with minor version bump
	@echo "$(BLUE)Starting minor release...$(RESET)"
	@./scripts/release.sh minor

release-patch: check ## Release with patch version bump
	@echo "$(BLUE)Starting patch release...$(RESET)"
	@./scripts/release.sh patch

release-dry-run: ## Preview release without making changes
	@echo "$(BLUE)Release dry run...$(RESET)"
	@./scripts/release.sh --dry-run

release-local: check ## Release locally (no push, no publish)
	@echo "$(BLUE)Local release...$(RESET)"
	@./scripts/release.sh --no-push --skip-publish

#==============================================================================
# PUBLISHING
#==============================================================================

publish: ## Publish to crates.io
	@echo "$(BLUE)Publishing to crates.io...$(RESET)"
	@./scripts/publish.sh

publish-dry: ## Dry run publish to crates.io
	@echo "$(BLUE)Publish dry run...$(RESET)"
	@./scripts/publish.sh --dry-run

#==============================================================================
# UTILITIES
#==============================================================================

setup: ## Setup development environment
	@echo "$(BLUE)Setting up development environment...$(RESET)"
	@chmod +x scripts/*.sh
	@rustup component add rustfmt clippy
	@echo "$(GREEN)Setup complete!$(RESET)"

ci: ## Run CI checks (used in GitHub Actions)
	@echo "$(BLUE)Running CI checks...$(RESET)"
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test
	cargo doc --no-deps
	@echo "$(GREEN)CI checks passed!$(RESET)"

## Create a new version tag
tag:
	@echo "$(CYAN)Creating version tag...$(RESET)"
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'); \
	echo "Creating tag v$$VERSION"; \
	git tag -a "v$$VERSION" -m "Release v$$VERSION"

## Push tags to remote
tag-push:
	@echo "$(CYAN)Pushing tags to remote...$(RESET)"
	git push --tags

# ============================================================================
# Dependencies
# ============================================================================

## Update dependencies
deps-update:
	@echo "$(CYAN)Updating dependencies...$(RESET)"
	cargo update

## Check for outdated dependencies (requires cargo-outdated)
outdated:
	@echo "$(CYAN)Checking for outdated dependencies...$(RESET)"
	cargo outdated --workspace

## Audit dependencies for security vulnerabilities (requires cargo-audit)
audit:
	@echo "$(CYAN)Auditing dependencies...$(RESET)"
	cargo audit

## Show dependency tree
deps-tree:
	@echo "$(CYAN)Showing dependency tree...$(RESET)"
	cargo tree

## Show dependency tree for a specific package
deps-tree-pkg:
	@echo "$(CYAN)Showing dependency tree for $(PKG)...$(RESET)"
	cargo tree -p $(PKG)

# ============================================================================
# Utilities
# ============================================================================

## Clean all build artifacts
clean:
	@echo "$(CYAN)Cleaning build artifacts...$(RESET)"
	cargo clean
	rm -rf examples/auth-store-example/dist

## Watch for changes and run tests
watch:
	@echo "$(CYAN)Watching for changes...$(RESET)"
	cargo watch -x test

## Watch for changes and run clippy
watch-clippy:
	@echo "$(CYAN)Watching for changes (clippy)...$(RESET)"
	cargo watch -x clippy

## Generate a new store template
new-store:
	@echo "$(CYAN)Store template:$(RESET)"
	@echo ""
	@echo "use leptos::prelude::*;"
	@echo "use leptos_store::prelude::*;"
	@echo ""
	@echo "#[derive(Clone, Debug, Default)]"
	@echo "pub struct MyState {"
	@echo "    // Add your state fields here"
	@echo "}"
	@echo ""
	@echo "#[derive(Clone)]"
	@echo "pub struct MyStore {"
	@echo "    state: RwSignal<MyState>,"
	@echo "}"
	@echo ""
	@echo "impl MyStore {"
	@echo "    pub fn new() -> Self {"
	@echo "        Self {"
	@echo "            state: RwSignal::new(MyState::default()),"
	@echo "        }"
	@echo "    }"
	@echo ""
	@echo "    // Add getters here"
	@echo ""
	@echo "    // Add mutators here"
	@echo "}"
	@echo ""
	@echo "impl Store for MyStore {"
	@echo "    type State = MyState;"
	@echo ""
	@echo "    fn state(&self) -> ReadSignal<Self::State> {"
	@echo "        self.state.read_only()"
	@echo "    }"
	@echo "}"

## Show lines of code statistics
loc:
	@echo "$(CYAN)Lines of code:$(RESET)"
	@find src -name "*.rs" | xargs wc -l | tail -1
	@echo ""
	@echo "$(CYAN)By file:$(RESET)"
	@find src -name "*.rs" | xargs wc -l | sort -n

# ============================================================================
# Help
# ============================================================================

## Show this help message
help:
	@echo "$(CYAN)leptos-store Makefile$(RESET)"
	@echo ""
	@echo "$(GREEN)Usage:$(RESET)"
	@echo "  make [target]"
	@echo "  make [target] NAME=<example-name>  # For example-specific commands"
	@echo ""
	@echo "$(GREEN)Building:$(RESET)"
	@echo "  build          Build the library in debug mode"
	@echo "  build-release  Build the library in release mode"
	@echo "  check          Check the library compiles"
	@echo "  check-features Check all feature combinations"
	@echo ""
	@echo "$(GREEN)Testing:$(RESET)"
	@echo "  test           Run all tests"
	@echo "  test-verbose   Run tests with output"
	@echo "  test-lib       Run only library tests"
	@echo "  test-doc       Run only doc tests"
	@echo "  test-example   Run auth-store-example tests"
	@echo "  test-all       Run all tests with all features"
	@echo "  coverage       Run tests with coverage"
	@echo ""
	@echo "$(GREEN)Code Quality:$(RESET)"
	@echo "  clippy         Run clippy lints"
	@echo "  clippy-fix     Run clippy with auto-fixes"
	@echo "  fmt            Format code"
	@echo "  fmt-check      Check code formatting"
	@echo "  ci             Run all CI checks"
	@echo ""
	@echo "$(GREEN)Documentation:$(RESET)"
	@echo "  doc            Build documentation"
	@echo "  doc-open       Build and open documentation"
	@echo "  doc-private    Build docs with private items"
	@echo ""
	@echo "$(GREEN)Showcase & Docker:$(RESET)"
	@echo "  showcase         Run showcase (dev mode with CSS gen)"
	@echo "  showcase-css     Generate scoped CSS for showcase"
	@echo "  showcase-dev     Alias for showcase"
	@echo "  showcase-build   Build showcase for production (release)"
	@echo "  docker-build     Build Docker image"
	@echo "  docker-run       Run Docker container on :8080"
	@echo "  docker           Build + run Docker container"
	@echo ""
	@echo "$(GREEN)Examples (Generic - use NAME=<example>):$(RESET)"
	@echo "  examples-list        List all available examples"
	@echo "  run                  Run example (make run NAME=token-explorer-example)"
	@echo "  build-example        Build example (make build-example NAME=...)"
	@echo "  build-example-release Build example in release mode"
	@echo "  test-example-pkg     Run example tests (make test-example-pkg NAME=...)"
	@echo "  check-example        Check example compiles (SSR + hydrate)"
	@echo "  clean-example        Clean example artifacts"
	@echo "  test-all-examples    Test all examples"
	@echo "  check-all-examples   Check all examples compile"
	@echo "  clean-all-examples   Clean all example artifacts"
	@echo ""
	@echo "$(GREEN)Examples (Legacy - auth-store-example):$(RESET)"
	@echo "  example          Run auth-store-example (SSR)"
	@echo "  example-ssr      Run auth-store-example (SSR manual)"
	@echo "  example-csr      Run auth-store-example (CSR)"
	@echo "  example-build    Build auth-store-example (SSR)"
	@echo "  example-release  Build auth-store-example (SSR, release)"
	@echo "  example-clean    Clean auth-store-example artifacts"
	@echo ""
	@echo "$(GREEN)Publishing:$(RESET)"
	@echo "  publish-dry    Dry run publish"
	@echo "  publish        Publish to crates.io"
	@echo "  tag            Create version tag"
	@echo "  tag-push       Push tags to remote"
	@echo ""
	@echo "$(GREEN)Dependencies:$(RESET)"
	@echo "  deps-update    Update dependencies"
	@echo "  outdated       Check for outdated deps"
	@echo "  audit          Audit for vulnerabilities"
	@echo "  deps-tree      Show dependency tree"
	@echo ""
	@echo "$(GREEN)Utilities:$(RESET)"
	@echo "  clean          Clean all build artifacts"
	@echo "  watch          Watch and run tests"
	@echo "  watch-clippy   Watch and run clippy"
	@echo "  new-store      Print store template"
	@echo "  loc            Show lines of code"
	@echo "  help           Show this help"
