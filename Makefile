.PHONY: setup build test fmt lint clean watch

setup:
	./scripts/setup_dev.sh

build:
	cargo build

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

watch:
	cargo watch -x check -x test -x run

# Development workflow
dev: fmt lint test build

# Security checks
security:
	cargo audit
	cargo outdated

# Documentation
docs:
	cargo doc --no-deps --document-private-items 