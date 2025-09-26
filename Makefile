# Root project Makefile - orchestrates all components

.PHONY: all test build clean core-test js-test core-build js-build

# Default target - build everything
all: build

# Run all tests
test: core-test js-test

# Build everything
build: core-build js-build

# Core C tests
core-test:
	cd core && make test

# JavaScript binding tests (requires WASM to be built first)
js-test: js-build
	cd bindings/javascript && npm test

# Build core C binary
core-build:
	cd core && make

# Build JavaScript WASM bindings
js-build:
	cd bindings/javascript && make

# Clean everything
clean:
	cd core && make clean
	cd bindings/javascript && make clean

# Development workflow - test everything then build
dev: test build
	@echo "All tests passed and build completed successfully!"