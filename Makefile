# SaaS Accelerator Rust – build and test
.PHONY: build test check run-admin run-customer run-webhook run-scheduler clean

# Build the whole workspace
build:
	cargo build --workspace

# Build release
build-release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Build and test (CI-style)
check: build test

# Run services (require DATABASE_URL and env; use separate terminals or docker-compose)
run-admin:
	cargo run -p admin-api

run-customer:
	cargo run -p customer-api

run-webhook:
	cargo run -p webhook-api

run-scheduler:
	cargo run -p scheduler

clean:
	cargo clean
