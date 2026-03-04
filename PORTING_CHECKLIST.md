# Porting Checklist: Microsoft SaaS Accelerator (.NET) → Rust

This document maps the original [Commercial Marketplace SaaS Accelerator](https://github.com/Azure/commercial-marketplace-saas-accelerator) capabilities to the Rust port so you can verify feature parity and test the system.

## Scope: Two deployable sites (Admin vs Customer)

**Target:** Match the original so that when you deploy, you get **two separate web apps** in Azure (or two images): **Admin** (publisher portal) and **Customer** (landing + webhook). Admin is isolated from Customer for security, cost, and deployment. See [docs/TWO_SITE_DEPLOYMENT.md](docs/TWO_SITE_DEPLOYMENT.md).

In the **original .NET solution**, “Admin Site” and “Customer Site” are full web applications: the same process serves both the **UI** (HTML, Razor/MVC pages) and the **API**. You open one URL and get a complete web app.

In this **Rust port**, the goal is the same **two sites**; implementation is split as follows:

| Layer | Original (.NET) | Rust port |
|-------|----------------------------|-----------|
| **Backend (API)** | Part of Admin/Customer sites | **Rust APIs only**: `admin-api` (3000), `customer-api` (3001), `webhook-api` (3002). These are REST APIs; they do not serve the full UI. |
| **Frontend (UI)** | Served by the same .NET app | **Separate React SPA** in `frontend/`: Admin and Customer portals (Vite, port 5173). It calls the Rust APIs via proxy. |

- **Docker Compose** runs only the **backend** (Postgres + migrate + the three APIs). So `http://localhost:3000` and `http://localhost:3001` are API roots; you get the small info pages we added, not the full Admin/Customer web apps.
- To get the **full web app experience** (like the .NET sites), run the **React frontend** as well:
  ```bash
  cd frontend && npm install && npm run dev
  ```
  Then open **http://localhost:5173** for the Customer portal and **http://localhost:5173/admin** for the Admin portal. The frontend proxies `/api` to the admin-api (3000) and customer-api (3001).

So: the **business logic and APIs** are ported to Rust; the **web UI** is a separate React app that talks to those APIs. Same capabilities, different layout (APIs vs. APIs + SPA).

## APIs and Services

| Original (.NET) | Rust port | Status |
|----------------|-----------|--------|
| **Admin Site** (MVC + API) | **admin-api** (Axum, port 3000) | ✅ Ported |
| **Customer Site** (MVC + API) | **customer-api** (Axum, port 3001) | ✅ Ported |
| **Webhook API** (`api/webhook`) | **webhook-api** (Axum, port 3002) | ✅ Ported |
| **Metered plan scheduler** (background job) | **scheduler** (Tokio binary) | ✅ Ported |

## Admin API Endpoints

| Original | Rust route | Notes |
|----------|------------|--------|
| Subscriptions list | `GET /api/subscriptions` | ✅ |
| Subscription by id | `GET /api/subscriptions/:id` | ✅ |
| Activate subscription | `POST /api/subscriptions/:id/activate` | ✅ |
| Change plan | `PATCH /api/subscriptions/:id/plan` | ✅ |
| Change quantity | `PATCH /api/subscriptions/:id/quantity` | ✅ |
| Emit usage (metering) | `POST /api/subscriptions/:id/usage` | ✅ |
| Subscription audit logs | `GET /api/subscriptions/:id/audit-logs` | ✅ |
| Delete subscription | `DELETE /api/subscriptions/:id` | ✅ |
| Plans list | `GET /api/plans` | ✅ |
| Plan by id | `GET /api/plans/:id` | ✅ |
| Offers list | `GET /api/offers` | ✅ |
| Application config list | `GET /api/config` | ✅ |
| Update application config | `PUT /api/config/:name` | ✅ |
| Auth login | `GET /auth/login` | ✅ (OAuth; callback stubbed until id_token flow) |
| Auth callback | `GET /auth/callback` | ⚠️ Returns 501 until id_token parsing |
| Auth logout | `GET /auth/logout` | ✅ |
| Current user | `GET /api/me` | ✅ (uses session when layer enabled) |
| Health | `GET /health` | ✅ |

## Customer API Endpoints

| Original | Rust route | Notes |
|----------|------------|--------|
| Landing page / resolve token | `GET /api/landing?token=...` | ✅ |
| Subscription by id | `GET /api/subscriptions/:id` | ✅ |
| Activate subscription | `POST /api/subscriptions/:id/activate` | ✅ |
| User subscriptions by email | `GET /api/users/:email/subscriptions` | ✅ |
| User by email | `GET /api/users/:email` | ✅ |
| Health | `GET /health` | ✅ |

## Webhook API

| Original | Rust | Notes |
|----------|------|--------|
| `POST /api/webhook` (Marketplace payload) | `POST /api/webhook` | ✅ JWT auth; idempotency by `operation_id` |
| Actions: Unsubscribe, ChangePlan, ChangeQuantity, Suspend, Reinstate, Renew, Transfer | Same + `Unknown` for future actions | ✅ |
| Idempotency (duplicate delivery) | `webhook_processed_operations` table + repo | ✅ |

## Background / Scheduler

| Original | Rust | Notes |
|----------|------|--------|
| Metered plan scheduler (recurring usage) | **scheduler** crate | ✅ Frequencies, next run, metering API calls |

## Data & Repositories

| Original (EF Core) | Rust (SQLx) | Notes |
|--------------------|-------------|--------|
| Subscriptions, Users, Plans, Offers | Same entities + repos | ✅ |
| Application config, audit logs, application logs | Same | ✅ |
| Email templates, events, plan events mapping | Same | ✅ |
| Metered plan scheduler rows | Same | ✅ |
| Webhook idempotency | `webhook_processed_operations` | ✅ |

## Authentication & Auth

| Original | Rust | Notes |
|----------|------|--------|
| Azure AD OAuth (login/callback/logout) | shared `AuthConfig`, admin-api auth routes | ✅ Login/logout; callback 501 until id_token |
| JWT validation (webhook) | `shared::auth::JwtValidator` (OIDC + JWKS) | ✅ |
| Session (tower-sessions) | Present but session layer disabled on admin (axum 0.7 compat) | ⚠️ TODO |

## Marketplace Integration

| Original | Rust | Notes |
|----------|------|--------|
| Fulfillment API (resolve, activate, list, change plan/quantity, delete) | **marketplace** crate `FulfillmentApiClient` | ✅ |
| Metering API (emit usage) | **marketplace** crate `MeteringApiClient` | ✅ |
| Client (token, HTTP) | **marketplace** `MarketplaceClient` | ✅ |

## Services (business logic)

| Original | Rust | Notes |
|----------|------|--------|
| SubscriptionService | **shared** `SubscriptionServiceImpl` | ✅ |
| Status handlers (Unsubscribe, PendingFulfillment, PendingActivation, Notification) | **shared** status handlers + **webhook-api** | ✅ |
| PlanService, UserService, ApplicationLogService | **shared** | ✅ |
| Email (SMTP, templates, helpers) | **shared** `SmtpEmailService`, `EmailHelper` | ✅ |

## How to build and test

```bash
# From saas-accelerator-rust/
make build    # or: cargo build --workspace
make test     # or: cargo test --workspace
make check    # build then test
```

To run the services you need:

- PostgreSQL (migrations in `crates/data/migrations/`)
- Environment variables (e.g. `DATABASE_URL`, `SaaS_API_*`, `AZURE_AD_*` for auth)

Then:

```bash
make run-admin    # port 3000
make run-customer # port 3001
make run-webhook  # port 3002
make run-scheduler
```

## Gaps / TODOs

- **OAuth callback**: Complete id_token extraction (Azure AD) so login flow persists user in session.
- **Session layer**: Re-enable on admin-api when tower-sessions and axum 0.7 are compatible (or add error mapping).
- **Admin UI**: Original has MVC views; this repo is API-only (frontend can be React/other).
- **Customer UI**: Same; customer-api is API-only.
