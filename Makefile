# Root project Makefile - orchestrates all components

.PHONY: all test build clean dev format lint

# Default target - build everything
all: build

# Run all tests
test:
	cd core && cargo test
	cd bindings/javascript/wrapper && npm test

# Build everything
build:
	cd bindings/javascript/wrapper && npm run build

# Clean everything
clean:
	cd core && cargo clean
	cd bindings/javascript/wrapper && npm run clean

# Format all code
format:
	cd core && cargo fmt
	cd bindings/javascript/wrapper && npm run format

# Lint all code
lint:
	cd core && cargo clippy -- -D warnings

# Development workflow - format, lint, test, then build
dev: format lint test build
	@echo "All checks passed and build completed successfully!"