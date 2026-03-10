#!/usr/bin/env bash
# =============================================================================
# SaaS Accelerator – Rust Edition  |  Initial Deployment Script
# Equivalent to the original Deploy.ps1 but for Rust + containers + PostgreSQL.
#
# Usage:
#   ./deploy.sh \
#     --prefix mycompany \
#     --location "East US" \
#     --publisher-admin-users "admin@example.com,other@example.com"
#
# All parameters can also be set in deployment/.env (see .env.template).
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPLOY_LOG="$SCRIPT_DIR/deploy-$(date +%Y%m%d-%H%M%S).log"
echo "=== SaaS Accelerator Deploy  $(date) ===" > "$DEPLOY_LOG"

# ── colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${GREEN}[INFO]${NC}  $*"; echo "[INFO]  $(date +%T) $*" >> "$DEPLOY_LOG"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; echo "[WARN]  $(date +%T) $*" >> "$DEPLOY_LOG"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; echo "[ERROR] $(date +%T) $*" >> "$DEPLOY_LOG"; }
section() { echo -e "\n${CYAN}══ $* ══${NC}"; echo -e "\n═══ $* ═══" >> "$DEPLOY_LOG"; }
die()     { error "$*"; exit 1; }
on_error() { error "Script failed at line $1 (exit $2)"; error "Log: $DEPLOY_LOG"; }
trap 'on_error $LINENO $?' ERR

# ── load .env if present ──────────────────────────────────────────────────────
ENV_FILE="$SCRIPT_DIR/.env"
[[ -f "$ENV_FILE" ]] && { info "Loading $ENV_FILE"; set -a; source "$ENV_FILE"; set +a; }

# ── CLI args (override .env) ──────────────────────────────────────────────────
WEB_APP_NAME_PREFIX="${WEB_APP_NAME_PREFIX:-}"
RESOURCE_GROUP="${RESOURCE_GROUP:-}"
LOCATION="${LOCATION:-}"
PUBLISHER_ADMIN_USERS="${PUBLISHER_ADMIN_USERS:-}"
TENANT_ID="${TENANT_ID:-}"
AZURE_SUBSCRIPTION_ID="${AZURE_SUBSCRIPTION_ID:-}"
AD_APPLICATION_ID="${AD_APPLICATION_ID:-}"
AD_APPLICATION_SECRET="${AD_APPLICATION_SECRET:-}"
AD_APPLICATION_ID_ADMIN="${AD_APPLICATION_ID_ADMIN:-}"
DB_SERVER_NAME="${DB_SERVER_NAME:-}"
DB_NAME="${DB_NAME:-}"
KEY_VAULT="${KEY_VAULT:-}"
ACR_NAME="${ACR_NAME:-}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)                WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --resource-group)        RESOURCE_GROUP="$2"; shift 2 ;;
        --location)              LOCATION="$2"; shift 2 ;;
        --publisher-admin-users) PUBLISHER_ADMIN_USERS="$2"; shift 2 ;;
        --tenant-id)             TENANT_ID="$2"; shift 2 ;;
        --subscription)          AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --ad-app-id)             AD_APPLICATION_ID="$2"; shift 2 ;;
        --ad-app-secret)         AD_APPLICATION_SECRET="$2"; shift 2 ;;
        --ad-admin-app-id)       AD_APPLICATION_ID_ADMIN="$2"; shift 2 ;;
        --db-server)             DB_SERVER_NAME="$2"; shift 2 ;;
        --db-name)               DB_NAME="$2"; shift 2 ;;
        --key-vault)             KEY_VAULT="$2"; shift 2 ;;
        --acr)                   ACR_NAME="$2"; shift 2 ;;
        *) die "Unknown option: $1" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]]    && die "--prefix (WEB_APP_NAME_PREFIX) is required"
[[ -z "$LOCATION" ]]               && die "--location is required"
[[ -z "$PUBLISHER_ADMIN_USERS" ]]  && die "--publisher-admin-users is required"
[[ "${#WEB_APP_NAME_PREFIX}" -gt 21 ]] && die "Prefix must be ≤ 21 characters"

# ── derive resource names ─────────────────────────────────────────────────────
RESOURCE_GROUP="${RESOURCE_GROUP:-$WEB_APP_NAME_PREFIX}"
DB_SERVER_NAME="${DB_SERVER_NAME:-${WEB_APP_NAME_PREFIX}-db}"
DB_NAME="${DB_NAME:-${WEB_APP_NAME_PREFIX}AMPSaaSDB}"
KEY_VAULT="${KEY_VAULT:-${WEB_APP_NAME_PREFIX}-kv}"
ACR_NAME="${ACR_NAME:-${WEB_APP_NAME_PREFIX//-/}acr}"   # ACR names: alphanumeric only
APP_PLAN="${WEB_APP_NAME_PREFIX}-asp"
ADMIN_APP="${WEB_APP_NAME_PREFIX}-admin"
CUSTOMER_APP="${WEB_APP_NAME_PREFIX}-portal"
VNET_NAME="${WEB_APP_NAME_PREFIX}-vnet"
ADMIN_URL="https://${ADMIN_APP}.azurewebsites.net"
CUSTOMER_URL="https://${CUSTOMER_APP}.azurewebsites.net"

section "Deployment configuration"
echo "  Prefix:           $WEB_APP_NAME_PREFIX"
echo "  Resource group:   $RESOURCE_GROUP"
echo "  Location:         $LOCATION"
echo "  DB server:        $DB_SERVER_NAME"
echo "  DB name:          $DB_NAME"
echo "  Key Vault:        $KEY_VAULT"
echo "  ACR:              $ACR_NAME"
echo "  Admin URL:        $ADMIN_URL"
echo "  Customer URL:     $CUSTOMER_URL"
echo "  Publisher users:  $PUBLISHER_ADMIN_USERS"

# ── prerequisites ─────────────────────────────────────────────────────────────
section "Checking prerequisites"
for cmd in az jq; do
    command -v "$cmd" &>/dev/null || die "'$cmd' not found. Please install it."
done
info "Prerequisites OK."

# ── subscription ──────────────────────────────────────────────────────────────
if [[ -n "$AZURE_SUBSCRIPTION_ID" ]]; then
    info "Setting subscription: $AZURE_SUBSCRIPTION_ID"
    az account set --subscription "$AZURE_SUBSCRIPTION_ID"
fi
CURRENT_TENANT=$(az account show --query tenantId -o tsv)
TENANT_ID="${TENANT_ID:-$CURRENT_TENANT}"
info "Tenant: $TENANT_ID"

# ── App Registrations ─────────────────────────────────────────────────────────
section "App Registrations"

# 1) Fulfillment API app reg
if [[ -z "$AD_APPLICATION_ID" ]]; then
    EXISTING_FULFILL_ID=$(az ad app list \
        --display-name "${WEB_APP_NAME_PREFIX}-FulfillmentAppReg" \
        --query "[0].appId" -o tsv 2>/dev/null || true)
    if [[ -n "$EXISTING_FULFILL_ID" ]]; then
        info "  Reusing existing Fulfillment App Registration: $EXISTING_FULFILL_ID"
        AD_APPLICATION_ID="$EXISTING_FULFILL_ID"
        AD_OBJ_ID=$(az ad app show --id "$AD_APPLICATION_ID" --query id -o tsv)
    else
        info "Creating Fulfillment API App Registration..."
        AD_OBJ_ID=$(az ad app create \
            --only-show-errors \
            --sign-in-audience AzureADMyOrg \
            --display-name "${WEB_APP_NAME_PREFIX}-FulfillmentAppReg" \
            --query id -o tsv)
        AD_APPLICATION_ID=$(az ad app show --id "$AD_OBJ_ID" --query appId -o tsv)
    fi
    az ad sp create --id "$AD_APPLICATION_ID" --only-show-errors >/dev/null 2>&1 || true
    sleep 5
    AD_APPLICATION_SECRET=$(az ad app credential reset \
        --id "$AD_OBJ_ID" \
        --append \
        --display-name "SaaSAPI-$(date +%Y%m%d)" \
        --years 2 \
        --query password -o tsv --only-show-errors)
    info "  Fulfillment App ID: $AD_APPLICATION_ID"
else
    info "  Using provided Fulfillment App ID: $AD_APPLICATION_ID"
fi

# 2) Admin Portal SSO app reg
if [[ -z "$AD_APPLICATION_ID_ADMIN" ]]; then
    EXISTING_ADMIN_ID=$(az ad app list \
        --display-name "${WEB_APP_NAME_PREFIX}-AdminPortalAppReg" \
        --query "[0].appId" -o tsv 2>/dev/null || true)
    if [[ -n "$EXISTING_ADMIN_ID" ]]; then
        info "  Reusing existing Admin Portal App Registration: $EXISTING_ADMIN_ID"
        AD_APPLICATION_ID_ADMIN="$EXISTING_ADMIN_ID"
    else
        info "Creating Admin Portal SSO App Registration..."
        ADMIN_APP_REG_BODY=$(cat <<EOF
{
  "displayName": "${WEB_APP_NAME_PREFIX}-AdminPortalAppReg",
  "api": { "requestedAccessTokenVersion": 2 },
  "signInAudience": "AzureADMyOrg",
  "web": {
    "redirectUris": ["${ADMIN_URL}/auth/callback"],
    "logoutUrl": "${ADMIN_URL}/auth/logout",
    "implicitGrantSettings": { "enableIdTokenIssuance": true }
  },
  "requiredResourceAccess": [{
    "resourceAppId": "00000003-0000-0000-c000-000000000000",
    "resourceAccess": [{ "id": "e1fe6dd8-ba31-4d61-89e7-88639da4683d", "type": "Scope" }]
  }]
}
EOF
)
        AD_APPLICATION_ID_ADMIN=$(az rest \
            --method POST \
            --headers "Content-Type=application/json" \
            --uri "https://graph.microsoft.com/v1.0/applications" \
            --body "$ADMIN_APP_REG_BODY" \
            --query appId -o tsv)
        info "  Admin SSO App ID: $AD_APPLICATION_ID_ADMIN"
    fi
else
    info "  Using provided Admin SSO App ID: $AD_APPLICATION_ID_ADMIN"
fi

# ── Resource Group ────────────────────────────────────────────────────────────
section "Resource Group"
az group create --name "$RESOURCE_GROUP" --location "$LOCATION" -o none
info "Resource group ready: $RESOURCE_GROUP"

# ── VNet ──────────────────────────────────────────────────────────────────────
section "Virtual Network"
az network vnet create \
    --resource-group "$RESOURCE_GROUP" \
    --name "$VNET_NAME" \
    --address-prefixes "10.0.0.0/20" -o none 2>/dev/null || true
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n default \
    --address-prefixes "10.0.0.0/24" -o none 2>/dev/null || true
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n web \
    --address-prefixes "10.0.1.0/24" \
    --service-endpoints "Microsoft.KeyVault" \
    --delegations "Microsoft.Web/serverfarms" -o none 2>/dev/null || true
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n db \
    --address-prefixes "10.0.2.0/24" \
    --delegations "Microsoft.DBforPostgreSQL/flexibleServers" -o none 2>/dev/null || true
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n aci \
    --address-prefixes "10.0.3.0/24" \
    --delegations "Microsoft.ContainerInstance/containerGroups" -o none 2>/dev/null || true
info "VNet ready: $VNET_NAME"

# ── PostgreSQL Flexible Server ────────────────────────────────────────────────
section "PostgreSQL"
DB_ADMIN_USER="saasadmin"
DB_ADMIN_PASS="$(openssl rand -base64 32 | tr -d '/+=')Aa1!"
DB_SUBNET_ID=$(az network vnet subnet show \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n db \
    --query id -o tsv)
DB_PRIVATE_DNS_ZONE="${DB_SERVER_NAME}.private.postgres.database.azure.com"

az network private-dns zone create \
    --resource-group "$RESOURCE_GROUP" \
    --name "$DB_PRIVATE_DNS_ZONE" -o none 2>/dev/null || true
az network private-dns link vnet create \
    --resource-group "$RESOURCE_GROUP" \
    --zone-name "$DB_PRIVATE_DNS_ZONE" \
    --name "${DB_SERVER_NAME}-dns-link" \
    --virtual-network "$VNET_NAME" \
    --registration-enabled false -o none 2>/dev/null || true

az postgres flexible-server create \
    --resource-group "$RESOURCE_GROUP" \
    --name "$DB_SERVER_NAME" \
    --location "$LOCATION" \
    --admin-user "$DB_ADMIN_USER" \
    --admin-password "$DB_ADMIN_PASS" \
    --sku-name Standard_B1ms \
    --tier Burstable \
    --version 14 \
    --storage-size 32 \
    --subnet "$DB_SUBNET_ID" \
    --private-dns-zone "$DB_PRIVATE_DNS_ZONE" \
    --yes -o none 2>/dev/null || warn "PostgreSQL server may already exist"

az postgres flexible-server db create \
    --resource-group "$RESOURCE_GROUP" \
    --server-name "$DB_SERVER_NAME" \
    --database-name "$DB_NAME" -o none 2>/dev/null || true

DB_HOST="${DB_SERVER_NAME}.postgres.database.azure.com"
DATABASE_URL="postgresql://${DB_ADMIN_USER}:${DB_ADMIN_PASS}@${DB_HOST}:5432/${DB_NAME}?sslmode=require"
info "PostgreSQL ready: $DB_HOST / $DB_NAME"

# ── Key Vault ─────────────────────────────────────────────────────────────────
section "Key Vault"
# Purge any soft-deleted KV with the same name (happens on RG delete + recreate)
az keyvault purge --name "$KEY_VAULT" --location "$LOCATION" -o none 2>/dev/null || true

az keyvault create \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --location "$LOCATION" \
    --enable-rbac-authorization false -o none 2>/dev/null || warn "Key Vault may already exist"

# Temporarily open the firewall so we can set secrets (works on first run and re-runs)
az keyvault update \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --default-action Allow -o none

az keyvault secret set --vault-name "$KEY_VAULT" --name "DatabaseUrl"         --value "$DATABASE_URL" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADApplicationSecret" --value "$AD_APPLICATION_SECRET" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADApplicationId"     --value "$AD_APPLICATION_ID" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADAdminAppId"        --value "$AD_APPLICATION_ID_ADMIN" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "TenantId"            --value "$TENANT_ID" -o none

# Lock down: allow only VNet web subnet + Azure services (for App Service KV references)
az keyvault update \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --bypass AzureServices \
    --default-action Deny -o none
az keyvault network-rule add \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" \
    --subnet web -o none
info "Key Vault ready: $KEY_VAULT"

# ── Azure Container Registry ───────────────────────────────────────────────────
section "Azure Container Registry"
ACR_EXISTS=$(az acr show --name "$ACR_NAME" --resource-group "$RESOURCE_GROUP" \
    --query name -o tsv 2>/dev/null || true)
if [[ -z "$ACR_EXISTS" ]]; then
    info "Creating ACR: $ACR_NAME"
    az acr create \
        --resource-group "$RESOURCE_GROUP" \
        --name "$ACR_NAME" \
        --sku Basic \
        --location "$LOCATION" \
        --admin-enabled true -o none
else
    info "Using existing ACR: $ACR_NAME"
    az acr update --name "$ACR_NAME" --admin-enabled true -o none
fi

ACR_LOGIN_SERVER=$(az acr show --name "$ACR_NAME" --query loginServer -o tsv)
ACR_USER=$(az acr credential show --name "$ACR_NAME" --query username -o tsv)
ACR_PASS=$(az acr credential show --name "$ACR_NAME" --query passwords[0].value -o tsv)
info "ACR ready: $ACR_LOGIN_SERVER"

# ── App Service Plan ───────────────────────────────────────────────────────────
section "App Service Plan"
az appservice plan create \
    --name "$APP_PLAN" \
    --resource-group "$RESOURCE_GROUP" \
    --location "$LOCATION" \
    --is-linux \
    --sku B1 -o none 2>/dev/null || warn "App Service Plan may already exist"
info "App Service Plan ready: $APP_PLAN (B1 Linux)"

# ── Build and push Docker images ───────────────────────────────────────────────
section "Docker build and push"

BUILD_TAG="$(date +%Y%m%d%H%M%S)"
ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"

if docker info >/dev/null 2>&1; then
    info "Docker daemon detected — building locally..."
    az acr login --name "$ACR_NAME"

    docker build \
        --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
        --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
        -f "$SCRIPT_DIR/Dockerfile.admin-site" \
        -t "$ADMIN_IMAGE" -t "${ACR_LOGIN_SERVER}/admin-site:latest" "$REPO_ROOT"

    docker build \
        --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
        --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
        -f "$SCRIPT_DIR/Dockerfile.customer-site" \
        -t "$CUSTOMER_IMAGE" -t "${ACR_LOGIN_SERVER}/customer-site:latest" "$REPO_ROOT"

    docker build -f "$SCRIPT_DIR/Dockerfile.migrate" \
        -t "${ACR_LOGIN_SERVER}/migrate:latest" "$REPO_ROOT"

    docker push "$ADMIN_IMAGE"
    docker push "${ACR_LOGIN_SERVER}/admin-site:latest"
    docker push "$CUSTOMER_IMAGE"
    docker push "${ACR_LOGIN_SERVER}/customer-site:latest"
    docker push "${ACR_LOGIN_SERVER}/migrate:latest"
else
    info "No Docker daemon — using az acr build (remote build in ACR)..."

    az acr build \
        --registry "$ACR_NAME" \
        --image "admin-site:${BUILD_TAG}" \
        --image "admin-site:latest" \
        --file "$SCRIPT_DIR/Dockerfile.admin-site" \
        --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
        --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
        "$REPO_ROOT"

    az acr build \
        --registry "$ACR_NAME" \
        --image "customer-site:${BUILD_TAG}" \
        --image "customer-site:latest" \
        --file "$SCRIPT_DIR/Dockerfile.customer-site" \
        --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
        --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
        "$REPO_ROOT"

    az acr build \
        --registry "$ACR_NAME" \
        --image "migrate:latest" \
        --file "$SCRIPT_DIR/Dockerfile.migrate" \
        "$REPO_ROOT"
fi

info "Images ready (tag: $BUILD_TAG)"

# ── Database Migrations ────────────────────────────────────────────────────────
section "Database Migrations"
MIGRATE_CONTAINER="${WEB_APP_NAME_PREFIX}-migrate"

# Clean up any leftover from a previous run
az container delete \
    --resource-group "$RESOURCE_GROUP" \
    --name "$MIGRATE_CONTAINER" \
    --yes -o none 2>/dev/null || true

info "Running migrations via Container Instance in VNet..."
az container create \
    --resource-group "$RESOURCE_GROUP" \
    --name "$MIGRATE_CONTAINER" \
    --image "${ACR_LOGIN_SERVER}/migrate:latest" \
    --registry-login-server "$ACR_LOGIN_SERVER" \
    --registry-username "$ACR_USER" \
    --registry-password "$ACR_PASS" \
    --os-type Linux \
    --restart-policy Never \
    --vnet "$VNET_NAME" \
    --subnet aci \
    --secure-environment-variables "DATABASE_URL=${DATABASE_URL}" \
    -o none

info "Waiting for migrations to complete..."
for i in $(seq 1 30); do
    STATE=$(az container show \
        --resource-group "$RESOURCE_GROUP" \
        --name "$MIGRATE_CONTAINER" \
        --query "containers[0].instanceView.currentState.state" -o tsv 2>/dev/null || echo "Unknown")
    if [[ "$STATE" == "Terminated" ]]; then
        EXIT_CODE=$(az container show \
            --resource-group "$RESOURCE_GROUP" \
            --name "$MIGRATE_CONTAINER" \
            --query "containers[0].instanceView.currentState.exitCode" -o tsv)
        az container logs --resource-group "$RESOURCE_GROUP" --name "$MIGRATE_CONTAINER" 2>/dev/null || true
        az container delete --resource-group "$RESOURCE_GROUP" --name "$MIGRATE_CONTAINER" --yes -o none 2>/dev/null || true
        [[ "$EXIT_CODE" == "0" ]] || die "Migration failed (exit code $EXIT_CODE)"
        info "Migrations complete."
        break
    fi
    info "  State: $STATE (attempt $i/30)..."
    sleep 10
done

# ── Helper: whitelist Web App outbound IPs on Key Vault ───────────────────────
_webapp_outbound_ips() {
    az webapp show --name "$1" --resource-group "$2" \
        --query "possibleOutboundIpAddresses" -o tsv | tr ',' '\n' | sort -u
}

kv_whitelist_webapp_ips() {
    local app_name="$1" rg="$2"
    info "  Whitelisting $app_name outbound IPs on Key Vault $KEY_VAULT..."
    local count=0
    while IFS= read -r ip; do
        [[ -z "$ip" ]] && continue
        az keyvault network-rule add \
            --name "$KEY_VAULT" \
            --resource-group "$RESOURCE_GROUP" \
            --ip-address "${ip}/32" -o none 2>/dev/null && count=$((count+1))
    done <<< "$(_webapp_outbound_ips "$app_name" "$rg")"
    info "  Added $count IP rules for $app_name to Key Vault"
}

# ── Key Vault secret references ────────────────────────────────────────────────
KV_REF_DB="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=DatabaseUrl)"
KV_REF_SECRET="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADApplicationSecret)"
KV_REF_APPID="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADApplicationId)"
KV_REF_TENANT="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=TenantId)"
KV_REF_ADMIN_APPID="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADAdminAppId)"

# ── Helper: configure a web app ────────────────────────────────────────────────
configure_webapp() {
    local app_name="$1" image="$2" port="$3"
    shift 3
    # $@ = extra appsettings

    info "  Creating/updating $app_name..."
    az webapp create \
        --resource-group "$RESOURCE_GROUP" \
        --plan "$APP_PLAN" \
        --name "$app_name" \
        --deployment-container-image-name "$image" -o none 2>/dev/null || true

    # System-assigned managed identity for Key Vault access
    local identity
    identity=$(az webapp identity assign \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --query principalId -o tsv)

    # Wait for identity to propagate, then grant KV access
    info "  Waiting for identity propagation (up to 120s)..."
    for i in $(seq 1 12); do
        if az keyvault set-policy \
            --name "$KEY_VAULT" \
            --resource-group "$RESOURCE_GROUP" \
            --object-id "$identity" \
            --secret-permissions get list -o none 2>/dev/null; then
            break
        fi
        info "  Not propagated yet, retrying in 10s (attempt $i/12)..."
        sleep 10
    done

    # Registry credentials via well-known app settings (most reliable method)
    az webapp config appsettings set \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --settings \
            "DOCKER_REGISTRY_SERVER_URL=https://${ACR_LOGIN_SERVER}" \
            "DOCKER_REGISTRY_SERVER_USERNAME=${ACR_USER}" \
            "DOCKER_REGISTRY_SERVER_PASSWORD=${ACR_PASS}" \
            "WEBSITES_PORT=80" \
            "PORT=$port" \
            "RUST_LOG=info" \
            "WEBSITE_VNET_ROUTE_ALL=0" \
            "$@" -o none

    # Set container image explicitly
    az webapp config set \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --linux-fx-version "DOCKER|${image}" -o none

    # Harden
    az webapp update \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --https-only true -o none
    az webapp config set \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --always-on true \
        --http20-enabled true \
        --min-tls-version "1.2" -o none

    # VNet integration
    az webapp vnet-integration add \
        --name "$app_name" \
        --resource-group "$RESOURCE_GROUP" \
        --vnet "$VNET_NAME" \
        --subnet web -o none 2>/dev/null || true

    # Whitelist outbound IPs on Key Vault
    kv_whitelist_webapp_ips "$app_name" "$RESOURCE_GROUP"
}

# ── Admin Web App ──────────────────────────────────────────────────────────────
section "Admin Web App"
configure_webapp "$ADMIN_APP" "$ADMIN_IMAGE" "3000" \
    "DATABASE_URL=${KV_REF_DB}" \
    "SaaS_API_CLIENT_ID=${KV_REF_APPID}" \
    "SaaS_API_CLIENT_SECRET=${KV_REF_SECRET}" \
    "SaaS_API_TENANT_ID=${KV_REF_TENANT}" \
    "AZURE_AD_TENANT_ID=${KV_REF_TENANT}" \
    "AZURE_AD_CLIENT_ID=${KV_REF_ADMIN_APPID}" \
    "AZURE_AD_CLIENT_SECRET=${KV_REF_SECRET}" \
    "AZURE_AD_REDIRECT_URI=${ADMIN_URL}/auth/callback" \
    "AZURE_AD_SIGNED_OUT_REDIRECT_URI=${ADMIN_URL}/admin" \
    "MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api" \
    "MARKETPLACE_API_VERSION=2018-08-31" \
    "KNOWN_USERS=${PUBLISHER_ADMIN_USERS}"
info "Admin Web App ready: $ADMIN_URL"

# ── Customer Web App ───────────────────────────────────────────────────────────
section "Customer Web App"
configure_webapp "$CUSTOMER_APP" "$CUSTOMER_IMAGE" "3001" \
    "DATABASE_URL=${KV_REF_DB}" \
    "SaaS_API_CLIENT_ID=${KV_REF_APPID}" \
    "SaaS_API_CLIENT_SECRET=${KV_REF_SECRET}" \
    "SaaS_API_TENANT_ID=${KV_REF_TENANT}" \
    "MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api" \
    "MARKETPLACE_API_VERSION=2018-08-31"
info "Customer Web App ready: $CUSTOMER_URL"

# ── DB Migrations ──────────────────────────────────────────────────────────────
section "Database migrations"
if command -v sqlx &>/dev/null; then
    info "Running sqlx migrations..."
    (cd "$REPO_ROOT/crates/data" && DATABASE_URL="$DATABASE_URL" sqlx migrate run)
    info "Migrations complete."
else
    warn "sqlx CLI not found — run migrations manually:"
    warn "  cd crates/data && DATABASE_URL='$DATABASE_URL' sqlx migrate run"
fi

# ── Seed publisher admin users ─────────────────────────────────────────────────
section "Seeding publisher admin users"
IFS=',' read -ra ADMIN_EMAILS <<< "$PUBLISHER_ADMIN_USERS"
for EMAIL in "${ADMIN_EMAILS[@]}"; do
    EMAIL="$(echo "$EMAIL" | tr -d '[:space:]')"
    [[ -z "$EMAIL" ]] && continue
    if command -v psql &>/dev/null; then
        psql "$DATABASE_URL" -c \
            "INSERT INTO known_users (user_email, role_id) VALUES ('${EMAIL}', 1) ON CONFLICT (user_email) DO NOTHING;" \
            -q 2>/dev/null && info "  Seeded: $EMAIL" || \
            warn "  Could not seed $EMAIL (psql error or not connected)"
    else
        warn "  psql not found — add $EMAIL to known_users manually (role_id=1)"
    fi
done

# ── Print summary ──────────────────────────────────────────────────────────────
section "Deployment complete"
echo ""
echo -e "${GREEN}✅  Resources created in resource group: ${RESOURCE_GROUP}${NC}"
echo ""
echo "  Admin portal:    $ADMIN_URL"
echo "  Customer portal: $CUSTOMER_URL"
echo "  ACR:             $ACR_LOGIN_SERVER"
echo "  Key Vault:       $KEY_VAULT"
echo "  PostgreSQL:      $DB_HOST"
echo ""
echo -e "${CYAN}▶  Next steps:${NC}"
echo "  1. In Azure AD → App Registration '${WEB_APP_NAME_PREFIX}-AdminPortalAppReg',"
echo "     confirm redirect URI:  ${ADMIN_URL}/auth/callback"
echo "  2. In Partner Center → Technical Configuration:"
echo "     Landing Page:          ${CUSTOMER_URL}/"
echo "     Webhook:               ${CUSTOMER_URL}/api/webhook"
echo "     Tenant ID:             ${TENANT_ID}"
echo "     AAD Application ID:    ${AD_APPLICATION_ID}"
echo "  3. Run DB migrations — from inside the VNet or via ACR task:"
echo "     DATABASE_URL='${DATABASE_URL}' sqlx migrate run --source crates/data/migrations"
echo "  4. Verify Known Users: ${ADMIN_URL}/admin/known-users"
echo ""
echo "  Log file: $DEPLOY_LOG"
echo ""
echo -e "${YELLOW}DO NOT CLOSE — copy the values above before continuing.${NC}"
