# Multi-stage Dockerfile for SaaS Accelerator Rust Edition

# Build stage (slim-trixie so glibc matches debian:trixie-slim runtime)
FROM rust:1.93.1-slim-trixie AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the admin-api
WORKDIR /app/crates/admin-api
RUN cargo build --release --bin admin-api

# Build the customer-api
WORKDIR /app/crates/customer-api
RUN cargo build --release --bin customer-api

# Build the webhook-api
WORKDIR /app/crates/webhook-api
RUN cargo build --release --bin webhook-api

# Runtime stage (trixie-slim so glibc matches rust:1.93.1-slim-trixie builder)
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/admin-api /app/admin-api
COPY --from=builder /app/target/release/customer-api /app/customer-api
COPY --from=builder /app/target/release/webhook-api /app/webhook-api

# Expose ports
EXPOSE 3000 3001 3002

# Default to admin-api (can be overridden)
CMD ["./admin-api"]

