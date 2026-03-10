# SaaS Accelerator — Rust Edition

A complete port of the [Microsoft Commercial Marketplace SaaS Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator) to **Rust + React**, with improved security, performance, and deployment tooling.

## Architecture

Two deployable sites — same model as the original:

| Site | Crate | Purpose |
|------|-------|---------|
| **Admin** | `crates/admin-api` (port 3000) | Publisher portal: subscriptions, plans, offers, config, scheduler, email templates, known users |
| **Customer** | `crates/customer-api` (port 3001) | Customer landing page, subscription management, **and** marketplace webhook |

Shared crates: `shared` (auth, secrets, services), `data` (repositories, models, pool), `marketplace` (fulfillment + metering API clients), `webhook`, `scheduler`.

Frontend: **React + TypeScript + Vite** in `frontend/` — one codebase, two routes (`/` = customer, `/admin` = publisher).

---

## Quick start (local dev)

### Option A — Docker Compose (recommended)

```bash
# Clone, then:
docker compose up -d
```

- Admin portal: **http://localhost:3000**
- Customer portal: **http://localhost:3001**

Default DB: `postgresql://saas:saas@localhost:5432/saas_accelerator`  
Override with a `.env` file — see the comments in `docker-compose.yml`.

### Option B — Run locally

```bash
# Prerequisites: Rust (stable), PostgreSQL 14+, Node.js 20+, sqlx-cli

# 1. Migrations
cd crates/data && DATABASE_URL=... sqlx migrate run && cd ../..

# 2. APIs (separate terminals)
DATABASE_URL=... cargo run -p admin-api     # http://localhost:3000
DATABASE_URL=... cargo run -p customer-api  # http://localhost:3001

# 3. Frontend dev server (hot reload)
cd frontend && npm install && npm run dev   # http://localhost:5173
```

---

## Configuration

### Local dev (`.env` in repo root)

```env
DATABASE_URL=postgresql://saas:saas@localhost:5432/saas_accelerator
RUST_LOG=info

# Optional — enables marketplace API calls
SaaS_API_TENANT_ID=
SaaS_API_CLIENT_ID=
SaaS_API_CLIENT_SECRET=

# Optional — enables Azure AD login on admin portal
AZURE_AD_TENANT_ID=
AZURE_AD_CLIENT_ID=
AZURE_AD_CLIENT_SECRET=
AZURE_AD_REDIRECT_URI=http://localhost:3000/auth/callback
AZURE_AD_SIGNED_OUT_REDIRECT_URI=http://localhost:3000/admin
```

When `AZURE_AD_*` vars are absent, admin routes are open (dev mode).

### Production (Azure Web Apps)

In Azure, authentication is layered:
1. **Azure AD login** — web app reads `AZURE_AD_*` env vars, authenticates the user
2. **Known Users check** — after login, the user's email must be in the `known_users` table with `role_id = 1`
3. **PostgreSQL — passwordless AAD auth** — set `AZURE_AD_AUTH=true`; the app fetches a short-lived OAuth2 token via Managed Identity instead of using a password
4. **Key Vault** — `ADApplicationSecret` (AD app secret) stored in KV; app reads it at startup via Managed Identity

App settings in Azure (non-secrets — set directly):

| Setting | Description |
|---------|-------------|
| `AZURE_AD_AUTH` | `true` — enables passwordless PostgreSQL |
| `DB_HOST` | PostgreSQL server hostname |
| `DB_NAME` | Database name |
| `KEY_VAULT_URL` | e.g. `https://mycompany-kv.vault.azure.net` |
| `SaaS_API_CLIENT_ID` | Fulfillment API app registration ID |
| `SaaS_API_TENANT_ID` | Azure AD tenant ID |
| `AZURE_AD_CLIENT_ID` | Admin SSO app registration ID |
| `CORS_ALLOWED_ORIGINS` | e.g. `https://mycompany-admin.azurewebsites.net` |

---

## Security model

| Layer | Mechanism |
|-------|-----------|
| Admin API routes | Session (Azure AD login) + Known Users (admin role) |
| Customer API routes | Marketplace JWT token (webhook), or email-based lookup |
| Secrets | Azure Key Vault — only `ADApplicationSecret`; no credentials in app settings |
| DB auth (production) | Azure AD Managed Identity — no password, token refreshed every 45 min |
| DB auth (local) | Standard postgres URL in `DATABASE_URL` |
| TLS | HTTPS-only enforced at App Service level; TLS 1.2+ |
| CORS | Locked to `CORS_ALLOWED_ORIGINS`; not permissive |
| Session cookies | Secure flag when running in Azure (`WEBSITE_SITE_NAME` present) |
| ACR pull | Managed Identity `AcrPull` role — no registry credentials stored |
| NSG | DB subnet: port 5432 allowed only from web/ACI subnets |

---

## Deployment to Azure

One-command initial deployment:

```bash
cd deployment
cp .env.template .env   # fill in required values
./deploy.sh --prefix myco --location "East US" \
  --publisher-admin-users "admin@myco.com"
```

The script creates: Resource Group → VNet + NSG → PostgreSQL (private, AAD-enabled) → Key Vault (purge-protected) → ACR → App Service Plan → Admin + Customer Web Apps (container, Managed Identity, HTTPS-only, FTP disabled) → runs DB migrations → seeds Known Users → health-checks.

**Re-runs are safe** — every section checks if the resource exists before creating it.

**Skip flags** (speed up re-runs):

```bash
./deploy.sh --prefix myco --location "East US" \
  --publisher-admin-users "admin@myco.com" \
  --skip-appregistrations --skip-network --skip-db --skip-kv
```

**Upgrade** (rebuild images + migrate + deploy):

```bash
./deployment/upgrade.sh --prefix myco
# or reuse the latest image:
./deployment/upgrade.sh --prefix myco --skip-build
```

**Optional: ACR pull-through cache** (avoids DockerHub rate limits, speeds up builds ~30%):

```bash
# Add to deployment/.env:
DOCKERHUB_USERNAME=myuser
DOCKERHUB_TOKEN=dckr_pat_...   # Docker Hub access token
```

On the next deploy run, credentials are stored in Key Vault and ACR cache rules are created for `rust`, `node`, `debian` base images.

**Build cache** (Docker BuildKit layer cache stored in ACR):  
All builds use `--cache-from/--cache-to type=registry,mode=max` automatically. First build: ~20 min. Subsequent builds with cache: **2–5 min**.

Full details: [docs/AZURE_DEPLOYMENT.md](docs/AZURE_DEPLOYMENT.md)

---

## Testing

```bash
cargo test --workspace           # unit + integration tests
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings allowed
cd frontend && npm audit --audit-level=high             # zero high/critical vulns
cd frontend && npm run build                            # production build
```

---

## Codebase structure

```
saas-accelerator-rust/
├── crates/
│   ├── admin-api/      Admin REST API + Azure AD auth middleware
│   ├── customer-api/   Customer REST API + webhook
│   ├── webhook/        Shared webhook handler (embedded in customer-api)
│   ├── webhook-api/    Optional standalone webhook server
│   ├── scheduler/      Metered billing background job
│   ├── shared/         Models, secrets (KV + AAD pool refresh), services
│   ├── data/           PostgreSQL repositories (SharedPool + AAD token refresh)
│   └── marketplace/    Azure Marketplace fulfillment + metering API clients
├── frontend/           React + TypeScript + Vite (admin and customer UI)
├── deployment/
│   ├── deploy.sh       Full infrastructure + container deployment
│   ├── upgrade.sh      Rebuild images + migrate + redeploy
│   ├── Dockerfile.admin-site
│   ├── Dockerfile.customer-site
│   ├── Dockerfile.migrate
│   ├── nginx-admin-site.conf
│   ├── nginx-customer-site.conf
│   └── .env.template
├── docker-compose.yml  Local development
└── docs/
    ├── AZURE_DEPLOYMENT.md   Full Azure deployment guide + security checklist
    └── CODE_REVIEW.md        Lint configuration and review notes
```

---

## License

MIT — see [LICENSE](LICENSE)
