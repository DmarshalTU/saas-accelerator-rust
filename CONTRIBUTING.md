# Contributing to SaaS Accelerator Rust Edition

Thank you for your interest in contributing to the SaaS Accelerator Rust Edition!

## Development Setup

1. **Install Prerequisites**
   - Rust 1.70+ (install via [rustup](https://rustup.rs/))
   - PostgreSQL 14+ (or Azure Database for PostgreSQL)
   - Azure CLI (for deployment)
   - Docker (optional, for containerized deployment)

2. **Clone the Repository**
   ```bash
   git clone git@github.com:DmarshalTU/Commercial-Marketplace-SaaS-Accelerator-Rust-Edition.git
   cd Commercial-Marketplace-SaaS-Accelerator-Rust-Edition
   ```

3. **Set Up Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Build the Project**
   ```bash
   cargo build
   ```

5. **Run Database Migrations**
   ```bash
   export DATABASE_URL="postgresql://user:password@localhost:5432/saas_accelerator"
   cd crates/data
   sqlx migrate run
   ```

6. **Run the Services**
   ```bash
   # Admin API
   cargo run --bin admin-api

   # Customer API
   cargo run --bin customer-api

   # Webhook API
   cargo run --bin webhook-api
   ```

## Project Structure

```
saas-accelerator-rust/
├── crates/
│   ├── admin-api/          # Admin/Publisher portal API
│   ├── customer-api/       # Customer portal API
│   ├── webhook-api/        # Webhook handler
│   ├── scheduler/          # Background job service
│   ├── shared/             # Shared models and utilities
│   ├── data/               # Database access layer
│   └── marketplace/        # Marketplace API clients
├── deployment/             # Deployment scripts
└── frontend/               # Frontend SPA (to be added)
```

## Code Style

- Follow Rust conventions and best practices
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for linting issues
- Write tests for new features

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p shared
```

## Submitting Changes

1. Create a feature branch from `main`
2. Make your changes
3. Ensure tests pass
4. Submit a pull request with a clear description

## Current Status

- Project structure and workspace setup - Complete
- Marketplace API clients (Fulfillment & Metering) - Complete
- Database layer with SQLx - Complete
- API server skeletons - Complete
- Deployment scripts - Complete
- Authentication middleware - In Progress
- Background scheduler - Pending
- Frontend SPA - Pending

## Questions?

Open an issue on GitHub for questions or discussions.

