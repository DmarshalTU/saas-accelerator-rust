#!/usr/bin/env bash
# =============================================================================
# SaaS Accelerator – Rust Edition  |  deploy.sh
# =============================================================================
# Idempotent first-run deployment.  Safe to re-run — every section checks
# whether resources already exist before creating them.
#
# Usage:
#   ./deployment/deploy.sh --prefix myco --location "East US" \
#     --publisher-admin-users "admin@myco.com"
#
# Key speed options (re-run scenarios):
#   --skip-appregistrations   Skip Azure AD app reg creation
#   --skip-network            Skip VNet/subnet creation
#   --skip-db                 Skip PostgreSQL creation/update
#   --skip-kv                 Skip Key Vault creation/update
#   --skip-acr                Skip ACR creation/cache-rule setup
#   --skip-build              Reuse the most-recent images in ACR (no Docker build)
#   --skip-migrations         Skip sqlx migrate run
#   --skip-webapps            Skip web-app creation/config update
#   --skip-hardening          Skip security hardening step
#
# Cache options (add to .env or pass as flags):
#   DOCKERHUB_USERNAME / DOCKERHUB_TOKEN   Enables ACR pull-through cache for
#                                          base images (rust, node, debian).
#                                          Credentials are stored in Key Vault.
# =============================================================================
set -euo pipefail

# ── Colours and utilities ─────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$SCRIPT_DIR/deploy-$(date +%Y%m%d-%H%M%S).log"
echo "=== deploy.sh  $(date) ===" > "$LOG_FILE"

log()     { echo "$*" >> "$LOG_FILE"; }
info()    { echo -e "${GREEN}  ✓${NC} $*"; log "[INFO] $*"; }
warn()    { echo -e "${YELLOW}  ⚠${NC} $*"; log "[WARN] $*"; }
error()   { echo -e "${RED}  ✗${NC} $*"; log "[ERROR] $*"; }
die()     { error "$*"; exit 1; }

_SECTION_TIMES=()
section() {
    local name="$1"
    echo -e "\n${CYAN}${BOLD}══ $name ══${NC}"
    log "\n=== $name ==="
    _SECTION_START=$(date +%s)
}
section_done() {
    local elapsed=$(( $(date +%s) - _SECTION_START ))
    echo -e "   ${GREEN}done in ${elapsed}s${NC}"
}

trap 'error "Failed at line $LINENO"; error "Log: $LOG_FILE"' ERR

# ── Load .env ─────────────────────────────────────────────────────────────────
[[ -f "$SCRIPT_DIR/.env" ]] && { set -a; source "$SCRIPT_DIR/.env"; set +a; }

# ── Parameters ────────────────────────────────────────────────────────────────
WEB_APP_NAME_PREFIX="${WEB_APP_NAME_PREFIX:-}"
LOCATION="${LOCATION:-}"
PUBLISHER_ADMIN_USERS="${PUBLISHER_ADMIN_USERS:-}"
RESOURCE_GROUP="${RESOURCE_GROUP:-}"
AZURE_SUBSCRIPTION_ID="${AZURE_SUBSCRIPTION_ID:-}"
TENANT_ID="${TENANT_ID:-}"
AD_APPLICATION_ID="${AD_APPLICATION_ID:-}"
AD_APPLICATION_SECRET="${AD_APPLICATION_SECRET:-}"
AD_APPLICATION_ID_ADMIN="${AD_APPLICATION_ID_ADMIN:-}"
DB_SERVER_NAME="${DB_SERVER_NAME:-}"
DB_NAME="${DB_NAME:-}"
KEY_VAULT="${KEY_VAULT:-}"
ACR_NAME="${ACR_NAME:-}"
ACR_SUBSCRIPTION="${ACR_SUBSCRIPTION:-}"
DOCKERHUB_USERNAME="${DOCKERHUB_USERNAME:-}"
DOCKERHUB_TOKEN="${DOCKERHUB_TOKEN:-}"

# Skip flags
SKIP_APPREGISTRATIONS=false
SKIP_NETWORK=false
SKIP_DB=false
SKIP_KV=false
SKIP_ACR=false
SKIP_BUILD=false
SKIP_MIGRATIONS=false
SKIP_WEBAPPS=false
SKIP_HARDENING=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)               WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --location)             LOCATION="$2"; shift 2 ;;
        --publisher-admin-users) PUBLISHER_ADMIN_USERS="$2"; shift 2 ;;
        --resource-group)       RESOURCE_GROUP="$2"; shift 2 ;;
        --subscription)         AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --tenant-id)            TENANT_ID="$2"; shift 2 ;;
        --ad-app-id)            AD_APPLICATION_ID="$2"; shift 2 ;;
        --ad-app-secret)        AD_APPLICATION_SECRET="$2"; shift 2 ;;
        --ad-admin-app-id)      AD_APPLICATION_ID_ADMIN="$2"; shift 2 ;;
        --db-server)            DB_SERVER_NAME="$2"; shift 2 ;;
        --db-name)              DB_NAME="$2"; shift 2 ;;
        --key-vault)            KEY_VAULT="$2"; shift 2 ;;
        --acr)                  ACR_NAME="$2"; shift 2 ;;
        --acr-subscription)     ACR_SUBSCRIPTION="$2"; shift 2 ;;
        --dockerhub-username)   DOCKERHUB_USERNAME="$2"; shift 2 ;;
        --dockerhub-token)      DOCKERHUB_TOKEN="$2"; shift 2 ;;
        --skip-appregistrations) SKIP_APPREGISTRATIONS=true; shift ;;
        --skip-network)         SKIP_NETWORK=true; shift ;;
        --skip-db)              SKIP_DB=true; shift ;;
        --skip-kv)              SKIP_KV=true; shift ;;
        --skip-acr)             SKIP_ACR=true; shift ;;
        --skip-build)           SKIP_BUILD=true; shift ;;
        --skip-migrations)      SKIP_MIGRATIONS=true; shift ;;
        --skip-webapps)         SKIP_WEBAPPS=true; shift ;;
        --skip-hardening)       SKIP_HARDENING=true; shift ;;
        *) die "Unknown option: $1 (run with --help for usage)" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]]   && die "--prefix is required"
[[ -z "$LOCATION" ]]              && die "--location is required"
[[ -z "$PUBLISHER_ADMIN_USERS" ]] && die "--publisher-admin-users is required"
[[ "${#WEB_APP_NAME_PREFIX}" -gt 21 ]] && die "Prefix must be ≤ 21 characters"

# ── Derived names ─────────────────────────────────────────────────────────────
P="$WEB_APP_NAME_PREFIX"
RESOURCE_GROUP="${RESOURCE_GROUP:-$P}"
DB_SERVER_NAME="${DB_SERVER_NAME:-${P}-db}"
DB_NAME="${DB_NAME:-${P}AMPSaaSDB}"
KEY_VAULT="${KEY_VAULT:-${P}-kv}"
ACR_NAME="${ACR_NAME:-${P//-/}acr}"
APP_PLAN="${P}-rsp"
ADMIN_APP="${P}-admin"
CUSTOMER_APP="${P}-portal"
VNET_NAME="${P}-vnet"
ADMIN_URL="https://${ADMIN_APP}.azurewebsites.net"
CUSTOMER_URL="https://${CUSTOMER_APP}.azurewebsites.net"
DB_HOST="${DB_SERVER_NAME}.postgres.database.azure.com"
DB_ADMIN_USER="saasadmin"
KV_URL="https://${KEY_VAULT}.vault.azure.net"

# Helper: run az commands in the ACR's subscription if it differs
acr_az() {
    if [[ -n "$ACR_SUBSCRIPTION" ]]; then
        az --subscription "$ACR_SUBSCRIPTION" "$@"
    else
        az "$@"
    fi
}

echo ""
echo -e "${BOLD}SaaS Accelerator – deploy.sh${NC}"
echo "  Prefix:      $P"
echo "  Tenant:      ${TENANT_ID:-<current>}"
echo "  Location:    $LOCATION"
echo "  Admin URL:   $ADMIN_URL"
echo "  Customer URL:$CUSTOMER_URL"
echo "  Log:         $LOG_FILE"
echo ""

# Prerequisites
for cmd in az docker jq curl; do
    command -v "$cmd" &>/dev/null || die "Required tool not found: $cmd"
done
[[ -n "$AZURE_SUBSCRIPTION_ID" ]] && az account set --subscription "$AZURE_SUBSCRIPTION_ID"
TENANT_ID="${TENANT_ID:-$(az account show --query tenantId -o tsv)}"
info "Tenant: $TENANT_ID"

# =============================================================================
# 1. APP REGISTRATIONS
# =============================================================================
if [[ "$SKIP_APPREGISTRATIONS" == "true" ]]; then
    warn "Skipping App Registrations (--skip-appregistrations)"
else
    section "App Registrations"

    if [[ -z "$AD_APPLICATION_ID" ]]; then
        EXISTING=$(az ad app list --display-name "${P}-FulfillmentAppReg" --query "[0].appId" -o tsv 2>/dev/null || true)
        if [[ -n "$EXISTING" ]]; then
            info "Fulfillment App Reg exists: $EXISTING"
            AD_APPLICATION_ID="$EXISTING"
            AD_OBJ_ID=$(az ad app show --id "$AD_APPLICATION_ID" --query id -o tsv)
        else
            info "Creating Fulfillment API App Registration..."
            AD_OBJ_ID=$(az ad app create --only-show-errors --sign-in-audience AzureADMyOrg \
                --display-name "${P}-FulfillmentAppReg" --query id -o tsv)
            AD_APPLICATION_ID=$(az ad app show --id "$AD_OBJ_ID" --query appId -o tsv)
            az ad sp create --id "$AD_APPLICATION_ID" --only-show-errors >/dev/null 2>&1 || true
            sleep 5
            AD_APPLICATION_SECRET=$(az ad app credential reset --id "$AD_OBJ_ID" --append \
                --display-name "SaaSAPI-$(date +%Y%m%d)" --years 2 \
                --query password -o tsv --only-show-errors)
            info "Fulfillment App: $AD_APPLICATION_ID"
        fi
    else
        info "Using provided Fulfillment App: $AD_APPLICATION_ID"
    fi

    if [[ -z "$AD_APPLICATION_ID_ADMIN" ]]; then
        EXISTING=$(az ad app list --display-name "${P}-AdminPortalAppReg" --query "[0].appId" -o tsv 2>/dev/null || true)
        if [[ -n "$EXISTING" ]]; then
            info "Admin SSO App Reg exists: $EXISTING"
            AD_APPLICATION_ID_ADMIN="$EXISTING"
        else
            info "Creating Admin Portal SSO App Registration..."
            BODY=$(cat <<EOF
{"displayName":"${P}-AdminPortalAppReg","api":{"requestedAccessTokenVersion":2},
"signInAudience":"AzureADMyOrg","web":{"redirectUris":["${ADMIN_URL}/auth/callback"],
"logoutUrl":"${ADMIN_URL}/auth/logout","implicitGrantSettings":{"enableIdTokenIssuance":true}},
"requiredResourceAccess":[{"resourceAppId":"00000003-0000-0000-c000-000000000000",
"resourceAccess":[{"id":"e1fe6dd8-ba31-4d61-89e7-88639da4683d","type":"Scope"}]}]}
EOF
)
            AD_APPLICATION_ID_ADMIN=$(az rest --method POST \
                --headers "Content-Type=application/json" \
                --uri "https://graph.microsoft.com/v1.0/applications" \
                --body "$BODY" --query appId -o tsv)
            info "Admin SSO App: $AD_APPLICATION_ID_ADMIN"
        fi
    else
        info "Using provided Admin SSO App: $AD_APPLICATION_ID_ADMIN"
    fi

    section_done
fi

# =============================================================================
# 2. RESOURCE GROUP
# =============================================================================
section "Resource Group"
if az group show -n "$RESOURCE_GROUP" -o none 2>/dev/null; then
    info "Already exists: $RESOURCE_GROUP"
else
    az group create --name "$RESOURCE_GROUP" --location "$LOCATION" -o none
    info "Created: $RESOURCE_GROUP"
fi
section_done

# =============================================================================
# 3. VIRTUAL NETWORK
# =============================================================================
if [[ "$SKIP_NETWORK" == "true" ]]; then
    warn "Skipping VNet (--skip-network)"
else
    section "Virtual Network"
    _create_subnet() {
        local name="$1" cidr="$2"; shift 2
        if az network vnet subnet show -g "$RESOURCE_GROUP" --vnet-name "$VNET_NAME" -n "$name" -o none 2>/dev/null; then
            info "Subnet exists: $name"
        else
            az network vnet subnet create -g "$RESOURCE_GROUP" --vnet-name "$VNET_NAME" \
                -n "$name" --address-prefixes "$cidr" "$@" -o none
            info "Subnet created: $name ($cidr)"
        fi
    }
    if ! az network vnet show -g "$RESOURCE_GROUP" -n "$VNET_NAME" -o none 2>/dev/null; then
        az network vnet create -g "$RESOURCE_GROUP" -n "$VNET_NAME" --address-prefixes "10.0.0.0/20" -o none
        info "VNet created: $VNET_NAME"
    else
        info "VNet exists: $VNET_NAME"
    fi
    _create_subnet default "10.0.0.0/24"
    _create_subnet web    "10.0.1.0/24" \
        --service-endpoints "Microsoft.KeyVault" --delegations "Microsoft.Web/serverfarms"
    _create_subnet db     "10.0.2.0/24" \
        --delegations "Microsoft.DBforPostgreSQL/flexibleServers"
    _create_subnet aci    "10.0.3.0/24" \
        --delegations "Microsoft.ContainerInstance/containerGroups"
    section_done
fi

# =============================================================================
# 4. POSTGRESQL
# =============================================================================
if [[ "$SKIP_DB" == "true" ]]; then
    warn "Skipping PostgreSQL (--skip-db)"
    # Try to read password from KV for later use
    DATABASE_URL=$(az keyvault secret show --vault-name "$KEY_VAULT" \
        --name "DatabasePassword" --query value -o tsv 2>/dev/null || true)
    DB_ADMIN_PASS="$DATABASE_URL"  # just a placeholder; will read properly below
else
    section "PostgreSQL"
    # Re-use existing password so existing connections aren't broken
    DB_ADMIN_PASS=$(az keyvault secret show --vault-name "$KEY_VAULT" \
        --name "DatabasePassword" --query value -o tsv 2>/dev/null || true)
    if [[ -z "$DB_ADMIN_PASS" ]]; then
        DB_ADMIN_PASS="$(openssl rand -hex 16)Aa1x"
        info "Generated new DB password"
    else
        info "Reusing existing DB password from Key Vault"
    fi

    DB_PRIVATE_DNS_ZONE="${DB_SERVER_NAME}.private.postgres.database.azure.com"
    DB_SUBNET_ID=$(az network vnet subnet show -g "$RESOURCE_GROUP" \
        --vnet-name "$VNET_NAME" -n db --query id -o tsv 2>/dev/null || true)

    # Private DNS zone
    if ! az network private-dns zone show -g "$RESOURCE_GROUP" -n "$DB_PRIVATE_DNS_ZONE" -o none 2>/dev/null; then
        az network private-dns zone create -g "$RESOURCE_GROUP" -n "$DB_PRIVATE_DNS_ZONE" -o none
        az network private-dns link vnet create -g "$RESOURCE_GROUP" \
            --zone-name "$DB_PRIVATE_DNS_ZONE" --name "${DB_SERVER_NAME}-link" \
            --virtual-network "$VNET_NAME" --registration-enabled false -o none
        info "Private DNS zone created: $DB_PRIVATE_DNS_ZONE"
    else
        info "Private DNS zone exists: $DB_PRIVATE_DNS_ZONE"
    fi

    if az postgres flexible-server show -g "$RESOURCE_GROUP" -n "$DB_SERVER_NAME" -o none 2>/dev/null; then
        info "PostgreSQL server exists: $DB_SERVER_NAME — updating password to match KV"
        az postgres flexible-server update -g "$RESOURCE_GROUP" -n "$DB_SERVER_NAME" \
            --admin-password "$DB_ADMIN_PASS" -o none
    else
        info "Creating PostgreSQL server: $DB_SERVER_NAME (this takes ~10 min)"
        az postgres flexible-server create -g "$RESOURCE_GROUP" -n "$DB_SERVER_NAME" \
            --location "$LOCATION" \
            --admin-user "$DB_ADMIN_USER" --admin-password "$DB_ADMIN_PASS" \
            --sku-name Standard_B1ms --tier Burstable --version 14 --storage-size 32 \
            ${DB_SUBNET_ID:+--subnet "$DB_SUBNET_ID" --private-dns-zone "$DB_PRIVATE_DNS_ZONE"} \
            --yes -o none
        info "PostgreSQL server created: $DB_SERVER_NAME"
    fi

    az postgres flexible-server db create -g "$RESOURCE_GROUP" \
        -s "$DB_SERVER_NAME" --database-name "$DB_NAME" -o none 2>/dev/null || true

    # Enable AAD + password auth; allow uuid-ossp extension
    az postgres flexible-server update -g "$RESOURCE_GROUP" -n "$DB_SERVER_NAME" \
        --active-directory-auth Enabled --password-auth Enabled -o none 2>/dev/null || true
    az postgres flexible-server parameter set -g "$RESOURCE_GROUP" -s "$DB_SERVER_NAME" \
        --name azure.extensions --value uuid-ossp -o none 2>/dev/null || true

    DATABASE_URL="postgresql://${DB_ADMIN_USER}:${DB_ADMIN_PASS}@${DB_HOST}:5432/${DB_NAME}?sslmode=require"
    info "PostgreSQL ready: $DB_HOST"
    section_done
fi

# =============================================================================
# 5. KEY VAULT
# =============================================================================
if [[ "$SKIP_KV" == "true" ]]; then
    warn "Skipping Key Vault (--skip-kv)"
else
    section "Key Vault"

    # Purge any soft-deleted KV with the same name
    az keyvault purge -n "$KEY_VAULT" --location "$LOCATION" -o none 2>/dev/null || true

    if ! az keyvault show -n "$KEY_VAULT" -g "$RESOURCE_GROUP" -o none 2>/dev/null; then
        az keyvault create -n "$KEY_VAULT" -g "$RESOURCE_GROUP" -l "$LOCATION" \
            --enable-rbac-authorization false -o none
        info "Key Vault created: $KEY_VAULT"
    else
        info "Key Vault exists: $KEY_VAULT"
    fi

    # Open temporarily to write secrets (tolerate already-Deny state)
    az keyvault update -n "$KEY_VAULT" -g "$RESOURCE_GROUP" --default-action Allow -o none 2>/dev/null || true

    # Only secrets that are genuine credentials
    [[ -n "${DB_ADMIN_PASS:-}" ]] && \
        az keyvault secret set --vault-name "$KEY_VAULT" --name "DatabasePassword" --value "$DB_ADMIN_PASS" -o none
    [[ -n "${AD_APPLICATION_SECRET:-}" ]] && \
        az keyvault secret set --vault-name "$KEY_VAULT" --name "ADApplicationSecret" --value "$AD_APPLICATION_SECRET" -o none
    [[ -n "${DOCKERHUB_USERNAME:-}" ]] && \
        az keyvault secret set --vault-name "$KEY_VAULT" --name "DockerHubUsername" --value "$DOCKERHUB_USERNAME" -o none
    [[ -n "${DOCKERHUB_TOKEN:-}" ]] && \
        az keyvault secret set --vault-name "$KEY_VAULT" --name "DockerHubToken" --value "$DOCKERHUB_TOKEN" -o none

    # Lock down: web subnet + AzureServices bypass (for App Service)
    WEB_SUBNET_ID=$(az network vnet subnet show -g "$RESOURCE_GROUP" \
        --vnet-name "$VNET_NAME" -n web --query id -o tsv 2>/dev/null || true)
    az keyvault update -n "$KEY_VAULT" -g "$RESOURCE_GROUP" \
        --bypass AzureServices --default-action Deny -o none
    [[ -n "$WEB_SUBNET_ID" ]] && az keyvault network-rule add -n "$KEY_VAULT" \
        -g "$RESOURCE_GROUP" --vnet-name "$VNET_NAME" --subnet web -o none 2>/dev/null || true

    # Purge protection (cannot be undone — prevents permanent deletion)
    az keyvault update -n "$KEY_VAULT" -g "$RESOURCE_GROUP" \
        --enable-purge-protection true -o none 2>/dev/null || \
        warn "Purge protection: could not enable (already set or unsupported)"

    info "Key Vault ready: $KEY_VAULT"
    section_done
fi

# =============================================================================
# 6. AZURE CONTAINER REGISTRY + CACHE
# =============================================================================
REGISTRY_PREFIX="docker.io"   # default; overridden if ACR pull-through cache is set up

if [[ "$SKIP_ACR" == "true" ]]; then
    warn "Skipping ACR setup (--skip-acr)"
    ACR_LOGIN_SERVER=$(acr_az acr show -n "$ACR_NAME" --query loginServer -o tsv 2>/dev/null || true)
else
    section "Container Registry"

    if acr_az acr show -n "$ACR_NAME" -o none 2>/dev/null; then
        info "ACR exists: $ACR_NAME"
    else
        info "Creating ACR: $ACR_NAME"
        acr_az acr create -g "$RESOURCE_GROUP" -n "$ACR_NAME" \
            --sku Basic -l "$LOCATION" --admin-enabled false -o none
        info "ACR created: $ACR_NAME"
    fi

    ACR_LOGIN_SERVER=$(acr_az acr show -n "$ACR_NAME" --query loginServer -o tsv)
    ACR_ID=$(acr_az acr show -n "$ACR_NAME" --query id -o tsv)
    info "ACR: $ACR_LOGIN_SERVER"

    # ── Pull-through cache (optional — needs DockerHub credentials) ─────────
    # Read from KV if not passed as env vars
    if [[ -z "$DOCKERHUB_USERNAME" ]]; then
        DOCKERHUB_USERNAME=$(az keyvault secret show --vault-name "$KEY_VAULT" \
            --name "DockerHubUsername" --query value -o tsv 2>/dev/null || true)
    fi
    if [[ -z "$DOCKERHUB_TOKEN" ]]; then
        DOCKERHUB_TOKEN=$(az keyvault secret show --vault-name "$KEY_VAULT" \
            --name "DockerHubToken" --query value -o tsv 2>/dev/null || true)
    fi

    if [[ -n "$DOCKERHUB_USERNAME" && -n "$DOCKERHUB_TOKEN" ]]; then
        info "Setting up ACR pull-through cache (DockerHub → ACR)..."
        KV_USER_URI="${KV_URL}/secrets/DockerHubUsername"
        KV_TOKEN_URI="${KV_URL}/secrets/DockerHubToken"

        if ! acr_az acr credential-set show -r "$ACR_NAME" -n "dockerhub" -o none 2>/dev/null; then
            acr_az acr credential-set create -r "$ACR_NAME" -n "dockerhub" \
                -l "docker.io" -u "$KV_USER_URI" -p "$KV_TOKEN_URI" -o none
            # Grant ACR credential set identity access to KV secrets
            CRED_PRINCIPAL=$(acr_az acr credential-set show -r "$ACR_NAME" -n "dockerhub" \
                --query "identity.principalId" -o tsv 2>/dev/null || true)
            [[ -n "$CRED_PRINCIPAL" ]] && az keyvault set-policy -n "$KEY_VAULT" \
                --object-id "$CRED_PRINCIPAL" --secret-permissions get -o none 2>/dev/null || true
            info "DockerHub credential set created"
        else
            info "DockerHub credential set exists"
        fi

        # Cache rules: base images used in our Dockerfiles
        for src_path in "library/rust" "library/node" "library/debian"; do
            rule_name="${src_path//\//-}-cache"
            if ! acr_az acr cache show -r "$ACR_NAME" -n "$rule_name" -o none 2>/dev/null; then
                acr_az acr cache create -r "$ACR_NAME" -n "$rule_name" \
                    -s "docker.io/${src_path}" -t "cache/${src_path}" \
                    -c "dockerhub" -o none 2>/dev/null && \
                    info "  Cache rule: docker.io/${src_path} → ${ACR_LOGIN_SERVER}/cache/${src_path}" || \
                    warn "  Could not create cache rule for ${src_path}"
            else
                info "  Cache rule exists: ${src_path}"
            fi
        done

        REGISTRY_PREFIX="${ACR_LOGIN_SERVER}/cache"
        info "Pull-through cache ready. REGISTRY_PREFIX=$REGISTRY_PREFIX"
    else
        warn "DockerHub credentials not set — skipping pull-through cache"
        warn "  Add DOCKERHUB_USERNAME + DOCKERHUB_TOKEN to .env to enable"
    fi

    section_done
fi

# =============================================================================
# 7. DOCKER BUILD + PUSH
# =============================================================================
BUILD_TAG="$(date +%Y%m%d%H%M%S)"
ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"

if [[ "$SKIP_BUILD" == "true" ]]; then
    warn "Skipping Docker build (--skip-build) — resolving latest tags from ACR"
    ADMIN_TAG=$(acr_az acr repository show-tags -n "$ACR_NAME" --repository admin-site \
        --orderby time_desc --top 1 -o tsv 2>/dev/null || echo "latest")
    CUSTOMER_TAG=$(acr_az acr repository show-tags -n "$ACR_NAME" --repository customer-site \
        --orderby time_desc --top 1 -o tsv 2>/dev/null || echo "latest")
    ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${ADMIN_TAG}"
    CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${CUSTOMER_TAG}"
    info "Admin image:    $ADMIN_IMAGE"
    info "Customer image: $CUSTOMER_IMAGE"
else
    section "Docker build + push  (tag: $BUILD_TAG)"

    # Detect whether a Docker daemon is reachable
    if docker info &>/dev/null 2>&1; then
        BUILD_METHOD="docker"
        info "Build method: local Docker (buildx)"
    else
        BUILD_METHOD="acr"
        warn "Docker daemon not available — falling back to ACR Tasks (builds run in Azure)"
    fi

    if [[ "$BUILD_METHOD" == "acr" ]]; then
        # ── ACR Tasks fallback (no local Docker required) ─────────────────────
        # Uses az acr run with a task YAML to enable BuildKit (required for
        # --mount=type=cache in the Dockerfiles).
        _acr_build() {
            local name="$1" dockerfile="$2"
            info "  Building ${name} via ACR Tasks (BuildKit)..."

            # Task YAML must live inside the repo root so it is included in the
            # uploaded source context that ACR Tasks downloads.
            local task_file="${REPO_ROOT}/.acr-task-${name}.yml"

            cat > "$task_file" <<TASKYAML
version: v1.1.0
steps:
  - id: build
    build: >-
      -f deployment/${dockerfile}
      --build-arg REGISTRY_PREFIX=${REGISTRY_PREFIX}
      --build-arg VITE_ADMIN_API_URL=${ADMIN_URL}
      --build-arg VITE_CUSTOMER_API_URL=${CUSTOMER_URL}
      --build-arg BUILDKIT_INLINE_CACHE=1
      --cache-from \$Registry/buildcache/${name}:cache
      -t \$Registry/${name}:${BUILD_TAG}
      -t \$Registry/${name}:latest
      -t \$Registry/buildcache/${name}:cache
      .
    env:
      - DOCKER_BUILDKIT=1
  - id: push
    push:
      - \$Registry/${name}:${BUILD_TAG}
      - \$Registry/${name}:latest
      - \$Registry/buildcache/${name}:cache
TASKYAML

            acr_az acr run \
                --registry "$ACR_NAME" \
                --file ".acr-task-${name}.yml" \
                "${REPO_ROOT}"

            rm -f "$task_file"
            info "  ✓ ${name} built and pushed"
        }

        _acr_build admin-site    Dockerfile.admin-site
        _acr_build customer-site Dockerfile.customer-site

        info "  Building migrate via ACR Tasks (BuildKit)..."
        migrate_task="${REPO_ROOT}/.acr-task-migrate.yml"
        cat > "$migrate_task" <<TASKYAML
version: v1.1.0
steps:
  - id: build
    build: >-
      -f deployment/Dockerfile.migrate
      --build-arg REGISTRY_PREFIX=${REGISTRY_PREFIX}
      --build-arg BUILDKIT_INLINE_CACHE=1
      --cache-from \$Registry/buildcache/migrate:cache
      -t \$Registry/migrate:latest
      -t \$Registry/buildcache/migrate:cache
      .
    env:
      - DOCKER_BUILDKIT=1
  - id: push
    push:
      - \$Registry/migrate:latest
      - \$Registry/buildcache/migrate:cache
TASKYAML

        acr_az acr run \
            --registry "$ACR_NAME" \
            --file ".acr-task-migrate.yml" \
            "${REPO_ROOT}"

        rm -f "$migrate_task"
        info "  ✓ migrate built and pushed"

    else
        # ── Local Docker buildx (preferred) ───────────────────────────────────
        acr_az acr login -n "$ACR_NAME"

        docker buildx create --name saas-builder --driver docker-container \
            --use 2>/dev/null || docker buildx use saas-builder
        docker buildx inspect --bootstrap > /dev/null 2>&1

        _buildx() {
            local name="$1" dockerfile="$2"
            local cache_ref="${ACR_LOGIN_SERVER}/buildcache/${name}"
            docker buildx build \
                --platform linux/amd64 \
                --cache-from "type=registry,ref=${cache_ref}" \
                --cache-to  "type=registry,ref=${cache_ref},mode=max" \
                --build-arg "REGISTRY_PREFIX=${REGISTRY_PREFIX}" \
                --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
                --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
                -f "${SCRIPT_DIR}/${dockerfile}" \
                --tag "${ACR_LOGIN_SERVER}/${name}:${BUILD_TAG}" \
                --tag "${ACR_LOGIN_SERVER}/${name}:latest" \
                --push "${REPO_ROOT}" \
                > "${SCRIPT_DIR}/build-${name//-site/}.log" 2>&1
        }

        info "Building all images in parallel (BuildKit layer cache enabled)..."
        _buildx admin-site    Dockerfile.admin-site    & ADMIN_PID=$!
        _buildx customer-site Dockerfile.customer-site & CUSTOMER_PID=$!
        docker buildx build --platform linux/amd64 \
            --cache-from "type=registry,ref=${ACR_LOGIN_SERVER}/buildcache/migrate" \
            --cache-to  "type=registry,ref=${ACR_LOGIN_SERVER}/buildcache/migrate,mode=max" \
            --build-arg "REGISTRY_PREFIX=${REGISTRY_PREFIX}" \
            -f "${SCRIPT_DIR}/Dockerfile.migrate" \
            --tag "${ACR_LOGIN_SERVER}/migrate:latest" --push "${REPO_ROOT}" \
            > "${SCRIPT_DIR}/build-migrate.log" 2>&1 &
        MIGRATE_PID=$!

        FAIL=0
        for pair in "${ADMIN_PID}:admin" "${CUSTOMER_PID}:customer" "${MIGRATE_PID}:migrate"; do
            pid="${pair%%:*}"; name="${pair##*:}"
            if wait "$pid"; then
                info "  ✓ ${name} image built"
            else
                error "  ✗ ${name} build failed — tail of log:"
                tail -15 "${SCRIPT_DIR}/build-${name}.log" 2>/dev/null || true
                FAIL=1
            fi
        done
        [[ "$FAIL" -eq 0 ]] || die "One or more Docker builds failed"
    fi

    section_done
fi

# =============================================================================
# 8. DATABASE MIGRATIONS
# =============================================================================
if [[ "$SKIP_MIGRATIONS" == "true" ]]; then
    warn "Skipping migrations (--skip-migrations)"
else
    section "Database Migrations"
    MIGRATE_JOB="${P}-migrate"

    az container delete -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" --yes -o none 2>/dev/null || true

    ACI_SUBNET_ID=$(az network vnet subnet show -g "$RESOURCE_GROUP" \
        --vnet-name "$VNET_NAME" -n aci --query id -o tsv 2>/dev/null || true)

    az container create -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" \
        --image "${ACR_LOGIN_SERVER}/migrate:latest" \
        --registry-login-server "$ACR_LOGIN_SERVER" \
        --assign-identity \
        --os-type Linux --cpu 1 --memory 1 --restart-policy Never \
        ${ACI_SUBNET_ID:+--vnet "$VNET_NAME" --subnet aci} \
        --secure-environment-variables "DATABASE_URL=${DATABASE_URL:-}" \
        -o none

    info "Waiting for migrations..."
    for i in $(seq 1 36); do
        STATE=$(az container show -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" \
            --query "containers[0].instanceView.currentState.state" -o tsv 2>/dev/null || echo "Unknown")
        if [[ "$STATE" == "Terminated" ]]; then
            EXIT=$(az container show -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" \
                --query "containers[0].instanceView.currentState.exitCode" -o tsv)
            az container logs -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" 2>/dev/null || true
            az container delete -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" --yes -o none 2>/dev/null || true
            [[ "$EXIT" == "0" ]] || die "Migrations failed (exit $EXIT)"
            info "Migrations complete"
            break
        fi
        info "  State: $STATE (${i}/36)..."
        sleep 10
    done
    section_done
fi

# =============================================================================
# 9. WEB APPS
# =============================================================================
_ensure_webapp() {
    local app_name="$1" image="$2" port="$3"
    shift 3

    if az webapp show -n "$app_name" -g "$RESOURCE_GROUP" -o none 2>/dev/null; then
        info "  Web App exists: $app_name — updating config"
    else
        az webapp create -g "$RESOURCE_GROUP" -p "$APP_PLAN" -n "$app_name" \
            --deployment-container-image-name "$image" -o none
        info "  Web App created: $app_name"
    fi

    # Core settings (no credentials — Managed Identity for ACR pull)
    az webapp config appsettings set -n "$app_name" -g "$RESOURCE_GROUP" --settings \
        "DOCKER_REGISTRY_SERVER_URL=https://${ACR_LOGIN_SERVER}" \
        "WEBSITES_PORT=80" "PORT=$port" \
        "RUST_LOG=info" \
        "WEBSITE_DNS_SERVER=168.63.129.16" \
        "WEBSITES_ENABLE_APP_SERVICE_STORAGE=false" \
        "$@" -o none

    # Container image + hardening
    az webapp config set -n "$app_name" -g "$RESOURCE_GROUP" \
        --linux-fx-version "DOCKER|${image}" \
        --always-on true --http20-enabled true \
        --min-tls-version "1.2" \
        --ftps-state Disabled \
        --remote-debugging-enabled false -o none
    az webapp update -n "$app_name" -g "$RESOURCE_GROUP" --https-only true -o none

    # VNet integration (outbound — reach DB/KV through private network)
    az webapp vnet-integration add -n "$app_name" -g "$RESOURCE_GROUP" \
        --vnet "$VNET_NAME" --subnet web -o none 2>/dev/null || true

    # System identity + AcrPull
    IDENTITY=$(az webapp identity assign -n "$app_name" -g "$RESOURCE_GROUP" \
        --identities "[system]" --query principalId -o tsv)
    acr_az role assignment create --assignee "$IDENTITY" --role AcrPull --scope "$ACR_ID" -o none 2>/dev/null || true
    az keyvault set-policy -n "$KEY_VAULT" -g "$RESOURCE_GROUP" \
        --object-id "$IDENTITY" --secret-permissions get list -o none 2>/dev/null || true

    # Enable Managed Identity for ACR pull (no credentials stored)
    APP_ID=$(az webapp show -n "$app_name" -g "$RESOURCE_GROUP" --query id -o tsv)
    az resource update --ids "${APP_ID}/config/web" \
        --set properties.acrUseManagedIdentityCreds=true \
        --set properties.scmIpSecurityRestrictionsUseMain=true -o none 2>/dev/null || true

    # Remove any legacy admin credentials that may have been set previously
    az webapp config appsettings delete -n "$app_name" -g "$RESOURCE_GROUP" \
        --setting-names "DOCKER_REGISTRY_SERVER_USERNAME" "DOCKER_REGISTRY_SERVER_PASSWORD" \
        -o none 2>/dev/null || true
}

if [[ "$SKIP_WEBAPPS" == "true" ]]; then
    warn "Skipping Web Apps (--skip-webapps)"
else
    section "App Service Plan"
    if az appservice plan show -n "$APP_PLAN" -g "$RESOURCE_GROUP" -o none 2>/dev/null; then
        info "App Service Plan exists: $APP_PLAN"
    else
        az appservice plan create -n "$APP_PLAN" -g "$RESOURCE_GROUP" \
            -l "$LOCATION" --is-linux --sku B1 -o none
        info "App Service Plan created: $APP_PLAN (B1 Linux)"
    fi
    section_done

    section "Admin Web App"
    _ensure_webapp "$ADMIN_APP" "$ADMIN_IMAGE" "3000" \
        "KEY_VAULT_URL=${KV_URL}" \
        "DB_HOST=${DB_HOST}" \
        "DB_NAME=${DB_NAME}" \
        "AZURE_AD_AUTH=true" \
        "SaaS_API_CLIENT_ID=${AD_APPLICATION_ID:-}" \
        "SaaS_API_TENANT_ID=${TENANT_ID}" \
        "AZURE_AD_TENANT_ID=${TENANT_ID}" \
        "AZURE_AD_CLIENT_ID=${AD_APPLICATION_ID_ADMIN:-}" \
        "AZURE_AD_REDIRECT_URI=${ADMIN_URL}/auth/callback" \
        "AZURE_AD_SIGNED_OUT_REDIRECT_URI=${ADMIN_URL}/admin" \
        "MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api" \
        "MARKETPLACE_API_VERSION=2018-08-31" \
        "CORS_ALLOWED_ORIGINS=${ADMIN_URL}" \
        "KNOWN_USERS=${PUBLISHER_ADMIN_USERS}"

    # Register managed identity as PostgreSQL AAD user
    ADMIN_OBJ=$(az webapp identity show -n "$ADMIN_APP" -g "$RESOURCE_GROUP" \
        --query principalId -o tsv 2>/dev/null || true)
    [[ -n "$ADMIN_OBJ" ]] && az postgres flexible-server ad-admin create \
        -g "$RESOURCE_GROUP" -s "$DB_SERVER_NAME" \
        --display-name "$ADMIN_APP" --object-id "$ADMIN_OBJ" -o none 2>/dev/null || true

    info "Admin Web App: $ADMIN_URL"
    section_done

    section "Customer Web App"
    _ensure_webapp "$CUSTOMER_APP" "$CUSTOMER_IMAGE" "3001" \
        "KEY_VAULT_URL=${KV_URL}" \
        "DB_HOST=${DB_HOST}" \
        "DB_NAME=${DB_NAME}" \
        "AZURE_AD_AUTH=true" \
        "SaaS_API_CLIENT_ID=${AD_APPLICATION_ID:-}" \
        "SaaS_API_TENANT_ID=${TENANT_ID}" \
        "MARKETPLACE_API_BASE_URL=https://marketplaceapi.microsoft.com/api" \
        "MARKETPLACE_API_VERSION=2018-08-31" \
        "CORS_ALLOWED_ORIGINS=${CUSTOMER_URL}"

    CUSTOMER_OBJ=$(az webapp identity show -n "$CUSTOMER_APP" -g "$RESOURCE_GROUP" \
        --query principalId -o tsv 2>/dev/null || true)
    [[ -n "$CUSTOMER_OBJ" ]] && az postgres flexible-server ad-admin create \
        -g "$RESOURCE_GROUP" -s "$DB_SERVER_NAME" \
        --display-name "$CUSTOMER_APP" --object-id "$CUSTOMER_OBJ" -o none 2>/dev/null || true

    info "Customer Web App: $CUSTOMER_URL"
    section_done
fi

# =============================================================================
# 10. SECURITY HARDENING
# =============================================================================
if [[ "$SKIP_HARDENING" == "true" ]]; then
    warn "Skipping security hardening (--skip-hardening)"
else
    section "Security hardening"

    # NSG on DB subnet: PostgreSQL only from web/aci subnets
    NSG_DB="${P}-db-nsg"
    if ! az network nsg show -g "$RESOURCE_GROUP" -n "$NSG_DB" -o none 2>/dev/null; then
        az network nsg create -g "$RESOURCE_GROUP" -n "$NSG_DB" -o none
        for ruledef in "Allow-PG-web:100:10.0.1.0/24" "Allow-PG-aci:110:10.0.3.0/24"; do
            rname="${ruledef%%:*}"; rest="${ruledef#*:}"; prio="${rest%%:*}"; src="${rest##*:}"
            az network nsg rule create -g "$RESOURCE_GROUP" --nsg-name "$NSG_DB" \
                --name "$rname" --priority "$prio" --access Allow --direction Inbound \
                --protocol Tcp --source-address-prefixes "$src" \
                --destination-port-ranges 5432 -o none
        done
        az network nsg rule create -g "$RESOURCE_GROUP" --nsg-name "$NSG_DB" \
            --name "Deny-All" --priority 4096 --access Deny --direction Inbound \
            --protocol "*" --source-address-prefixes "*" --destination-port-ranges "*" -o none
        az network vnet subnet update -g "$RESOURCE_GROUP" --vnet-name "$VNET_NAME" \
            -n db --network-security-group "$NSG_DB" -o none
        info "NSG $NSG_DB applied to db subnet (port 5432 from web/aci only)"
    else
        info "NSG $NSG_DB exists"
    fi

    section_done
fi

# =============================================================================
# 11. RESTART + HEALTH CHECK
# =============================================================================
section "Restart + health check"
for app_name in "$ADMIN_APP" "$CUSTOMER_APP"; do
    az webapp restart -n "$app_name" -g "$RESOURCE_GROUP" -o none 2>/dev/null || true
    info "  Restarted: $app_name"
done

_wait_healthy() {
    local url="$1/health" name="$2"
    info "  Waiting for $name to become healthy..."
    for i in $(seq 1 18); do
        code=$(curl -sf -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null || echo "000")
        [[ "$code" == "200" ]] && { info "  ✓ $name healthy (attempt $i)"; return 0; }
        echo -n "."
        sleep 10
    done
    echo ""
    warn "$name not healthy after 3 min — check: az webapp log tail -n $name -g $RESOURCE_GROUP"
}
_wait_healthy "$ADMIN_URL"    "$ADMIN_APP"
_wait_healthy "$CUSTOMER_URL" "$CUSTOMER_APP"
section_done

# =============================================================================
# 12. SEED KNOWN USERS
# =============================================================================
section "Seed publisher admin users"
IFS=',' read -ra ADMIN_EMAILS <<< "$PUBLISHER_ADMIN_USERS"
for EMAIL in "${ADMIN_EMAILS[@]}"; do
    EMAIL="$(echo "$EMAIL" | tr -d '[:space:]')"
    [[ -z "$EMAIL" ]] && continue
    if command -v psql &>/dev/null && [[ -n "${DATABASE_URL:-}" ]]; then
        psql "$DATABASE_URL" -c \
            "INSERT INTO known_users(user_email,role_id) VALUES('${EMAIL}',1) ON CONFLICT DO NOTHING;" \
            -q 2>/dev/null && info "  Seeded: $EMAIL" || warn "  Could not seed $EMAIL"
    else
        warn "  psql not available — add $EMAIL to known_users manually (role_id=1)"
    fi
done
section_done

# =============================================================================
# SUMMARY
# =============================================================================
echo ""
echo -e "${BOLD}${GREEN}✅  Deployment complete${NC}"
echo ""
echo "  Admin portal:   $ADMIN_URL"
echo "  Customer portal:$CUSTOMER_URL"
echo "  Key Vault:      $KEY_VAULT"
echo "  ACR:            ${ACR_LOGIN_SERVER:-$ACR_NAME}"
echo "  PostgreSQL:     $DB_HOST"
echo ""
echo -e "${CYAN}▶  Partner Center configuration:${NC}"
echo "  Landing Page:          ${CUSTOMER_URL}/"
echo "  Webhook:               ${CUSTOMER_URL}/api/webhook"
echo "  Tenant ID:             ${TENANT_ID}"
echo "  AAD Application ID:    ${AD_APPLICATION_ID:-<set in Partner Center>}"
echo ""
echo "  Log: $LOG_FILE"
