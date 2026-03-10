#!/usr/bin/env bash
# =============================================================================
# SaaS Accelerator – Rust Edition  |  upgrade.sh
# =============================================================================
# Rebuild Docker images (with BuildKit cache → fast), run DB migrations,
# update both web apps, and restart them.  Safe to re-run any time.
#
# Usage:
#   ./deployment/upgrade.sh --prefix myco
#
# Options:
#   --prefix PREFIX          Required — same prefix used in deploy.sh
#   --resource-group RG      (default: same as prefix)
#   --subscription ID        Azure subscription
#   --acr NAME               ACR name (default: derived from prefix)
#   --acr-subscription ID    If ACR is in a different subscription
#   --tag TAG                Use a specific existing tag instead of rebuilding
#   --skip-build             Redeploy existing latest image without rebuilding
#   --skip-migrations        Skip sqlx migrate run
# =============================================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$SCRIPT_DIR/upgrade-$(date +%Y%m%d-%H%M%S).log"
echo "=== upgrade.sh  $(date) ===" > "$LOG_FILE"

log()  { echo "$*" >> "$LOG_FILE"; }
info() { echo -e "${GREEN}  ✓${NC} $*"; log "[INFO] $*"; }
warn() { echo -e "${YELLOW}  ⚠${NC} $*"; log "[WARN] $*"; }
error(){ echo -e "${RED}  ✗${NC} $*"; log "[ERROR] $*"; }
die()  { error "$*"; exit 1; }

_SECTION_START=0
section() {
    echo -e "\n${CYAN}${BOLD}══ $1 ══${NC}"
    log "\n=== $1 ==="
    _SECTION_START=$(date +%s)
}
section_done() { echo -e "   ${GREEN}done in $(( $(date +%s) - _SECTION_START ))s${NC}"; }

trap 'error "Failed at line $LINENO"; error "Log: $LOG_FILE"' ERR

# ── Load .env ─────────────────────────────────────────────────────────────────
[[ -f "$SCRIPT_DIR/.env" ]] && { set -a; source "$SCRIPT_DIR/.env"; set +a; }

# ── Parameters ────────────────────────────────────────────────────────────────
WEB_APP_NAME_PREFIX="${WEB_APP_NAME_PREFIX:-}"
RESOURCE_GROUP="${RESOURCE_GROUP:-}"
AZURE_SUBSCRIPTION_ID="${AZURE_SUBSCRIPTION_ID:-}"
ACR_NAME="${ACR_NAME:-}"
ACR_SUBSCRIPTION="${ACR_SUBSCRIPTION:-}"
EXPLICIT_TAG=""
SKIP_BUILD=false
SKIP_MIGRATIONS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)           WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --resource-group)   RESOURCE_GROUP="$2"; shift 2 ;;
        --subscription)     AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --acr)              ACR_NAME="$2"; shift 2 ;;
        --acr-subscription) ACR_SUBSCRIPTION="$2"; shift 2 ;;
        --tag)              EXPLICIT_TAG="$2"; shift 2 ;;
        --skip-build)       SKIP_BUILD=true; shift ;;
        --skip-migrations)  SKIP_MIGRATIONS=true; shift ;;
        *) die "Unknown option: $1" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]] && die "--prefix is required"

P="$WEB_APP_NAME_PREFIX"
RESOURCE_GROUP="${RESOURCE_GROUP:-$P}"
ACR_NAME="${ACR_NAME:-${P//-/}acr}"
ADMIN_APP="${P}-admin"
CUSTOMER_APP="${P}-portal"
ADMIN_URL="https://${ADMIN_APP}.azurewebsites.net"
CUSTOMER_URL="https://${CUSTOMER_APP}.azurewebsites.net"
KEY_VAULT="${KEY_VAULT:-${P}-kv}"

acr_az() {
    [[ -n "$ACR_SUBSCRIPTION" ]] && az --subscription "$ACR_SUBSCRIPTION" "$@" || az "$@"
}

[[ -n "$AZURE_SUBSCRIPTION_ID" ]] && az account set --subscription "$AZURE_SUBSCRIPTION_ID"

echo ""
echo -e "${BOLD}SaaS Accelerator – upgrade.sh${NC}"
echo "  Prefix: $P  |  Resource Group: $RESOURCE_GROUP"
echo "  Log:    $LOG_FILE"
echo ""

for cmd in az docker; do command -v "$cmd" &>/dev/null || die "Required: $cmd"; done

ACR_LOGIN_SERVER=$(acr_az acr show -n "$ACR_NAME" --query loginServer -o tsv 2>/dev/null) || \
    die "ACR '$ACR_NAME' not found — run deploy.sh first"

# Determine REGISTRY_PREFIX (use ACR pull-through cache if configured)
REGISTRY_PREFIX="docker.io"
if acr_az acr credential-set show -r "$ACR_NAME" -n "dockerhub" -o none 2>/dev/null; then
    REGISTRY_PREFIX="${ACR_LOGIN_SERVER}/cache"
    info "Pull-through cache active: REGISTRY_PREFIX=$REGISTRY_PREFIX"
fi

# =============================================================================
# 1. BUILD
# =============================================================================
if [[ -n "$EXPLICIT_TAG" ]]; then
    BUILD_TAG="$EXPLICIT_TAG"
    ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
    CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"
    info "Using explicit tag: $BUILD_TAG"
elif [[ "$SKIP_BUILD" == "true" ]]; then
    ADMIN_TAG=$(acr_az acr repository show-tags -n "$ACR_NAME" --repository admin-site \
        --orderby time_desc --top 1 -o tsv 2>/dev/null || echo "latest")
    CUSTOMER_TAG=$(acr_az acr repository show-tags -n "$ACR_NAME" --repository customer-site \
        --orderby time_desc --top 1 -o tsv 2>/dev/null || echo "latest")
    BUILD_TAG="$ADMIN_TAG"
    ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${ADMIN_TAG}"
    CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${CUSTOMER_TAG}"
    warn "Skipping build — redeploying: admin=$ADMIN_TAG  customer=$CUSTOMER_TAG"
else
    section "Docker build + push"
    BUILD_TAG="$(date +%Y%m%d%H%M%S)"
    ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
    CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"

    # Resolve ADMIN_URL / CUSTOMER_URL from existing app settings if not set
    ADMIN_URL="${ADMIN_URL:-$(az webapp config appsettings list -n "$ADMIN_APP" \
        -g "$RESOURCE_GROUP" --query "[?name=='AZURE_AD_REDIRECT_URI'].value" -o tsv \
        2>/dev/null | sed 's|/auth/callback||')}"
    CUSTOMER_URL="${CUSTOMER_URL:-$(az webapp config appsettings list -n "$CUSTOMER_APP" \
        -g "$RESOURCE_GROUP" --query "[?name=='CORS_ALLOWED_ORIGINS'].value" -o tsv 2>/dev/null || true)}"

    acr_az acr login -n "$ACR_NAME"

    docker buildx create --name saas-builder --driver docker-container \
        --use 2>/dev/null || docker buildx use saas-builder
    docker buildx inspect --bootstrap > /dev/null 2>&1

    _buildx() {
        local name="$1" dockerfile="$2"
        docker buildx build \
            --platform linux/amd64 \
            --cache-from "type=registry,ref=${ACR_LOGIN_SERVER}/buildcache/${name}" \
            --cache-to  "type=registry,ref=${ACR_LOGIN_SERVER}/buildcache/${name},mode=max" \
            --build-arg "REGISTRY_PREFIX=${REGISTRY_PREFIX}" \
            --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL:-}" \
            --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL:-}" \
            -f "${SCRIPT_DIR}/${dockerfile}" \
            --tag "${ACR_LOGIN_SERVER}/${name}:${BUILD_TAG}" \
            --tag "${ACR_LOGIN_SERVER}/${name}:latest" \
            --push "${REPO_ROOT}" \
            > "${SCRIPT_DIR}/build-${name//-site/}.log" 2>&1
    }

    info "Building in parallel (tag: $BUILD_TAG)..."
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
            info "  ✓ ${name}"
        else
            error "  ✗ ${name} failed:"; tail -15 "${SCRIPT_DIR}/build-${name}.log" 2>/dev/null || true
            FAIL=1
        fi
    done
    [[ "$FAIL" -eq 0 ]] || die "Build failed"
    section_done
fi

# =============================================================================
# 2. DATABASE MIGRATIONS
# =============================================================================
if [[ "$SKIP_MIGRATIONS" == "true" ]]; then
    warn "Skipping migrations (--skip-migrations)"
else
    section "Database migrations"
    VNET_NAME="${P}-vnet"
    MIGRATE_JOB="${P}-migrate"
    DB_URL=$(az keyvault secret show --vault-name "$KEY_VAULT" \
        --name "DatabasePassword" --query value -o tsv 2>/dev/null | \
        { read -r pass; echo "postgresql://saasadmin:${pass}@${P}-db.postgres.database.azure.com:5432/${P}AMPSaaSDB?sslmode=require"; } \
    ) 2>/dev/null || DB_URL=""

    az container delete -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" --yes -o none 2>/dev/null || true
    ACI_SUBNET_ID=$(az network vnet subnet show -g "$RESOURCE_GROUP" \
        --vnet-name "$VNET_NAME" -n aci --query id -o tsv 2>/dev/null || true)
    az container create -g "$RESOURCE_GROUP" -n "$MIGRATE_JOB" \
        --image "${ACR_LOGIN_SERVER}/migrate:latest" \
        --registry-login-server "$ACR_LOGIN_SERVER" \
        --assign-identity \
        --os-type Linux --cpu 1 --memory 1 --restart-policy Never \
        ${ACI_SUBNET_ID:+--vnet "$VNET_NAME" --subnet aci} \
        --secure-environment-variables "DATABASE_URL=${DB_URL:-}" \
        -o none

    for i in $(seq 1 30); do
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
        echo -n "."; sleep 10
    done
    echo ""
    section_done
fi

# =============================================================================
# 3. UPDATE WEB APPS + RESTART
# =============================================================================
section "Deploy + restart"

az webapp config set -n "$ADMIN_APP" -g "$RESOURCE_GROUP" \
    --linux-fx-version "DOCKER|${ADMIN_IMAGE}" -o none
az webapp restart -n "$ADMIN_APP" -g "$RESOURCE_GROUP" -o none
info "Admin Web App updated: $ADMIN_IMAGE"

az webapp config set -n "$CUSTOMER_APP" -g "$RESOURCE_GROUP" \
    --linux-fx-version "DOCKER|${CUSTOMER_IMAGE}" -o none
az webapp restart -n "$CUSTOMER_APP" -g "$RESOURCE_GROUP" -o none
info "Customer Web App updated: $CUSTOMER_IMAGE"

# Health check
_wait_healthy() {
    local url="$1/health" name="$2"
    info "  Waiting for $name..."
    for i in $(seq 1 18); do
        code=$(curl -sf -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null || echo "000")
        [[ "$code" == "200" ]] && { info "  ✓ $name healthy"; return 0; }
        echo -n "."; sleep 10
    done
    echo ""; warn "$name not healthy — check: az webapp log tail -n $name -g $RESOURCE_GROUP"
}
_wait_healthy "$ADMIN_URL"    "$ADMIN_APP"
_wait_healthy "$CUSTOMER_URL" "$CUSTOMER_APP"
section_done

echo ""
echo -e "${BOLD}${GREEN}✅  Upgrade complete  (tag: $BUILD_TAG)${NC}"
echo ""
echo "  Admin:    $ADMIN_URL"
echo "  Customer: $CUSTOMER_URL"
echo "  Log:      $LOG_FILE"
