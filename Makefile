# Default target
.PHONY: all
all: check-fmt check-clippy check-udeps test

# -------------------------
# Format checking
# -------------------------
.PHONY: check-fmt
check-fmt: ## Check Rust formatting
	cargo fmt --all -- --check

# -------------------------
# Clippy linting
# -------------------------
.PHONY: check-clippy
check-clippy: ## Run Clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

# -------------------------
# Unused dependencies check
# -------------------------
.PHONY: check-udeps
check-udeps: ## Check for unused dependencies
	cargo +nightly udeps --workspace --all-targets

# -------------------------
# Build project
# -------------------------
.PHONY: build
build: ## Build Rust project
	cargo build --all

# -------------------------
# Run tests
# -------------------------
.PHONY: test
test: ## Run all tests
	cargo test --all

# -------------------------
# Clean project
# -------------------------
.PHONY: clean
clean: ## Remove target directory
	cargo clean