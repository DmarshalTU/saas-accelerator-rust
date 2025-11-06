# Porting Guide: ASP.NET to Rust

This document outlines the architecture decisions and mapping from the original ASP.NET implementation to the Rust version.

## Architecture Overview

### Original Structure → Rust Structure

| Original Component | Rust Equivalent | Status |
|-------------------|-----------------|--------|
| `AdminSite` (ASP.NET MVC) | `crates/admin-api` (Axum REST API) | 🚧 In Progress |
| `CustomerSite` (ASP.NET MVC) | `crates/customer-api` (Axum REST API) | 🚧 In Progress |
| `Services` (Business Logic) | `crates/shared` + `crates/marketplace` | ✅ Complete |
| `DataAccess` (EF Core) | `crates/data` (SQLx) | 🚧 In Progress |
| `MeteredTriggerJob` | `crates/scheduler` | 🚧 In Progress |
| Frontend (Razor Views) | Separate SPA (TBD) | 📋 Planned |

## Key Design Decisions

### 1. **Marketplace API Clients**

**Original**: Uses `Microsoft.Marketplace.SaaS` and `Microsoft.Marketplace.Metering` .NET SDKs

**Rust**: Uses Azure SDK for Rust (`azure_identity`, `azure_core`) with direct REST API calls to:
- `https://marketplaceapi.microsoft.com/api/saas/...` (Fulfillment API)
- `https://marketplaceapi.microsoft.com/api/metered/...` (Metering API)

**Why**: Azure SDK for Rust doesn't have Marketplace-specific crates, but provides excellent HTTP client and authentication infrastructure.

### 2. **Database**

**Original**: SQL Server with Entity Framework Core

**Rust**: PostgreSQL with SQLx

**Why**: 
- Better Linux support
- SQLx provides compile-time checked SQL queries
- Similar performance to SQL Server
- Azure Database for PostgreSQL is fully supported

### 3. **Web Framework**

**Original**: ASP.NET Core MVC with Razor views

**Rust**: Axum REST API + Separate SPA frontend

**Why**:
- Modern async/await patterns
- Type-safe request/response handling
- Separation of concerns (frontend team can work independently)
- Better hot reload support with modern frontend tools

### 4. **Authentication**

**Original**: ASP.NET Core OpenID Connect middleware

**Rust**: `openidconnect` crate with Azure AD integration

**Why**: 
- Official OpenID Connect implementation
- Works with Azure AD
- Can use Azure SDK for token validation

### 5. **Background Jobs**

**Original**: Azure WebJobs

**Rust**: Custom scheduler service using `background` crate or SQLx polling

**Why**: 
- More control over scheduling
- Can run as separate service or within API
- Better observability

## API Endpoint Mapping

### Admin API (Publisher Portal)

| Original Route | Rust Route | Method |
|---------------|------------|--------|
| `/Home/Index` | `GET /api/subscriptions` | List subscriptions |
| `/Home/Subscriptions/{id}` | `GET /api/subscriptions/{id}` | Get subscription |
| `/Home/Activate/{id}` | `POST /api/subscriptions/{id}/activate` | Activate subscription |
| `/Home/ChangePlan/{id}` | `PATCH /api/subscriptions/{id}/plan` | Change plan |
| `/Home/ManageSubscriptionUsage` | `POST /api/subscriptions/{id}/usage` | Emit usage event |

### Customer API (Landing Page)

| Original Route | Rust Route | Method |
|---------------|------------|--------|
| `/Home/Index` | `GET /api/landing` | Landing page |
| `/Home/Activate/{id}` | `POST /api/subscriptions/{id}/activate` | Activate subscription |

### Webhook API

| Original Route | Rust Route | Method |
|---------------|------------|--------|
| `/api/AzureWebhook` | `POST /api/webhook` | Handle marketplace webhooks |

## Database Migration

The Entity Framework migrations need to be converted to SQLx migrations. Key changes:

1. **Naming**: EF uses PascalCase, SQLx uses snake_case (PostgreSQL convention)
2. **Types**: 
   - `nvarchar` → `VARCHAR` or `TEXT`
   - `datetime` → `TIMESTAMP WITH TIME ZONE`
   - `uniqueidentifier` → `UUID`
   - `int` → `INTEGER` or `BIGINT`

3. **Relationships**: Foreign keys remain the same, but syntax differs slightly

## Deployment

### Original
- PowerShell script (`Deploy.ps1`)
- Creates Azure resources via Azure CLI
- Deploys .NET apps to Azure Web Apps

### Rust
- Bash script (`deploy.sh`) for Linux/macOS
- PowerShell script (`deploy.ps1`) for Windows
- Builds Rust binaries
- Deploys to Azure Web Apps (Linux)

## Status

- ✅ Project structure and workspace setup
- ✅ Marketplace API clients (Fulfillment & Metering)
- ✅ Shared models and error types
- 🚧 Database layer (SQLx + migrations)
- 🚧 API servers (Axum)
- 🚧 Authentication middleware
- 🚧 Scheduler service
- 📋 Frontend SPA
- 📋 Deployment scripts

## Next Steps

1. Complete database migrations (convert EF to SQLx)
2. Implement repository pattern in `crates/data`
3. Create Axum API servers with authentication
4. Implement webhook handler
5. Create scheduler service
6. Set up frontend project (React/Vue/Svelte)
7. Create deployment scripts

