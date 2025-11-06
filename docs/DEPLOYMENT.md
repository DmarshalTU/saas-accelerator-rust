# Deployment Guide

This guide covers deploying the SaaS Accelerator - Rust Edition to Azure Web Apps.

## Prerequisites

- Azure CLI installed and configured
- Azure subscription with appropriate permissions
- PostgreSQL database (Azure Database for PostgreSQL or self-hosted)
- Azure AD App Registration configured

## Architecture Overview

The application consists of:
- **Admin API** - Publisher portal backend (Port 3000)
- **Customer API** - Customer portal backend (Port 3001)
- **Webhook API** - Marketplace webhook handler (Port 3002)
- **Scheduler** - Background job service
- **Frontend** - React SPA (Port 5173)

## Deployment Options

### Option 1: Azure Web Apps (Recommended)

Deploy each service as a separate Azure Web App.

#### Step 1: Build the Application

```bash
# Build all Rust binaries
cargo build --release

# Build frontend
cd frontend
npm install
npm run build
cd ..
```

#### Step 2: Deploy Backend Services

Use the provided deployment scripts:

**Linux/Bash:**
```bash
./deployment/deploy.sh
```

**Windows/PowerShell:**
```powershell
.\deployment\Deploy.ps1
```

These scripts will:
1. Create Azure Resource Group
2. Create Azure Database for PostgreSQL
3. Create Azure Web Apps for each service
4. Configure application settings
5. Deploy the application

#### Step 3: Configure Environment Variables

Set the following application settings in Azure Portal for each Web App:

```
DATABASE_URL=postgresql://user:password@server.postgres.database.azure.com:5432/dbname
SaaS_API_TENANT_ID=your-tenant-id
SaaS_API_CLIENT_ID=your-client-id
SaaS_API_CLIENT_SECRET=your-client-secret
SaaS_API_RESOURCE=https://marketplaceapi.microsoft.com
MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api
MARKETPLACE_API_VERSION=2018-08-31
```

### Option 2: Docker Containers

Deploy using Docker containers to Azure Container Instances or Azure Kubernetes Service.

#### Build Docker Images

```bash
docker build -t saas-accelerator-admin-api -f deployment/Dockerfile.admin .
docker build -t saas-accelerator-customer-api -f deployment/Dockerfile.customer .
docker build -t saas-accelerator-webhook-api -f deployment/Dockerfile.webhook .
docker build -t saas-accelerator-scheduler -f deployment/Dockerfile.scheduler .
docker build -t saas-accelerator-frontend -f deployment/Dockerfile.frontend .
```

#### Run Migrations

Before deploying, run database migrations:

```bash
cd crates/data
sqlx migrate run
```

### Option 3: Manual Deployment

1. Build release binaries:
   ```bash
   cargo build --release
   ```

2. Copy binaries to server:
   - `target/release/admin-api`
   - `target/release/customer-api`
   - `target/release/webhook-api`
   - `target/release/scheduler`

3. Set up systemd services (Linux) or Windows Services

4. Configure reverse proxy (nginx/Apache) for frontend

## Database Setup

### Create PostgreSQL Database

```bash
# Using Azure CLI
az postgres flexible-server create \
  --resource-group <resource-group> \
  --name <server-name> \
  --location <location> \
  --admin-user <admin-user> \
  --admin-password <admin-password> \
  --sku-name Standard_B1ms \
  --tier Burstable \
  --version 14

# Create database
az postgres flexible-server db create \
  --resource-group <resource-group> \
  --server-name <server-name> \
  --database-name saas_accelerator
```

### Run Migrations

```bash
export DATABASE_URL=postgresql://user:password@server:5432/saas_accelerator
cd crates/data
sqlx migrate run
```

## Frontend Deployment

### Option 1: Static Web App Hosting

Build and deploy to Azure Static Web Apps:

```bash
cd frontend
npm run build
# Deploy dist/ folder to Azure Static Web Apps
```

### Option 2: Serve with Backend

Configure nginx or Apache to serve the frontend build:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        root /path/to/frontend/dist;
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://localhost:3000;
    }

    location /customer-api {
        proxy_pass http://localhost:3001;
    }
}
```

## Monitoring and Logging

### Application Insights

Enable Azure Application Insights for monitoring:

1. Create Application Insights resource
2. Add connection string to application settings
3. Configure logging in Rust code

### Health Checks

All services expose `/health` endpoints:
- Admin API: `http://localhost:3000/health`
- Customer API: `http://localhost:3001/health`
- Webhook API: `http://localhost:3002/health`

## Scaling

### Horizontal Scaling

- Deploy multiple instances of each API service
- Use Azure Load Balancer or Application Gateway
- Configure session affinity if needed

### Vertical Scaling

- Increase Web App service plan tier
- Scale up PostgreSQL database tier

## Security

### Network Security

- Use Azure Virtual Network for private endpoints
- Configure firewall rules for PostgreSQL
- Enable HTTPS only for Web Apps

### Authentication

- Configure Azure AD authentication
- Use Managed Identity where possible
- Store secrets in Azure Key Vault

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Verify DATABASE_URL is correct
   - Check firewall rules allow your IP
   - Verify SSL mode if required

2. **Azure Authentication Failures**
   - Verify tenant ID, client ID, and secret
   - Check Azure AD app registration permissions
   - Verify resource URL is correct

3. **Webhook Failures**
   - Verify JWT validation is working
   - Check webhook endpoint is publicly accessible
   - Review webhook logs

## Maintenance

### Database Backups

Configure automated backups for PostgreSQL:

```bash
az postgres flexible-server backup create \
  --resource-group <resource-group> \
  --server-name <server-name> \
  --backup-name daily-backup
```

### Updates

1. Pull latest code
2. Build new binaries
3. Run migrations if needed
4. Deploy to staging first
5. Deploy to production

## Support

For issues and questions, please open an issue on GitHub.

