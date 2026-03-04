# Testing the SaaS Accelerator in Action

This guide walks you through running the APIs locally and hitting them with real requests.

## Option A: Docker Compose (all-in-one)

From the repo root:

```bash
cd saas-accelerator-rust
docker compose up -d
```

This starts **PostgreSQL** (port 5432), runs **migrations**, then **Admin API** (3000), **Customer API** (3001), and **Webhook API** (3002). Default DB credentials: `saas` / `saas` / database `saas_accelerator`.

Smoke test:

```bash
./scripts/smoke-test.sh
```

To use your own DB credentials or Azure/Marketplace env vars, create a `.env` file (see **Option B** for variable names). Then run `docker compose up -d` again.

To stop: `docker compose down`. Data is kept in the `postgres_data` volume.

**If you see "pull access denied for saas-accelerator-api"**: the image is built locally (not pulled). Run `docker compose build` once, then `docker compose up -d`, or ensure all three API services have the same `build` block so Compose builds the image before starting.

**If only Postgres is running** (admin/customer/webhook containers exit or never start):

1. **Migrate must succeed first** – API services depend on `migrate` completing. Check `docker compose logs migrate`. If migrations failed (e.g. schema error), fix the migration, run `docker compose build migrate`, then `docker compose down -v` and `docker compose up -d` to get a fresh DB.
2. **Admin API** – It no longer requires Azure AD to start. If `AZURE_AD_*` env vars are unset or empty, the server still runs and serves `/health` and API routes; auth routes return 503 until you set Azure AD credentials.

---

## Option B: Run APIs locally (with or without Docker for Postgres)

### 1. Prerequisites

- **Rust**: `rustup install stable`
- **PostgreSQL 14+**: running locally or reachable (e.g. Docker)
- **Environment**: `.env` in the workspace root with at least `DATABASE_URL`

### 2. One-time setup

### Database

Create a database and run migrations:

```bash
# From repo root; DATABASE_URL must be set (e.g. in .env)
cd saas-accelerator-rust
source .env   # or export DATABASE_URL=postgresql://user:pass@localhost:5432/saas_db

cd crates/data
sqlx migrate run
cd ../..
```

Example `DATABASE_URL`: `postgresql://postgres:postgres@localhost:5432/saas_accelerator`

### Optional: minimal .env for local testing

Admin/Customer/Webhook APIs need `DATABASE_URL`. They can start with dummy Azure/Marketplace vars; only routes that call Marketplace will fail.

```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/saas_accelerator
# Optional for health-only testing; required for real Marketplace calls:
# SaaS_API_TENANT_ID=...
# SaaS_API_CLIENT_ID=...
# SaaS_API_CLIENT_SECRET=...
# AZURE_AD_TENANT_ID=...
# AZURE_AD_CLIENT_ID=...
# AZURE_AD_CLIENT_SECRET=...
```

### 3. Start the services

Use **four terminals** (or run in background).

**Terminal 1 – Admin API (port 3000):**
```bash
cd saas-accelerator-rust
cargo run -p admin-api
# or: make run-admin
```

**Terminal 2 – Customer API (port 3001):**
```bash
cd saas-accelerator-rust
cargo run -p customer-api
# or: make run-customer
```

**Terminal 3 – Webhook API (port 3002):**
```bash
cd saas-accelerator-rust
cargo run -p webhook-api
# or: make run-webhook
```

**Terminal 4 – Scheduler (optional; for metered billing):**
```bash
cd saas-accelerator-rust
cargo run -p scheduler
# or: make run-scheduler
```

### 4. Smoke test (health checks)

With all three APIs running, from another terminal:

```bash
cd saas-accelerator-rust
./scripts/smoke-test.sh
```

Or manually:

```bash
# Admin API
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health
# Expect: 200

# Customer API
curl -s -o /dev/null -w "%{http_code}" http://localhost:3001/health
# Expect: 200

# Webhook API (POST)
curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:3002/health
# Expect: 200
```

### 5. Try the APIs

### Admin API (port 3000)

```bash
# Health
curl -s http://localhost:3000/health

# List subscriptions (returns [] if DB empty or 500 if no Marketplace config)
curl -s http://localhost:3000/api/subscriptions

# List plans (needs plans in DB)
curl -s http://localhost:3000/api/plans

# List offers
curl -s http://localhost:3000/api/offers

# Application config
curl -s http://localhost:3000/api/config
```

### Customer API (port 3001)

```bash
# Health
curl -s http://localhost:3001/health

# Landing page (no token)
curl -s http://localhost:3001/api/landing

# Landing page with marketplace token (resolve subscription)
curl -s "http://localhost:3001/api/landing?token=YOUR_MARKETPLACE_TOKEN"

# User subscriptions by email (needs data in DB)
curl -s http://localhost:3001/api/users/test@example.com/subscriptions

# User by email
curl -s http://localhost:3001/api/users/test@example.com
```

### Webhook API (port 3002)

Requires a valid JWT (Marketplace webhook secret). Without it, you get 401.

```bash
# Health (POST)
curl -s -X POST http://localhost:3002/health

# Webhook (expect 401 without Authorization header)
curl -s -X POST http://localhost:3002/api/webhook \
  -H "Content-Type: application/json" \
  -d '{"activityId":"550e8400-e29b-41d4-a716-446655440000","subscriptionId":"550e8400-e29b-41d4-a716-446655440001","offerId":"offer","timeStamp":"2024-01-15T12:00:00Z","action":"Unsubscribe","status":"Success"}'
```

### 6. Run the automated test suite

Unit and integration tests (no services need to be running):

```bash
cd saas-accelerator-rust
cargo test --workspace
# or: make test
```

### How the original .NET SaaS Accelerator runs locally

The [Commercial-Marketplace-SaaS-Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator) does not use Docker for local dev. Local workflow:

- **Visual Studio**: Open the solution and press **F5** to run (Admin and Customer are separate projects; one solution).
- **Config**: Connection strings and SaaS/Azure config in `appsettings.json` (and `appsettings.Development.json`). Many values are commented out in the sample; you fill them for your tenant.
- **Webhook in dev**: The .NET app can bypass webhook operation verification when `ASPNETCORE_ENVIRONMENT=Development` so you can POST webhook payloads from Postman locally. Never enable that in production.

The Rust port gives you the same capabilities via **Option A** (Docker Compose) or **Option B** (run binaries + Postgres), with PostgreSQL instead of SQL Server.

## Summary

| Step              | Command / action                          |
|-------------------|--------------------------------------------|
| **Docker (all)**  | `docker compose up -d`                     |
| Migrate DB        | `cd crates/data && sqlx migrate run`       |
| Start Admin API   | `cargo run -p admin-api`                   |
| Start Customer API| `cargo run -p customer-api`                |
| Start Webhook API | `cargo run -p webhook-api`                 |
| Smoke test        | `./scripts/smoke-test.sh`                  |
| Unit tests        | `cargo test --workspace`                   |
