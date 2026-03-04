# How This Gets Deployed to Azure

This document describes how to deploy the Rust SaaS Accelerator to Azure so that **Admin is secured** and **Customer** traffic is isolated, matching the [original .NET accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator) model.

---

## Security: Admin vs Customer

| Concern | Answer |
|--------|--------|
| **Is Admin secured when deployed to Azure?** | **Yes**, when you configure Azure AD (see below). The admin API requires a valid session (login via Azure AD) and, when Azure AD is configured, the user must be in the **Known Users** list with admin role. Unauthenticated requests get **401**; signed-in users not in Known Users get **403**. |
| **Can anyone enter Admin in Docker/local?** | With **no** Azure AD env vars, the admin API is **open** (dev mode). Set `AZURE_AD_*` and add users to Known Users via the Admin UI to lock it down. |
| **Are customers secured?** | **Yes.** Customers use only the **Customer** URL and API. They never hit the Admin URL. Customer API has no Azure AD login; it uses marketplace tokens and subscription context. |
| **Can a customer accidentally log into Admin?** | **No.** Admin and Customer are **different URLs** (different Azure Web Apps). A customer goes to the customer site; they never see the admin login. If someone opens the admin URL and signs in, they only get in if their email is in Known Users (admin role). |

**Summary:** Use **two separate Azure Web Apps** (admin URL and customer URL). Configure Azure AD and Known Users on the admin app. Customers use only the customer URL.

---

## In Production: 2 Different Web Apps (2 URLs)

In real deployment you have **two separate Azure Web Apps** (two URLs, two resources):

| Web App   | URL (example)                        | Who uses it        |
|-----------|--------------------------------------|--------------------|
| **Admin** | `https://your-app-admin.azurewebsites.net`   | Publishers (your team) |
| **Customer** | `https://your-app-customer.azurewebsites.net` | End customers (marketplace users) |

- Each is its own Azure App Service / container app (separate scaling, config, and security).
- Users never mix the two: they open either the admin URL or the customer URL.
- You deploy the **same** frontend bundle to **both**; each web app runs its own API and serves that bundle. On the admin URL the app shows the admin UI; on the customer URL it shows the customer UI.

So in production it really is **2 different webapps** — two distinct sites, two URLs. Local/Docker (one UI at 5173 with `/` and `/admin`) is for development only.

---

## What the Original Actually Does (Database)

In the **original** .NET SaaS Accelerator, **both** the Admin Web App and the Customer/Portal Web App use the **same** Azure SQL database:

- **One** SQL server, **one** database (e.g. `contosoAMPSaaSDB`).
- **Same** connection string key in both: `DefaultConnection` (from Key Vault).
- Deploy.ps1 sets `DefaultConnection` to the **same** Key Vault secret for both `*-admin` and `*-portal` web apps.

So the original uses a **shared database**. Isolation in the original is **at the app level** (two separate Web Apps = two URLs, separate scaling, separate Managed Identities, separate deployment), **not** at the database level. Our Rust port matches that: one DB, two apps.

---

## Yes: Frontend Is Still One Project

The UI is **one React app** in `frontend/`:

- **One codebase**, one `npm run build` → one `dist/` bundle.
- **Two areas in the app**: `/admin/*` (publisher) and `/` (customer), via React Router.
- **Two API base URLs** in code: `VITE_ADMIN_API_URL` and `VITE_CUSTOMER_API_URL` (see `frontend/src/api/client.ts`). Admin pages use the admin API, customer pages use the customer API.

So: one frontend project, one build, but it’s **deployed twice** (once per Azure Web App), and each deployment is paired with a **different backend**.

---

## Two Azure Web Apps (Like the Original)

You end up with **two separate Azure Web Apps** (or App Service / Container Apps), each with its own URL and resources:

| Azure Web App | What runs there | URL example |
|---------------|------------------|-------------|
| **Admin**     | admin-api + same static UI (from `dist/`) | `https://your-app-admin.azurewebsites.net` |
| **Customer**  | customer-api (with webhook) + same static UI | `https://your-app-customer.azurewebsites.net` |

Shared: one Azure PostgreSQL (or other DB), same connection string used by both apps — same as the original (one DB, two Web Apps).

So:

- **Admin site** = one Web App = **admin-api + UI** (admin gets its own URL, scale, and security boundary).
- **Customer site** = one Web App = **customer-api + webhook + UI** (customer + marketplace webhook on one URL).

That matches the original accelerator’s “two separate web apps” model.

---

## Deployment Flow (Concrete)

### 1. Build the frontend **once**

Set the two API URLs to your **production** hostnames and build:

```bash
cd frontend
export VITE_ADMIN_API_URL=https://your-app-admin.azurewebsites.net
export VITE_CUSTOMER_API_URL=https://your-app-customer.azurewebsites.net
npm ci
npm run build
```

You get a single **`dist/`** (one project, one build). That same folder is used for **both** Web Apps.

### 2. Admin Web App

- **Code/runtime**: Run **admin-api** (Rust binary or container).
- **Static files**: Serve the contents of **`dist/`** (e.g. static file server or same process).
- **Routing** (conceptually):
  - `GET /api/*` → admin-api.
  - `GET /*` → static files (e.g. `index.html` for SPA, assets from `dist/`).

So when someone opens `https://your-app-admin.azurewebsites.net`, they get the React app; all `/api` calls stay on that host and hit admin-api. Admin pages in the app already use `VITE_ADMIN_API_URL`, which points to this host → correct.

### 3. Customer Web App

- **Code/runtime**: Run **customer-api** (same binary that includes the webhook).
- **Static files**: Same **`dist/`** as above.
- **Routing**:
  - `GET/POST /api/*` (including `POST /api/webhook`, `POST /api/webhook/health`) → customer-api.
  - `GET /*` → static files.

When someone opens `https://your-app-customer.azurewebsites.net`, they get the same React app; all customer API calls (and webhook) go to this host. Customer pages use `VITE_CUSTOMER_API_URL` → this host → correct.

### 4. Database and config

- One **Azure PostgreSQL** (or shared DB).
- **Connection string** and other config (e.g. Marketplace, Azure AD) set in each Web App’s app settings (or Key Vault), so each app talks to the same DB but has its own config if needed.

---

## How Serving the UI Can Work Technically

You need each Web App to:

1. Run the Rust API (admin-api or customer-api).
2. Serve the built UI from `dist/`.

Ways to do that:

- **Option A – API serves static files**  
  Build admin-api/customer-api so they can serve a directory of static files (e.g. `dist/`) for non-`/api` routes and fallback `index.html` for SPA. Then one process does both API + UI.

- **Option B – Reverse proxy in front**  
  One process (e.g. nginx or Caddy) in the container:  
  - `/api/*` → proxy to the Rust API (admin-api or customer-api).  
  - `/*` → serve `dist/` (and `index.html` for client-side routes).

- **Option C – Azure static + custom backend**  
  Put `dist/` in Azure Static Web Apps or a storage + CDN, and point both “sites” to the same static bundle; each site’s “backend” is the corresponding API (Admin vs Customer). Less common for this setup but possible.

So: **one frontend project, one build, two deployments** (Admin Web App and Customer Web App), each deployment = one API + same UI bundle. That’s how this is intended to be deployed to Azure.

---

## Checklist: Securing Admin in Production

1. **Two Web Apps** – Deploy admin-api to one Azure Web App (admin URL) and customer-api to another (customer URL). Do not expose admin API on the customer URL.
2. **Azure AD** – Set these on the **Admin** Web App only: `AZURE_AD_TENANT_ID`, `AZURE_AD_CLIENT_ID`, `AZURE_AD_CLIENT_SECRET`, `AZURE_AD_REDIRECT_URI` (e.g. `https://<your-admin-host>/auth/callback`), `AZURE_AD_SIGNED_OUT_REDIRECT_URI` (e.g. `https://<your-admin-host>/admin`).
3. **Known Users** – After first deploy, sign in to the admin site with an account that you will add to Known Users, then add your publisher emails to **Known Users** (Admin UI) with role **1** (admin). Only those users can access admin API once Azure AD is configured.
4. **HTTPS** – Use HTTPS in production; set the session cookie to secure in code or via reverse proxy.
5. **Customer site** – No Azure AD needed. Customers use only the customer URL; they cannot "log into" admin.

---

## Azure Pipelines (CI/CD)

A pipeline is defined in **`azure-pipelines.yml`** at the repo root.

**On every push/PR to `main`/`master`:**
- **Build stage:** Rust (build, test, clippy), Frontend (npm ci, npm run build). Publishes artifacts on `main`.

**On push to `main` only (after Build succeeds):**
- **Deploy stage:** Builds the **admin-site** and **customer-site** Docker images (from `deployment/Dockerfile.admin-site` and `Dockerfile.customer-site`), pushes them to your Azure Container Registry, and deploys each image to its Azure Web App. Your Web Apps are updated with the new version automatically.

### One-time setup

1. **Azure resources:** Two Web Apps (Linux, container), one Azure Container Registry (ACR). Create them in the portal or via IaC. Ensure each Web App is configured to pull from ACR (e.g. Managed identity or admin credentials).

2. **Azure DevOps – Service connections:**
   - **Docker Registry** type, pointing to your ACR (for pushing images).
   - **Azure Resource Manager** type, with access to the subscription where the Web Apps and ACR live.

3. **Azure DevOps – Environments:** In Pipelines → Environments, create **`production-admin`** and **`production-customer`** (optional but recommended for approval gates and history).

4. **Pipeline variables:** In Pipelines → your pipeline → Edit → Variables, set:

   | Variable | Description |
   |----------|-------------|
   | `DockerRegistryServiceConnection` | Name of the Docker Registry service connection (your ACR). |
   | `ContainerRegistryHost` | ACR login server, e.g. `myregistry.azurecr.io`. |
   | `AzureServiceConnection` | Name of the Azure Resource Manager service connection. |
   | `AdminWebAppName` | Exact name of the Admin Azure Web App. |
   | `CustomerWebAppName` | Exact name of the Customer Azure Web App. |
   | `VITE_ADMIN_API_URL` | (Optional) e.g. `https://<admin-app>.azurewebsites.net` – used when building the frontend inside the Docker image. |
   | `VITE_CUSTOMER_API_URL` | (Optional) e.g. `https://<customer-app>.azurewebsites.net`. |

After this, every push to `main` runs the pipeline and deploys the new version to both Web Apps.
