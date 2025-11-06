# SaaS Accelerator - Rust Edition

A modern, high-performance port of the Microsoft Commercial Marketplace SaaS Accelerator to Rust, providing better developer experience, hot reload capabilities, and cross-platform deployment.

inspired by [Commercial-Marketplace-SaaS-Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator)

## Architecture

This project is organized as a Cargo workspace with the following components:

### Backend (Rust)
- **`crates/admin-api`** - Admin/Publisher portal REST API (Port 3000)
- **`crates/customer-api`** - Customer portal REST API (Port 3001)
- **`crates/webhook-api`** - Webhook handler for marketplace events (Port 3002)
- **`crates/scheduler`** - Background job service for metered billing
- **`crates/shared`** - Shared business logic and utilities
- **`crates/data`** - Database access layer (SQLx + PostgreSQL)
- **`crates/marketplace`** - Marketplace API clients (Fulfillment & Metering)

### Frontend (React + TypeScript)
- **`frontend/`** - React SPA with Vite for hot reload (Port 5173)
  - Admin Portal (`/admin/*`)
  - Customer Portal (`/*`)

## Features

- 🚀 **High Performance** - Rust's zero-cost abstractions and memory safety
- 🔥 **Hot Reload** - Frontend hot reload with modern SPA framework
- 🔐 **Azure Integration** - Uses official Azure SDK for Rust
- 🗄️ **PostgreSQL** - Better Linux support than SQL Server
- 🌐 **Cross-Platform** - Deploy to Linux, Windows, or macOS
- 📦 **Modular** - Separate frontend/backend for team collaboration

## Prerequisites

- Rust 1.70+ (`rustup install stable`)
- PostgreSQL 14+ (or Azure Database for PostgreSQL)
- Azure CLI (for deployment)
- Node.js 18+ (for frontend development)

## Quick Start

### Backend Development

```bash
# Install dependencies
cargo build

# Run database migrations
cd crates/data
sqlx migrate run

# Run admin API
cargo run --bin admin-api

# Run customer API  
cargo run --bin customer-api

# Run webhook API
cargo run --bin webhook-api

# Run scheduler
cargo run --bin scheduler
```

### Frontend Development

```bash
cd frontend
npm install
npm run dev  # Hot reload enabled at http://localhost:5173
```

### Setup Script

For first-time setup, run:

```bash
./scripts/setup.sh
```

This will:
- Check prerequisites (Rust, PostgreSQL, Node.js)
- Build the Rust workspace
- Install frontend dependencies
- Create `.env` file from template

## Configuration

Create a `.env` file in the root directory:

```env
DATABASE_URL=postgresql://user:password@localhost:5432/saas_accelerator
AZURE_TENANT_ID=your-tenant-id
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api
```

## Deployment

See `deployment/` directory for deployment scripts:
- `deploy.sh` - Linux/Bash deployment script
- `deploy.ps1` - Windows/PowerShell deployment script

Both scripts support deployment to Azure Web Apps.

## License

MIT License - See LICENSE file

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines.

