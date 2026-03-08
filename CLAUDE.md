# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A Rust port of the [Microsoft Commercial Marketplace SaaS Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator). It provides publisher (Admin) and customer portals for managing Azure Marketplace SaaS subscriptions, with a React frontend.

## Build & Test Commands

```bash
# Build
cargo build --workspace            # debug build
cargo build --workspace --release  # release build

# Test
cargo test --workspace             # all tests
cargo test -p shared               # single crate
cargo test -p admin-api            # single crate
cargo test --workspace -- test_name # single test by name

# Lint
cargo fmt --all                    # format
cargo clippy --workspace --all-targets -- -D warnings  # lint (CI enforces -D warnings)

# Run services (each needs DATABASE_URL in .env)
cargo run -p admin-api             # port 3000
cargo run -p customer-api          # port 3001
cargo run -p webhook-api           # port 3002
cargo run -p scheduler             # background metered billing

# Database migrations
cd crates/data && sqlx migrate run

# Frontend
cd frontend && npm install && npm run dev   # dev server at :5173
cd frontend && npm run build                # production build

# Docker (starts Postgres + migrations + both APIs)
docker compose up -d
```

## Architecture

**Two-site deployment** mirroring the original .NET accelerator:

| Site               | Crate          | Port | Purpose                                                                                                              |
| ------------------ | -------------- | ---- | -------------------------------------------------------------------------------------------------------------------- |
| Admin              | `admin-api`    | 3000 | Publisher portal - manage subscriptions, plans, offers, config. Protected by Azure AD + Known Users when configured. |
| Customer           | `customer-api` | 3001 | Customer landing page, subscription management, **and** embedded webhook handler at `POST /api/webhook`.             |
| Webhook (optional) | `webhook-api`  | 3002 | Standalone webhook if not using customer-api's embedded one.                                                         |
| Scheduler          | `scheduler`    | —    | Background job for metered billing usage emission.                                                                   |

### Workspace Crate Dependency Graph

```
admin-api ──┐
customer-api┤──→ shared ──→ (config, models, errors, auth, services)
webhook-api ┤──→ data ────→ (PostgreSQL repositories via sqlx)
scheduler   ┘──→ marketplace → (FulfillmentApiClient, MeteringApiClient)
                  webhook ───→ (shared webhook state, used by customer-api and webhook-api)
```

### Key Patterns

- **Web framework**: Axum 0.7 with Tower middleware (CORS, tracing, sessions).
- **Repository pattern**: Each entity has a trait (e.g. `SubscriptionRepository`) and a Postgres implementation (e.g. `PostgresSubscriptionRepository`). All repositories are in `crates/data/src/repositories/`, injected as `Arc<dyn Trait>` into `AppState`.
- **Adapter pattern in webhook crate**: `crates/webhook/src/adapters.rs` bridges `data` repository traits to `shared` service traits (e.g. `SubscriptionRepositoryAdapter` implements `SubscriptionRepositoryTrait` by delegating to `SubscriptionRepository`). This decouples the service layer from the data layer.
- **Service layer**: `crates/shared/src/services/` contains business logic (subscription lifecycle, status handlers, email, webhook processing). Services depend on trait abstractions, not concrete repos.
- **Status handler chain**: Webhook actions flow through `AbstractSubscriptionStatusHandler` → `NotificationStatusHandler` → email notifications. Status handlers for each action type (pending activation, pending fulfillment, unsubscribe) are in separate files.
- **Auth**: Admin API uses Azure AD OAuth2 via `openidconnect`/`oauth2` crates with session-based auth (`tower-sessions`). Webhook endpoints validate Azure Marketplace JWT tokens via `JwtValidator` (OIDC discovery + JWKS).
- **Error handling**: `AcceleratorError` enum in `shared::errors` with `thiserror`. API handlers return `StatusCode` or `(StatusCode, Json<_>)`.
- **Config**: Environment variables loaded via `dotenv`. Marketplace config uses `SaaS_API_` prefix. Azure AD uses `AZURE_AD_` prefix.

### Frontend

React 18 + TypeScript + Vite SPA in `frontend/`. Uses React Router for routing, Axios for API calls, TanStack React Query for data fetching.

- Customer pages: `frontend/src/pages/customer/` (Landing, Subscriptions, SubscriptionDetail)
- Admin pages: `frontend/src/pages/admin/` (Dashboard, Subscriptions, Plans, Offers, Config, KnownUsers, Scheduler, EmailTemplates, ApplicationLog)
- API client: `frontend/src/api/client.ts`
- Layouts: `AdminLayout.tsx` and `CustomerLayout.tsx`

### Database

PostgreSQL via `sqlx` (compile-time query checking disabled; runtime queries). Migrations in `crates/data/migrations/`. Connection pool created via `data::pool::create_pool`.

## CI/CD

Azure Pipelines (`azure-pipelines.yml`): Build stage runs `cargo build --release`, `cargo test`, `cargo clippy` and frontend `npm ci && npm run build` in parallel. Deploy stage builds Docker images and pushes to ACR, deploying to two Azure Web Apps (Admin + Customer).

## Environment Variables

Required: `DATABASE_URL`

Marketplace API: `SaaS_API_TENANT_ID`, `SaaS_API_CLIENT_ID`, `SaaS_API_CLIENT_SECRET`, `SaaS_API_RESOURCE`

Admin Azure AD (optional for local dev): `AZURE_AD_TENANT_ID`, `AZURE_AD_CLIENT_ID`, `AZURE_AD_CLIENT_SECRET`, `AZURE_AD_REDIRECT_URI`, `AZURE_AD_SIGNED_OUT_REDIRECT_URI`

Rust edition is 2024; minimum Rust version 1.70+.
