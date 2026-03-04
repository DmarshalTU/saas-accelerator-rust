# Two-Site Deployment: Admin and Customer Isolation

## Why two sites (like the original)

In production you deploy **2 different web apps** (2 URLs): one Admin, one Customer. The original SaaS Accelerator does the same — Azure deployment creates **separate web applications**:

- **Publisher (Admin) portal** – one web app, e.g. `contoso-admin.azurewebsites.net`
- **Customer (provisioning) portal** – one web app, e.g. `contoso-portal.azurewebsites.net` (landing page + webhook in the same app)

**Admin isolation from Customer** matters for:

1. **Security** – Publisher and customer traffic and data are in separate apps. A compromise or abuse on the customer-facing app does not directly impact the admin app.
2. **Cost** – Scale and cost can be attributed and controlled per app. Admin is often low traffic; customer/landing can be scaled independently.
3. **Deployment** – Update or roll back one site without touching the other.
4. **Compliance** – Clear boundary between “publisher back office” and “customer-facing” systems.

This repo is structured so that **deployment mirrors the original**: two deployable “sites,” each with its own backend and UI.

---

## Target architecture

| Site | Contents | Deployable | Azure (example) |
|------|----------|------------|------------------|
| **Admin** | Admin API + Admin UI (static) | One image / one Web App | `*-admin.azurewebsites.net` |
| **Customer** | Customer API + Webhook API + Customer UI (static) | One image / one Web App | `*-portal.azurewebsites.net` (landing + webhook) |

- **Admin site**: Single process. Serves REST API (e.g. `/api/*`, `/auth/*`) and serves the Admin SPA (e.g. `/*` for UI). Port 3000 (or 443).
- **Customer site**: Single process. Serves Customer REST API (e.g. `/api/landing`, `/api/subscriptions/*`, `/api/users/*`), **webhook** at `POST /api/webhook`, and serves the Customer SPA (e.g. `/*` for UI). Port 3001 (or 443). Marketplace **Landing page URL** and **Webhook URL** both point to this same host (e.g. `https://contoso-portal.azurewebsites.net` and `https://contoso-portal.azurewebsites.net/api/webhook`).

Shared infrastructure (database, migrate) is separate. Scheduler (metered job) can be a separate job/container or co-located with Admin depending on preference.

---

## Implementation summary

- **Webhook** is part of the Customer site (same binary as customer-api), matching the original where webhook lives in CustomerSite.
- **Frontend** is built as two outputs: Admin UI bundle and Customer UI bundle. Each site serves its own bundle as static files.
- **Docker**: Two images – `saas-accelerator-admin` (admin-api + admin UI) and `saas-accelerator-customer` (customer-api including webhook + customer UI). Compose or Azure deploy runs these as two services plus Postgres and migrate.
- **Local dev**: Can still run admin-api and customer-api (with webhook) as separate processes; optional standalone `webhook-api` binary is no longer required for the standard two-site deploy.

---

## Mapping to original

| Original | Rust two-site |
|----------|----------------|
| AdminSite (MVC + API) | Admin image: admin-api + Admin UI |
| CustomerSite (MVC + API + `/api/AzureWebhook`) | Customer image: customer-api (includes `/api/webhook`) + Customer UI |
| Two Azure Web Apps | Two images / two Web Apps |

---

## Implementation status

- **Done**: Webhook lib; customer-api embeds webhook (POST /api/webhook, POST /api/webhook/health); optional standalone webhook-api; Docker Compose and smoke test updated for two-site (Admin + Customer).
- **Optional later** (to complete “Customer site = one deployable with webhook”):
  Static UI and per-site Docker images with bundled UI.
