# Root project Makefile - orchestrates all components

.PHONY: all test build clean dev

# Default target - build everything
all: build

# Run all tests
test:
	cd core && make test
	cd bindings/javascript && make
	cd bindings/javascript && npm test

# Build everything
build:
	cd core && make
	cd bindings/javascript && make

# Clean everything
clean:
	cd core && make clean
	cd bindings/javascript && make clean

# Development workflow - test everything then build
dev: test build
	@echo "All tests passed and build completed successfully!"