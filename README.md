# SaaS Accelerator - Rust Edition

A modern, high-performance port of the Microsoft Commercial Marketplace SaaS Accelerator to Rust, providing better developer experience, hot reload capabilities, and cross-platform deployment.

inspired by [Commercial-Marketplace-SaaS-Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator)

## Architecture: two deployable sites (like the original)

Deployment matches the **original SaaS Accelerator**: **Admin** and **Customer** are separate deployables for security, cost, and isolation. See [docs/TWO_SITE_DEPLOYMENT.md](docs/TWO_SITE_DEPLOYMENT.md).

| Site | Contents | Purpose |
|------|----------|---------|
| **Admin** | admin-api + Admin UI | Publisher portal (subscriptions, plans, offers, config). One Azure Web App / image. |
| **Customer** | customer-api + webhook + Customer UI | Landing page, customer subscriptions, **and** webhook (same app as in original). One Azure Web App / image. |

### Workspace crates

- **`crates/admin-api`** – Admin site backend (Port 3000). Serves Admin UI when deployed.
- **`crates/customer-api`** – Customer site backend (Port 3001). Includes webhook at `POST /api/webhook`. Serves Customer UI when deployed.
- **`crates/webhook-api`** – Optional standalone webhook server (Port 3002). For two-site deploy, webhook runs inside customer-api.
- **`crates/scheduler`** – Background job for metered billing
- **`crates/shared`**, **`crates/data`**, **`crates/marketplace`** – Shared logic, DB, marketplace clients

### Frontend (React + TypeScript) — **this is the UI**

- **`frontend/`** – The web UI (React + Vite). Run it with:
  ```bash
  cd frontend && npm install && npm run dev
  ```
  Then open **http://localhost:5173** (Customer) or **http://localhost:5173/admin** (Admin). Start the APIs first (e.g. `docker compose up -d` or run the binaries).  
  For **how this deploys to Azure** (one frontend project, two Web Apps), see [docs/AZURE_DEPLOYMENT.md](docs/AZURE_DEPLOYMENT.md).

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

### Docker Compose (backend + UI)

```bash
docker compose up -d
```

Starts PostgreSQL (with migrations), Admin API (3000), Customer API (3001), optional Webhook API (3002), and the **UI** (5173). Default DB: user `saas`, password `saas`, database `saas_accelerator`. See [docs/TESTING_IN_ACTION.md](docs/TESTING_IN_ACTION.md) for details and optional `.env` overrides.

**Note:** Open **http://localhost:5173** for the React app (Customer at `/`, Admin at `/admin`). Ports 3000 and 3001 serve the REST APIs (and a small root info page). For the **full Admin/Customer web UIs** (like the .NET “sites”), use Frontend Development for hot reload.

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

## Testing

Build and run all tests:

```bash
# Using Make
make build    # Build the whole workspace
make test     # Run all tests
make check    # Build then test

# Or using Cargo directly
cargo build --workspace
cargo test --workspace
```

Tests include:
- **shared**: Model serialization (WebhookAction, TermUnitEnum, SubscriptionStatus), URL validation
- **admin-api**, **customer-api**, **webhook-api**: Health endpoint (router tests)

See **PORTING_CHECKLIST.md** for a feature-by-feature comparison with the original Microsoft SaaS Accelerator and how to verify the port.

## Test in action (run the APIs locally)

To run the services and hit them with real requests:

1. **Database**: Create a PostgreSQL database, set `DATABASE_URL` in `.env`, then run migrations:
   ```bash
   cd crates/data && sqlx migrate run && cd ../..
   ```
2. **Start the APIs** (each in its own terminal):
   ```bash
   cargo run -p admin-api      # port 3000
   cargo run -p customer-api  # port 3001
   cargo run -p webhook-api   # port 3002
   ```
3. **Smoke test** (with all three running):
   ```bash
   ./scripts/smoke-test.sh
   ```
4. **Try endpoints**: e.g. `curl http://localhost:3000/health`, `curl http://localhost:3001/api/landing`, etc.

Full steps and curl examples: **[docs/TESTING_IN_ACTION.md](docs/TESTING_IN_ACTION.md)**.

## Configuration

Create a `.env` file in the root directory:

```env
DATABASE_URL=postgresql://user:password@localhost:5432/saas_accelerator
AZURE_TENANT_ID=your-tenant-id
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api
```

## Deployment to Azure

Deployment follows the **same model as the [original SaaS Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator)**: two separate Azure Web Apps (Admin and Customer), one shared database.

### Security when deployed

| Question | Answer |
|----------|--------|
| **Is Admin secured?** | **Yes** when Azure AD is configured. Admin API requires login (session) and, when Azure AD is set, the user must be in **Known Users** (admin role). Otherwise 401/403. |
| **Can customers log into Admin by accident?** | **No.** Admin and Customer are **different URLs** (different Web Apps). Customers use only the customer URL. |
| **Docker / local without Azure AD?** | Admin API is **open** for development. Set Azure AD env vars and add Known Users to secure it. |

### Steps (high level)

1. **Two Azure Web Apps** – One for Admin (admin-api + UI), one for Customer (customer-api + webhook + UI). Same frontend `dist/` deployed to both; each app uses its own API base URL.
2. **Database** – One Azure PostgreSQL (or compatible) instance; same connection string for both apps.
3. **Admin app settings** – Set `DATABASE_URL`, Marketplace API vars (`SaaS_API_TENANT_ID`, `SaaS_API_CLIENT_ID`, `SaaS_API_CLIENT_SECRET`), and **Azure AD** vars: `AZURE_AD_TENANT_ID`, `AZURE_AD_CLIENT_ID`, `AZURE_AD_CLIENT_SECRET`, `AZURE_AD_REDIRECT_URI` (e.g. `https://<admin-host>/auth/callback`), `AZURE_AD_SIGNED_OUT_REDIRECT_URI`.
4. **Customer app settings** – Set `DATABASE_URL` and Marketplace API vars. No Azure AD needed.
5. **Build frontend once** with production API URLs, then deploy the same `dist/` to both Web Apps:
   ```bash
   cd frontend
   export VITE_ADMIN_API_URL=https://<your-admin-host>
   export VITE_CUSTOMER_API_URL=https://<your-customer-host>
   npm ci && npm run build
   ```
6. **Known Users** – After first deploy, add publisher emails to Known Users (Admin UI) with admin role so they can access the admin API.

**Full details:** [docs/AZURE_DEPLOYMENT.md](docs/AZURE_DEPLOYMENT.md) (routing, static files, security checklist). The original .NET accelerator uses [Deploy.ps1](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator/blob/main/deployment/Deploy.ps1) and [Installation-Instructions.md](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator/blob/main/docs/Installation-Instructions.md); this port uses the same two-Web-App, shared-DB design; you can use your own CI/CD or the scripts in `deployment/` to deploy the Rust binaries and frontend.

## License

MIT License - See LICENSE file

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines.

