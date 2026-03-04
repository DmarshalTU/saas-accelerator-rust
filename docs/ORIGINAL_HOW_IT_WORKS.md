# How the Original .NET SaaS Accelerator Works

This document summarizes how the [Commercial Marketplace SaaS Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator) (the original .NET solution in `Commercial-Marketplace-SaaS-Accelerator/`) is structured and run, so the Rust port and its UI can match behavior and flows.

## Original Architecture

### Two separate web applications (same repo)

| App | Purpose | Tech |
|-----|---------|------|
| **AdminSite** | Publisher portal: manage subscriptions, plans, offers, config, metered usage, scheduler, known users, application log. | ASP.NET Core 8 MVC. Controllers return **Razor views** (server-rendered HTML). |
| **CustomerSite** | Customer portal: landing page (with token from marketplace), view/activate/manage subscriptions, change plan/quantity. Also **hosts the webhook** at `/api/AzureWebhook`. | ASP.NET Core 8 MVC. Same pattern: Controllers → Views (Razor). |

There is **no separate “API” project**. Each site is one process: the same app serves the **UI** (HTML from Razor views) and the **backend logic** (controllers call Services/DataAccess; some actions return JSON for partial updates, but the main flow is full page views).

### Shared libraries (used by both sites)

- **Services** – Fulfillment API client, metering, subscription/plan/offer logic, status handlers, email, etc.
- **DataAccess** – Repositories, Entity Framework, SQL Server.
- **MeteredTriggerJob** – Background job for metered billing (separate executable).

### How the UI is served in the original

- **Admin**: Controllers (e.g. `HomeController`, `PlansController`, `OffersController`) handle requests and return `this.View(model)`. Views live under `Views/` (e.g. `Views/Home/Index.cshtml`, `Views/Home/Subscriptions.cshtml`, `Views/Plans/Index.cshtml`). Layout: `Views/Shared/_Layout.cshtml`.
- **Customer**: Same idea. `HomeController` has `Index` (landing, with optional `?token=` from marketplace), `Subscriptions`, `SubscriptionDetail`, `SubscriptionOperationAsync` (activate/deactivate), `ChangeSubscriptionPlan`, `ChangeSubscriptionQuantity`, etc. All return Views.
- **Auth**: Azure AD OpenID Connect. Login redirect, callback, session. Admin uses `[ServiceFilter(typeof(KnownUserAttribute))]` for known-user check.

### How you run it locally (original)

1. Open the solution in **Visual Studio**.
2. Set **AdminSite** or **CustomerSite** as the startup project.
3. Press **F5**. The chosen site runs (e.g. `https://localhost:5001` or `http://localhost:5000` per `launchSettings.json`).
4. You get a **full browser UI**: one URL → server-rendered HTML (Razor). No separate SPA or API URL to think about.
5. **Webhook**: In the original, the webhook is inside CustomerSite at `/api/AzureWebhook`. For local testing, `appsettings.Development.json` can set `ASPNETCORE_ENVIRONMENT=Development` so the webhook can bypass operation verification (local only).

### Deployment (original)

- Two web apps: e.g. **contoso-admin**.azurewebsites.net (Admin) and **contoso-portal**.azurewebsites.net (Customer).
- **Landing page** for the marketplace offer = Customer site root (e.g. https://contoso-portal.azurewebsites.net).
- **Webhook URL** = Customer site webhook (e.g. https://contoso-portal.azurewebsites.net/api/AzureWebhook).
- SQL Server (Azure SQL) and config (appsettings, connection strings, Azure AD app registration).

---

## How the Rust port differs (and matches)

| Aspect | Original (.NET) | Rust port |
|--------|------------------|-----------|
| **Backend** | Logic inside each MVC app (Services + DataAccess). | **Rust APIs**: `admin-api` (3000), `customer-api` (3001), `webhook-api` (3002). Same capabilities, REST JSON. |
| **UI** | Same process as backend; Razor views. | **Separate React SPA** in `frontend/`. One dev server (e.g. port 5173); it calls the Rust APIs via proxy. |
| **DB** | SQL Server, Entity Framework. | PostgreSQL, SQLx. |
| **Webhook** | Inside CustomerSite. | Standalone **webhook-api** (3002). |
| **Running “the UI”** | F5 → one URL → full site. | Run APIs (Docker or `cargo run`) **and** run `frontend` (`npm run dev`) → open http://localhost:5173 for the full Admin/Customer UI. |

So: the **original** is “one process per site, UI + logic together.” The **Rust port** is “Rust APIs + one React app that implements the same screens and calls those APIs.” The flows (landing with token, subscriptions list, activate, change plan/quantity, admin plans/offers/config, etc.) are intended to be the same; only the delivery is different (Razor vs React + REST).

---

## Quick reference: original URLs vs Rust port

| Original (e.g. AdminSite) | Rust port |
|----------------------------|-----------|
| `/` or `/Home/Index` | Admin API root: `GET /` (info page). React app: http://localhost:5173/admin (dashboard). |
| `/Home/Subscriptions` | `GET /api/subscriptions` (API). React: `/admin/subscriptions`. |
| `/Plans/Index` | `GET /api/plans`. React: `/admin/plans`. |
| `/Offers/Index` | `GET /api/offers`. React: `/admin/offers`. |
| `/ApplicationConfig/Index` | `GET /api/config`. React: `/admin/config`. |
| Auth: `/Account/Login` etc. | `GET /auth/login`, `/auth/callback`, `/auth/logout` (admin-api). React uses same or session. |

| Original (CustomerSite) | Rust port |
|-------------------------|-----------|
| `/` or `/Home/Index` with `?token=` | `GET /api/landing?token=...` (API). React: `/` (Landing) and `/subscriptions`. |
| `/Home/Subscriptions` | `GET /api/users/:email/subscriptions` (API). React: `/subscriptions`. |
| Activate / change plan / quantity | `POST` to customer-api equivalents. React calls same APIs. |
| Webhook | `POST /api/AzureWebhook` (same app) | `POST /api/webhook` (webhook-api, port 3002). |

Use this when implementing or testing the React UI so each screen and action aligns with the original behavior.

---

## "How do I add plans?" and who can do what

### Plans and offers – you don’t add them in the UI

- **Plans** and **offers** are not created from the admin UI. They come from your **marketplace offer** in Partner Center and are stored when:
  - The webhook receives subscription lifecycle events (new subscription, etc.), and/or
  - The app syncs or resolves plan/offer data from the Fulfillment API.
- In the admin UI you **view** plans and offers and **open** them to see or edit **details** (e.g. plan attributes/events, offer attributes). The Dashboard and Plans/Offers pages state this.

### What admin can do (not just “watch”)

- **Subscriptions** – List, open one, then: activate, change plan, change quantity, record metered usage, view audit log, delete.
- **Plans / Offers** – View list, open by GUID to see/edit details (no “Add plan” button; plans/offers come from the marketplace).
- **Users** – Add/remove known user emails, Save All.
- **Scheduler** – Add or delete metered-usage schedules; view run log per item.
- **Configuration** – Edit any config key/value; **upload Logo and Favicon** (png/ico).
- **Application logs** – View log.
- **Email templates** – View and edit by status.

### What the customer can do (all their control)

- **Landing** (`/`) – With `?token=...`: one-click activate subscription. Without token: paste subscription ID and Activate. Link to My Subscriptions and Privacy.
- **My Subscriptions** (`/subscriptions`) – Enter email → **Load** → list of subscriptions. **Click a subscription** to open it.
- **Subscription detail** (`/subscriptions/:id`) – **Activate** (if pending), **Change plan**, **Change quantity**. Success message after each action.

So: admin is not “static pages” – you manage subscriptions, users, scheduler, config, and templates. Plans/offers are viewed and edited in place; they are not “added” in the UI. Customer control is: Landing (activate with token/ID), Subscriptions list (load by email), and Subscription detail (activate, change plan, change quantity).
