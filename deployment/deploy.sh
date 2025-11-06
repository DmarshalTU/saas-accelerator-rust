#!/bin/bash

# Deployment script for SaaS Accelerator Rust Edition
# This script deploys the application to Azure Web Apps (Linux)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    if ! command -v az &> /dev/null; then
        print_error "Azure CLI is not installed. Please install it first."
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install it first."
        exit 1
    fi
    
    print_info "Prerequisites check passed"
}

# Parse command line arguments
WEB_APP_NAME_PREFIX=""
RESOURCE_GROUP=""
LOCATION=""
PUBLISHER_ADMIN_USERS=""
TENANT_ID=""
AZURE_SUBSCRIPTION_ID=""
AD_APPLICATION_ID=""
AD_APPLICATION_SECRET=""
SQL_SERVER_NAME=""
SQL_DATABASE_NAME=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --web-app-name-prefix)
            WEB_APP_NAME_PREFIX="$2"
            shift 2
            ;;
        --resource-group)
            RESOURCE_GROUP="$2"
            shift 2
            ;;
        --location)
            LOCATION="$2"
            shift 2
            ;;
        --publisher-admin-users)
            PUBLISHER_ADMIN_USERS="$2"
            shift 2
            ;;
        --tenant-id)
            TENANT_ID="$2"
            shift 2
            ;;
        --azure-subscription-id)
            AZURE_SUBSCRIPTION_ID="$2"
            shift 2
            ;;
        --ad-application-id)
            AD_APPLICATION_ID="$2"
            shift 2
            ;;
        --ad-application-secret)
            AD_APPLICATION_SECRET="$2"
            shift 2
            ;;
        --sql-server-name)
            SQL_SERVER_NAME="$2"
            shift 2
            ;;
        --sql-database-name)
            SQL_DATABASE_NAME="$2"
            shift 2
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required parameters
if [[ -z "$WEB_APP_NAME_PREFIX" ]] || [[ -z "$LOCATION" ]] || [[ -z "$PUBLISHER_ADMIN_USERS" ]]; then
    print_error "Required parameters missing. Please provide: --web-app-name-prefix, --location, --publisher-admin-users"
    exit 1
fi

# Set defaults
if [[ -z "$RESOURCE_GROUP" ]]; then
    RESOURCE_GROUP="$WEB_APP_NAME_PREFIX"
fi

if [[ -z "$SQL_SERVER_NAME" ]]; then
    SQL_SERVER_NAME="${WEB_APP_NAME_PREFIX}-sql"
fi

if [[ -z "$SQL_DATABASE_NAME" ]]; then
    SQL_DATABASE_NAME="${WEB_APP_NAME_PREFIX}AMPSaaSDB"
fi

print_info "Starting deployment with the following configuration:"
echo "  Web App Name Prefix: $WEB_APP_NAME_PREFIX"
echo "  Resource Group: $RESOURCE_GROUP"
echo "  Location: $LOCATION"
echo "  SQL Server: $SQL_SERVER_NAME"
echo "  SQL Database: $SQL_DATABASE_NAME"

# Set Azure subscription if provided
if [[ -n "$AZURE_SUBSCRIPTION_ID" ]]; then
    print_info "Setting Azure subscription: $AZURE_SUBSCRIPTION_ID"
    az account set --subscription "$AZURE_SUBSCRIPTION_ID"
fi

# Build Rust projects
print_info "Building Rust projects..."
cargo build --release

# Create resource group
print_info "Creating resource group: $RESOURCE_GROUP"
az group create --name "$RESOURCE_GROUP" --location "$LOCATION" || true

# Create PostgreSQL server (Azure Database for PostgreSQL)
print_info "Creating PostgreSQL server: $SQL_SERVER_NAME"
az postgres flexible-server create \
    --resource-group "$RESOURCE_GROUP" \
    --name "$SQL_SERVER_NAME" \
    --location "$LOCATION" \
    --admin-user saasadmin \
    --admin-password "$(openssl rand -base64 32)" \
    --sku-name Standard_B1ms \
    --tier Burstable \
    --version 14 \
    --storage-size 32 \
    || print_warning "PostgreSQL server may already exist"

# Create database
print_info "Creating database: $SQL_DATABASE_NAME"
az postgres flexible-server db create \
    --resource-group "$RESOURCE_GROUP" \
    --server-name "$SQL_SERVER_NAME" \
    --database-name "$SQL_DATABASE_NAME" \
    || print_warning "Database may already exist"

# Create App Service Plan
print_info "Creating App Service Plan"
APP_SERVICE_PLAN="${WEB_APP_NAME_PREFIX}-asp"
az appservice plan create \
    --name "$APP_SERVICE_PLAN" \
    --resource-group "$RESOURCE_GROUP" \
    --location "$LOCATION" \
    --is-linux \
    --sku B1 \
    || print_warning "App Service Plan may already exist"

# Create Web Apps
print_info "Creating Web Apps"

# Admin API
ADMIN_WEB_APP="${WEB_APP_NAME_PREFIX}-admin"
az webapp create \
    --name "$ADMIN_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_SERVICE_PLAN" \
    --runtime "RUST:1.75" \
    || print_warning "Admin Web App may already exist"

# Customer API
CUSTOMER_WEB_APP="${WEB_APP_NAME_PREFIX}-portal"
az webapp create \
    --name "$CUSTOMER_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_SERVICE_PLAN" \
    --runtime "RUST:1.75" \
    || print_warning "Customer Web App may already exist"

# Webhook API
WEBHOOK_WEB_APP="${WEB_APP_NAME_PREFIX}-webhook"
az webapp create \
    --name "$WEBHOOK_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_SERVICE_PLAN" \
    --runtime "RUST:1.75" \
    || print_warning "Webhook Web App may already exist"

# Configure app settings
print_info "Configuring app settings"

# Get PostgreSQL connection string
POSTGRES_CONNECTION_STRING=$(az postgres flexible-server show-connection-string \
    --server-name "$SQL_SERVER_NAME" \
    --database-name "$SQL_DATABASE_NAME" \
    --admin-user saasadmin \
    --admin-password "$(az postgres flexible-server show -n $SQL_SERVER_NAME -g $RESOURCE_GROUP --query administratorLoginPassword -o tsv)" \
    -s postgresql)

# Set app settings for Admin API
az webapp config appsettings set \
    --name "$ADMIN_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --settings \
        "DATABASE_URL=$POSTGRES_CONNECTION_STRING" \
        "SaaS_API_CLIENT_ID=$AD_APPLICATION_ID" \
        "SaaS_API_CLIENT_SECRET=$AD_APPLICATION_SECRET" \
        "SaaS_API_TENANT_ID=$TENANT_ID" \
        "RUST_LOG=info" \
    || print_warning "Failed to set app settings for Admin API"

# Deploy binaries
print_info "Preparing deployment packages..."

# Create deployment directory
DEPLOY_DIR="deployment/temp"
mkdir -p "$DEPLOY_DIR"

# Copy binaries (when using Azure Web Apps with Rust, we may need to use Docker)
print_info "Note: For Rust deployment to Azure Web Apps, Docker containers are recommended"
print_info "See deployment/Dockerfile for containerized deployment"

print_info "Deployment script completed successfully!"
print_info "Next steps:"
echo "  1. Configure Azure AD applications"
echo "  2. Run database migrations"
echo "  3. Deploy using Docker containers or Azure Container Instances"

