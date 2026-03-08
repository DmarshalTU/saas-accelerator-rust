#!/bin/bash

# Deployment script for SaaS Accelerator Rust Edition
# This script deploys the application to Azure Web Apps (Linux)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Log file
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPLOY_LOG="$SCRIPT_DIR/deploy-$(date +%Y%m%d-%H%M%S).log"
echo "Deployment log started at $(date)" > "$DEPLOY_LOG"

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
    echo "[INFO] $(date +%H:%M:%S) $1" >> "$DEPLOY_LOG"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    echo "[WARN] $(date +%H:%M:%S) $1" >> "$DEPLOY_LOG"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    echo "[ERROR] $(date +%H:%M:%S) $1" >> "$DEPLOY_LOG"
}

# Run a command with error handling. Logs stdout/stderr on success and failure.
run_step() {
    local step_name="$1"
    shift
    print_info "$step_name"
    local output
    if output=$("$@" 2>&1); then
        echo "$output" >> "$DEPLOY_LOG"
        return 0
    else
        local exit_code=$?
        print_error "$step_name FAILED (exit code $exit_code)"
        echo "[ERROR] $(date +%H:%M:%S) $step_name output:" >> "$DEPLOY_LOG"
        echo "$output" >> "$DEPLOY_LOG"
        print_error "See log for details: $DEPLOY_LOG"
        return $exit_code
    fi
}

# Global error trap — catches any unhandled failure
on_error() {
    local exit_code=$?
    local line_no=$1
    print_error "Script failed at line $line_no (exit code $exit_code)"
    print_error "Log file: $DEPLOY_LOG"
}
trap 'on_error $LINENO' ERR

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

    if ! command -v cross &> /dev/null; then
        print_error "cross is not installed. Install with: cargo install cross --git https://github.com/cross-rs/cross"
        exit 1
    fi

    if ! command -v docker &> /dev/null; then
        print_error "Docker is required for cross-compilation. Please install Docker Desktop."
        exit 1
    fi

    print_info "Prerequisites check passed"
}

check_prerequisites

# Load environment variables from .env file
ENV_FILE="$SCRIPT_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
    print_info "Loading configuration from $ENV_FILE"
    set -a
    source "$ENV_FILE"
    set +a
else
    print_error "Missing $ENV_FILE — copy deployment/.env.template and fill in your values"
    exit 1
fi

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

ADMIN_WEB_APP="${WEB_APP_NAME_PREFIX}-admin"
CUSTOMER_WEB_APP="${WEB_APP_NAME_PREFIX}-portal"
APP_SERVICE_PLAN="${WEB_APP_NAME_PREFIX}-asp"
ADMIN_URL="https://${ADMIN_WEB_APP}.azurewebsites.net"
CUSTOMER_URL="https://${CUSTOMER_WEB_APP}.azurewebsites.net"

print_info "Starting deployment with the following configuration:"
echo "  Web App Name Prefix: $WEB_APP_NAME_PREFIX"
echo "  Resource Group: $RESOURCE_GROUP"
echo "  Location: $LOCATION"
echo "  SQL Server: $SQL_SERVER_NAME"
echo "  SQL Database: $SQL_DATABASE_NAME"
echo "  Admin URL: $ADMIN_URL"
echo "  Customer URL: $CUSTOMER_URL"

# Set Azure subscription if provided
if [[ -n "$AZURE_SUBSCRIPTION_ID" ]]; then
    run_step "Setting Azure subscription: $AZURE_SUBSCRIPTION_ID" \
        az account set --subscription "$AZURE_SUBSCRIPTION_ID"
fi

# Build Rust projects (cross-compile for Linux x86_64 via Docker)
run_step "Building Rust projects (x86_64-unknown-linux-gnu)" \
    cross build --release --target x86_64-unknown-linux-gnu

# Create resource group
run_step "Creating resource group: $RESOURCE_GROUP" \
    az group create --name "$RESOURCE_GROUP" --location "$LOCATION" \
    || print_warning "Resource group may already exist"

# Create PostgreSQL server (Azure Database for PostgreSQL)
run_step "Creating PostgreSQL server: $SQL_SERVER_NAME" \
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
    --yes \
    || print_warning "PostgreSQL server may already exist"

# Create database
run_step "Creating database: $SQL_DATABASE_NAME" \
    az postgres flexible-server db create \
    --resource-group "$RESOURCE_GROUP" \
    --server-name "$SQL_SERVER_NAME" \
    --database-name "$SQL_DATABASE_NAME" \
    || print_warning "Database may already exist"

# Create App Service Plan
run_step "Creating App Service Plan: $APP_SERVICE_PLAN" \
    az appservice plan create \
    --name "$APP_SERVICE_PLAN" \
    --resource-group "$RESOURCE_GROUP" \
    --location "$LOCATION" \
    --is-linux \
    --sku B1 \
    || print_warning "App Service Plan may already exist"

# Create Web Apps
run_step "Creating Admin Web App: $ADMIN_WEB_APP" \
    az webapp create \
    --name "$ADMIN_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_SERVICE_PLAN" \
    --runtime "NODE:20-lts" \
    || print_warning "Admin Web App may already exist"

run_step "Creating Customer Web App: $CUSTOMER_WEB_APP" \
    az webapp create \
    --name "$CUSTOMER_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_SERVICE_PLAN" \
    --runtime "NODE:20-lts" \
    || print_warning "Customer Web App may already exist"

# Configure startup commands (run the Rust binary)
run_step "Setting Admin startup command" \
    az webapp config set --name "$ADMIN_WEB_APP" --resource-group "$RESOURCE_GROUP" \
    --startup-file "./admin-api" \
    || print_warning "Failed to set Admin startup command"

run_step "Setting Customer startup command" \
    az webapp config set --name "$CUSTOMER_WEB_APP" --resource-group "$RESOURCE_GROUP" \
    --startup-file "./customer-api" \
    || print_warning "Failed to set Customer startup command"

# Configure app settings
POSTGRES_CONNECTION_STRING=$(az postgres flexible-server show-connection-string \
    --server-name "$SQL_SERVER_NAME" \
    --database-name "$SQL_DATABASE_NAME" \
    --admin-user saasadmin \
    --admin-password "$(az postgres flexible-server show -n "$SQL_SERVER_NAME" -g "$RESOURCE_GROUP" --query administratorLoginPassword -o tsv 2>/dev/null || echo 'REPLACE_ME')" \
    -s postgresql 2>/dev/null || echo "postgresql://saasadmin:REPLACE_ME@${SQL_SERVER_NAME}.postgres.database.azure.com:5432/${SQL_DATABASE_NAME}")

run_step "Configuring Admin app settings" \
    az webapp config appsettings set \
    --name "$ADMIN_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --settings \
        "DATABASE_URL=$POSTGRES_CONNECTION_STRING" \
        "SaaS_API_CLIENT_ID=$AD_APPLICATION_ID" \
        "SaaS_API_CLIENT_SECRET=$AD_APPLICATION_SECRET" \
        "SaaS_API_TENANT_ID=$TENANT_ID" \
        "RUST_LOG=info" \
        "PORT=8080" \
    || print_warning "Failed to set app settings for Admin API"

run_step "Configuring Customer app settings" \
    az webapp config appsettings set \
    --name "$CUSTOMER_WEB_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --settings \
        "DATABASE_URL=$POSTGRES_CONNECTION_STRING" \
        "SaaS_API_CLIENT_ID=$AD_APPLICATION_ID" \
        "SaaS_API_CLIENT_SECRET=$AD_APPLICATION_SECRET" \
        "SaaS_API_TENANT_ID=$TENANT_ID" \
        "RUST_LOG=info" \
        "PORT=8080" \
    || print_warning "Failed to set app settings for Customer API"

# Build frontend
print_info "Building frontend..."
cd "$REPO_ROOT/frontend"
if ! VITE_ADMIN_API_URL="$ADMIN_URL" VITE_CUSTOMER_API_URL="$CUSTOMER_URL" npm ci >> "$DEPLOY_LOG" 2>&1; then
    print_error "npm ci failed"
    exit 1
fi
if ! VITE_ADMIN_API_URL="$ADMIN_URL" VITE_CUSTOMER_API_URL="$CUSTOMER_URL" npm run build >> "$DEPLOY_LOG" 2>&1; then
    print_error "npm run build failed"
    exit 1
fi
cd "$REPO_ROOT"

# Package Admin site
print_info "Packaging Admin site..."
PUBLISH_DIR="$REPO_ROOT/Publish"
rm -rf "$PUBLISH_DIR"
mkdir -p "$PUBLISH_DIR/AdminSite" "$PUBLISH_DIR/CustomerSite"

cp "target/x86_64-unknown-linux-gnu/release/admin-api" "$PUBLISH_DIR/AdminSite/"
cp -r "$REPO_ROOT/frontend/dist" "$PUBLISH_DIR/AdminSite/wwwroot"

cd "$PUBLISH_DIR/AdminSite"
zip -r "$PUBLISH_DIR/AdminSite.zip" . -q
cd "$REPO_ROOT"

# Package Customer site
print_info "Packaging Customer site..."
cp "target/x86_64-unknown-linux-gnu/release/customer-api" "$PUBLISH_DIR/CustomerSite/"
cp -r "$REPO_ROOT/frontend/dist" "$PUBLISH_DIR/CustomerSite/wwwroot"

cd "$PUBLISH_DIR/CustomerSite"
zip -r "$PUBLISH_DIR/CustomerSite.zip" . -q
cd "$REPO_ROOT"

# Deploy via zip deploy (same approach as original .NET SaaS Accelerator)
run_step "Deploying Admin site via zip deploy" \
    az webapp deploy \
    --resource-group "$RESOURCE_GROUP" \
    --name "$ADMIN_WEB_APP" \
    --src-path "$PUBLISH_DIR/AdminSite.zip" \
    --type zip

run_step "Deploying Customer site via zip deploy" \
    az webapp deploy \
    --resource-group "$RESOURCE_GROUP" \
    --name "$CUSTOMER_WEB_APP" \
    --src-path "$PUBLISH_DIR/CustomerSite.zip" \
    --type zip

# Clean up
rm -rf "$PUBLISH_DIR"

print_info "Deployment completed!"
print_info "Log file: $DEPLOY_LOG"
echo ""
echo "  Admin:    $ADMIN_URL"
echo "  Customer: $CUSTOMER_URL"
