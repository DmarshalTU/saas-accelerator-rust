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

# ── prerequisites ─────────────────────────────────────────────────────────────
check_prereqs() {
    section "Checking prerequisites"
    for cmd in az docker jq curl; do
        command -v "$cmd" &>/dev/null || die "'$cmd' not found. Please install it."
    done
    info "All prerequisites found."
}

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
AD_APPLICATION_ID="${AD_APPLICATION_ID:-}"          # Fulfillment API app reg (optional, created if empty)
AD_APPLICATION_SECRET="${AD_APPLICATION_SECRET:-}"
AD_APPLICATION_ID_ADMIN="${AD_APPLICATION_ID_ADMIN:-}" # Admin SSO app reg (optional, created if empty)
DB_SERVER_NAME="${DB_SERVER_NAME:-}"
DB_NAME="${DB_NAME:-}"
KEY_VAULT="${KEY_VAULT:-}"
ACR_NAME="${ACR_NAME:-}"
ACR_SUBSCRIPTION="${ACR_SUBSCRIPTION:-}"   # Set if ACR is in a different subscription
LOGO_URL_PNG="${LOGO_URL_PNG:-}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)               WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --resource-group)       RESOURCE_GROUP="$2"; shift 2 ;;
        --location)             LOCATION="$2"; shift 2 ;;
        --publisher-admin-users) PUBLISHER_ADMIN_USERS="$2"; shift 2 ;;
        --tenant-id)            TENANT_ID="$2"; shift 2 ;;
        --subscription)         AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --ad-app-id)            AD_APPLICATION_ID="$2"; shift 2 ;;
        --ad-app-secret)        AD_APPLICATION_SECRET="$2"; shift 2 ;;
        --ad-admin-app-id)      AD_APPLICATION_ID_ADMIN="$2"; shift 2 ;;
        --db-server)            DB_SERVER_NAME="$2"; shift 2 ;;
        --db-name)              DB_NAME="$2"; shift 2 ;;
        --key-vault)            KEY_VAULT="$2"; shift 2 ;;
        --acr)                  ACR_NAME="$2"; shift 2 ;;
        --acr-subscription)     ACR_SUBSCRIPTION="$2"; shift 2 ;;
        *) die "Unknown option: $1" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]] && die "--prefix (WEB_APP_NAME_PREFIX) is required"
[[ -z "$LOCATION" ]]            && die "--location is required"
[[ -z "$PUBLISHER_ADMIN_USERS" ]] && die "--publisher-admin-users is required"
[[ "${#WEB_APP_NAME_PREFIX}" -gt 21 ]] && die "Prefix must be ≤ 21 characters"

# ── derive resource names (mirrors Deploy.ps1 naming) ────────────────────────
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

check_prereqs

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

# 1) Fulfillment API app reg (single-tenant, client credentials)
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
        info "  Fulfillment App ID: $AD_APPLICATION_ID"
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

# 2) Admin Portal SSO app reg (single-tenant, id_token, openid+profile+email)
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
    "redirectUris": [
      "${ADMIN_URL}/auth/callback"
    ],
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
    fi  # end else (not existing)
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
    --address-prefixes "10.0.0.0/20" -o none
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n default \
    --address-prefixes "10.0.0.0/24" -o none
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n web \
    --address-prefixes "10.0.1.0/24" \
    --service-endpoints "Microsoft.KeyVault" \
    --delegations "Microsoft.Web/serverfarms" -o none
az network vnet subnet create \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n db \
    --address-prefixes "10.0.2.0/24" \
    --delegations "Microsoft.DBforPostgreSQL/flexibleServers" -o none
info "VNet ready: $VNET_NAME (10.0.0.0/20)"

# ── PostgreSQL Flexible Server ────────────────────────────────────────────────
section "PostgreSQL"
DB_ADMIN_USER="saasadmin"
DB_ADMIN_PASS="$(openssl rand -base64 32 | tr -d '/+=')Aa1!"   # meets Azure complexity rules
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
az keyvault create \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --location "$LOCATION" \
    --enable-rbac-authorization false -o none 2>/dev/null || warn "Key Vault may already exist"

az keyvault secret set --vault-name "$KEY_VAULT" --name "DatabaseUrl"          --value "$DATABASE_URL" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADApplicationSecret"  --value "$AD_APPLICATION_SECRET" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADApplicationId"      --value "$AD_APPLICATION_ID" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "ADAdminAppId"         --value "$AD_APPLICATION_ID_ADMIN" -o none
az keyvault secret set --vault-name "$KEY_VAULT" --name "TenantId"             --value "$TENANT_ID" -o none

# Restrict KV to VNet web subnet
WEB_SUBNET_ID=$(az network vnet subnet show \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" -n web \
    --query id -o tsv)
az keyvault update \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --default-action Deny -o none
az keyvault network-rule add \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --vnet-name "$VNET_NAME" \
    --subnet web -o none
info "Key Vault ready: $KEY_VAULT"

# ── Azure Container Registry ───────────────────────────────────────────────────
section "Azure Container Registry"

# Helper: run an az command optionally scoped to a different subscription.
acr_az() { 
    if [[ -n "$ACR_SUBSCRIPTION" ]]; then
        az --subscription "$ACR_SUBSCRIPTION" "$@"
    else
        az "$@"
    fi
}

ACR_EXISTS=$(acr_az acr show --name "$ACR_NAME" --query name -o tsv 2>/dev/null || true)
if [[ -z "$ACR_EXISTS" ]]; then
    info "Creating ACR: $ACR_NAME (Basic, admin disabled)"
    acr_az acr create \
        --resource-group "$RESOURCE_GROUP" \
        --name "$ACR_NAME" \
        --sku Basic \
        --location "$LOCATION" \
        --admin-enabled false -o none
else
    info "Using existing ACR: $ACR_NAME (subscription: ${ACR_SUBSCRIPTION:-current})"
fi

ACR_LOGIN_SERVER=$(acr_az acr show --name "$ACR_NAME" --query loginServer -o tsv)
ACR_ID=$(acr_az acr show --name "$ACR_NAME" --query id -o tsv)
info "ACR: $ACR_LOGIN_SERVER  (id: $ACR_ID)"

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
# az acr login uses your Azure AD token — works with network-restricted ACRs
# (your machine must be on an allowed network or have a service endpoint).
info "Logging in to ACR via Azure AD token..."
az acr login --name "$ACR_NAME"

BUILD_TAG="$(date +%Y%m%d%H%M%S)"
ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"
ADMIN_LATEST="${ACR_LOGIN_SERVER}/admin-site:latest"
CUSTOMER_LATEST="${ACR_LOGIN_SERVER}/customer-site:latest"

info "Building admin-site image (tag: $BUILD_TAG)..."
DOCKER_BUILDKIT=1 docker build \
    --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
    --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
    -f "$SCRIPT_DIR/Dockerfile.admin-site" \
    -t "$ADMIN_IMAGE" -t "$ADMIN_LATEST" "$REPO_ROOT"

info "Building customer-site image (tag: $BUILD_TAG)..."
DOCKER_BUILDKIT=1 docker build \
    --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
    --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
    -f "$SCRIPT_DIR/Dockerfile.customer-site" \
    -t "$CUSTOMER_IMAGE" -t "$CUSTOMER_LATEST" "$REPO_ROOT"

info "Pushing images to ACR..."
docker push "$ADMIN_IMAGE" && docker push "$ADMIN_LATEST"
docker push "$CUSTOMER_IMAGE" && docker push "$CUSTOMER_LATEST"
info "Images pushed (tag: $BUILD_TAG)"

# ── ACR network: whitelist Web App outbound IPs ───────────────────────────────
# Adds a Web App's outbound IPs to the ACR's network allowlist.
# Required when the ACR is network-restricted (selected networks) — the Web App
# must be able to reach the ACR endpoint to pull images at startup.
acr_whitelist_webapp_ips() {
    local app_name="$1"
    local rg="$2"
    info "  Fetching outbound IPs for $app_name..."
    local ips
    ips=$(az webapp show --name "$app_name" --resource-group "$rg" \
        --query "[possibleOutboundIpAddresses]" -o tsv | tr ',' '\n' | sort -u)
    local count=0
    while IFS= read -r ip; do
        [[ -z "$ip" ]] && continue
        acr_az acr network-rule add \
            --name "$ACR_NAME" \
            --ip-address "${ip}/32" -o none 2>/dev/null && count=$((count+1))
    done <<< "$ips"
    info "  Added $count IP rules for $app_name to ACR $ACR_NAME"
}

# ── Key Vault secret references ────────────────────────────────────────────────
KV_REF_DB="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=DatabaseUrl) "
KV_REF_SECRET="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADApplicationSecret) "
KV_REF_APPID="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADApplicationId) "
KV_REF_TENANT="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=TenantId) "
KV_REF_ADMIN_APPID="@Microsoft.KeyVault(VaultName=${KEY_VAULT};SecretName=ADAdminAppId) "

# ── Admin Web App ──────────────────────────────────────────────────────────────
section "Admin Web App"
az webapp create \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_PLAN" \
    --name "$ADMIN_APP" \
    --deployment-container-image-name "$ADMIN_IMAGE" -o none 2>/dev/null || \
    warn "Admin Web App may already exist"

# Assign system identity first (needed for both KV access and ACR pull)
ADMIN_IDENTITY=$(az webapp identity assign \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --identities "[system]" \
    --query principalId -o tsv)

# Grant AcrPull on ACR — cross-subscription works because we use the full resource ID scope
acr_az role assignment create \
    --assignee "$ADMIN_IDENTITY" \
    --role AcrPull \
    --scope "$ACR_ID" -o none

# Grant Key Vault read access
az keyvault set-policy \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --object-id "$ADMIN_IDENTITY" \
    --secret-permissions get list -o none

# Configure container — no username/password, identity does the pull
az webapp config container set \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --docker-custom-image-name "$ADMIN_IMAGE" \
    --docker-registry-server-url "https://${ACR_LOGIN_SERVER}" -o none

# Enable Managed Identity ACR pull
az webapp config set \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --generic-configurations '{"acrUseManagedIdentityCreds": true}' -o none 2>/dev/null || \
az resource update \
    --ids "$(az webapp show --name "$ADMIN_APP" --resource-group "$RESOURCE_GROUP" --query id -o tsv)/config/web" \
    --set properties.acrUseManagedIdentityCreds=true -o none

# App settings (using Key Vault references for secrets)
az webapp config appsettings set \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --settings \
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
        "KNOWN_USERS=${PUBLISHER_ADMIN_USERS}" \
        "RUST_LOG=info" \
        "PORT=3000" -o none

# Harden: HTTPS-only, always-on, HTTP/2
az webapp update \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --https-only true -o none
az webapp config set \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --always-on true \
    --http20-enabled true \
    --min-tls-version "1.2" -o none

# VNet integration
az webapp vnet-integration add \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --vnet "$VNET_NAME" \
    --subnet web -o none

# Whitelist Admin Web App outbound IPs on the ACR network rules
acr_whitelist_webapp_ips "$ADMIN_APP" "$RESOURCE_GROUP"

info "Admin Web App ready: $ADMIN_URL"

# ── Customer Web App ───────────────────────────────────────────────────────────
section "Customer Web App"
az webapp create \
    --resource-group "$RESOURCE_GROUP" \
    --plan "$APP_PLAN" \
    --name "$CUSTOMER_APP" \
    --deployment-container-image-name "$CUSTOMER_IMAGE" -o none 2>/dev/null || \
    warn "Customer Web App may already exist"

CUSTOMER_IDENTITY=$(az webapp identity assign \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --identities "[system]" \
    --query principalId -o tsv)

acr_az role assignment create \
    --assignee "$CUSTOMER_IDENTITY" \
    --role AcrPull \
    --scope "$ACR_ID" -o none

az keyvault set-policy \
    --name "$KEY_VAULT" \
    --resource-group "$RESOURCE_GROUP" \
    --object-id "$CUSTOMER_IDENTITY" \
    --secret-permissions get list -o none

az webapp config container set \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --docker-custom-image-name "$CUSTOMER_IMAGE" \
    --docker-registry-server-url "https://${ACR_LOGIN_SERVER}" -o none

az webapp config set \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --generic-configurations '{"acrUseManagedIdentityCreds": true}' -o none 2>/dev/null || \
az resource update \
    --ids "$(az webapp show --name "$CUSTOMER_APP" --resource-group "$RESOURCE_GROUP" --query id -o tsv)/config/web" \
    --set properties.acrUseManagedIdentityCreds=true -o none

az webapp config appsettings set \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --settings \
        "DATABASE_URL=${KV_REF_DB}" \
        "SaaS_API_CLIENT_ID=${KV_REF_APPID}" \
        "SaaS_API_CLIENT_SECRET=${KV_REF_SECRET}" \
        "SaaS_API_TENANT_ID=${KV_REF_TENANT}" \
        "MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api" \
        "MARKETPLACE_API_VERSION=2018-08-31" \
        "RUST_LOG=info" \
        "PORT=3001" -o none

az webapp update \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --https-only true -o none
az webapp config set \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --always-on true \
    --http20-enabled true \
    --min-tls-version "1.2" -o none

az webapp vnet-integration add \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --vnet "$VNET_NAME" \
    --subnet web -o none

# Whitelist Customer Web App outbound IPs on the ACR network rules
acr_whitelist_webapp_ips "$CUSTOMER_APP" "$RESOURCE_GROUP"

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
            -q 2>/dev/null && info "  Seeded: $EMAIL" || warn "  Could not seed $EMAIL (psql error or not connected)"
    else
        warn "  psql not found — add $EMAIL to known_users manually (role_id=1)"
    fi
done

# ── Print summary ──────────────────────────────────────────────────────────────
section "Deployment complete"
echo ""
echo -e "${GREEN}✅  Resources created in resource group: ${RESOURCE_GROUP}${NC}"
echo ""
echo "  Admin portal:       $ADMIN_URL"
echo "  Customer portal:    $CUSTOMER_URL"
echo "  ACR:                $ACR_LOGIN_SERVER"
echo "  Key Vault:          $KEY_VAULT"
echo "  PostgreSQL:         $DB_HOST"
echo ""
echo -e "${CYAN}▶  Next steps:${NC}"
echo "  1. In Azure AD → App Registration '${WEB_APP_NAME_PREFIX}-AdminPortalAppReg',"
echo "     confirm redirect URI:  ${ADMIN_URL}/auth/callback"
echo "  2. In Partner Center → Technical Configuration:"
echo "     Landing Page:          ${CUSTOMER_URL}/"
echo "     Webhook:               ${CUSTOMER_URL}/api/webhook"
echo "     Tenant ID:             ${TENANT_ID}"
echo "     AAD Application ID:    ${AD_APPLICATION_ID}"
echo "  3. Verify Known Users in Admin UI: ${ADMIN_URL}/admin/known-users"
echo ""
echo "  Log file: $DEPLOY_LOG"
echo ""
echo -e "${YELLOW}DO NOT CLOSE — copy the values above before continuing.${NC}"
