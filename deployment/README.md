# Deployment Guide

This directory contains deployment scripts and configuration for the SaaS Accelerator Rust Edition.

## Prerequisites

- Azure CLI installed and configured
- Rust toolchain installed
- Docker (for containerized deployment)
- Azure subscription with appropriate permissions

## Deployment Options

### Option 1: Azure Web Apps (Linux) with Docker

1. Build Docker images:
```bash
docker build -t saas-accelerator-admin:latest -f deployment/Dockerfile --target admin-api .
docker build -t saas-accelerator-customer:latest -f deployment/Dockerfile --target customer-api .
docker build -t saas-accelerator-webhook:latest -f deployment/Dockerfile --target webhook-api .
```

2. Push to Azure Container Registry:
```bash
az acr login --name <your-registry>
docker tag saas-accelerator-admin:latest <your-registry>.azurecr.io/saas-accelerator-admin:latest
docker push <your-registry>.azurecr.io/saas-accelerator-admin:latest
```

3. Deploy to Azure Web Apps using the deployment script:
```bash
./deployment/deploy.sh \
  --web-app-name-prefix "saas-acc" \
  --location "eastus" \
  --publisher-admin-users "admin@example.com"
```

### Option 2: Manual Deployment

1. Build the projects:
```bash
cargo build --release
```

2. Run database migrations:
```bash
cd crates/data
sqlx migrate run
```

3. Configure environment variables (see `.env.example`)

4. Run the services:
```bash
cargo run --bin admin-api
cargo run --bin customer-api
cargo run --bin webhook-api
```

## Environment Variables

See `.env.example` for required environment variables.

## Database Setup

The project uses PostgreSQL. You can use:
- Azure Database for PostgreSQL (Flexible Server)
- Local PostgreSQL for development

Run migrations:
```bash
export DATABASE_URL="postgresql://user:password@host:5432/database"
sqlx migrate run
```

## Troubleshooting

### Build Issues
- Ensure Rust 1.70+ is installed
- Check that all dependencies are available

### Database Connection Issues
- Verify DATABASE_URL is correct
- Check firewall rules if using Azure Database for PostgreSQL
- Ensure SSL is properly configured

### Azure Deployment Issues
- Verify Azure CLI is logged in: `az account show`
- Check resource group permissions
- Ensure Docker images are pushed to Azure Container Registry

