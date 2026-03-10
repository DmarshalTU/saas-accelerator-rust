#!/usr/bin/env bash
# =============================================================================
# SaaS Accelerator – Rust Edition  |  Upgrade Script
# Updates running containers on existing Admin + Customer Web Apps.
# Equivalent to the original Upgrade.ps1.
#
# Usage:
#   ./upgrade.sh --prefix mycompany [--subscription <id>]
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
UPGRADE_LOG="$SCRIPT_DIR/upgrade-$(date +%Y%m%d-%H%M%S).log"
echo "=== SaaS Accelerator Upgrade  $(date) ===" > "$UPGRADE_LOG"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${GREEN}[INFO]${NC}  $*"; echo "[INFO]  $(date +%T) $*" >> "$UPGRADE_LOG"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; echo "[WARN]  $(date +%T) $*" >> "$UPGRADE_LOG"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; echo "[ERROR] $(date +%T) $*" >> "$UPGRADE_LOG"; }
section() { echo -e "\n${CYAN}══ $* ══${NC}"; echo -e "\n═══ $* ═══" >> "$UPGRADE_LOG"; }
die()     { error "$*"; exit 1; }
on_error() { error "Upgrade failed at line $1 (exit $2)"; error "Log: $UPGRADE_LOG"; }
trap 'on_error $LINENO $?' ERR

# ── load .env ─────────────────────────────────────────────────────────────────
ENV_FILE="$SCRIPT_DIR/.env"
[[ -f "$ENV_FILE" ]] && { info "Loading $ENV_FILE"; set -a; source "$ENV_FILE"; set +a; }

# ── parameters ────────────────────────────────────────────────────────────────
WEB_APP_NAME_PREFIX="${WEB_APP_NAME_PREFIX:-}"
RESOURCE_GROUP="${RESOURCE_GROUP:-}"
AZURE_SUBSCRIPTION_ID="${AZURE_SUBSCRIPTION_ID:-}"
ACR_NAME="${ACR_NAME:-}"
VITE_ADMIN_API_URL="${VITE_ADMIN_API_URL:-}"
VITE_CUSTOMER_API_URL="${VITE_CUSTOMER_API_URL:-}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)       WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --resource-group) RESOURCE_GROUP="$2"; shift 2 ;;
        --subscription) AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --acr)          ACR_NAME="$2"; shift 2 ;;
        *) die "Unknown option: $1" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]] && die "--prefix is required"
RESOURCE_GROUP="${RESOURCE_GROUP:-$WEB_APP_NAME_PREFIX}"
ACR_NAME="${ACR_NAME:-${WEB_APP_NAME_PREFIX//-/}acr}"
ADMIN_APP="${WEB_APP_NAME_PREFIX}-admin"
CUSTOMER_APP="${WEB_APP_NAME_PREFIX}-portal"
ADMIN_URL="${VITE_ADMIN_API_URL:-https://${ADMIN_APP}.azurewebsites.net}"
CUSTOMER_URL="${VITE_CUSTOMER_API_URL:-https://${CUSTOMER_APP}.azurewebsites.net}"

section "Upgrade configuration"
echo "  Prefix:         $WEB_APP_NAME_PREFIX"
echo "  Resource group: $RESOURCE_GROUP"
echo "  ACR:            $ACR_NAME"
echo "  Admin URL:      $ADMIN_URL"
echo "  Customer URL:   $CUSTOMER_URL"

for cmd in az docker; do
    command -v "$cmd" &>/dev/null || die "'$cmd' not found."
done

[[ -n "$AZURE_SUBSCRIPTION_ID" ]] && az account set --subscription "$AZURE_SUBSCRIPTION_ID"

# ── ACR credentials ───────────────────────────────────────────────────────────
section "ACR login"
ACR_LOGIN_SERVER=$(az acr show --name "$ACR_NAME" --query loginServer -o tsv)
ACR_USERNAME=$(az acr credential show --name "$ACR_NAME" --query username -o tsv)
ACR_PASSWORD=$(az acr credential show --name "$ACR_NAME" --query "passwords[0].value" -o tsv)

DOCKER_BUILDKIT=1 docker login "$ACR_LOGIN_SERVER" \
    --username "$ACR_USERNAME" --password "$ACR_PASSWORD"

BUILD_TAG="$(date +%Y%m%d%H%M%S)"
ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"
ADMIN_LATEST="${ACR_LOGIN_SERVER}/admin-site:latest"
CUSTOMER_LATEST="${ACR_LOGIN_SERVER}/customer-site:latest"

# ── Build and push ────────────────────────────────────────────────────────────
section "Building images (tag: $BUILD_TAG)"
DOCKER_BUILDKIT=1 docker build \
    --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
    --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
    -f "$SCRIPT_DIR/Dockerfile.admin-site" \
    -t "$ADMIN_IMAGE" -t "$ADMIN_LATEST" "$REPO_ROOT"
DOCKER_BUILDKIT=1 docker build \
    --build-arg "VITE_ADMIN_API_URL=${ADMIN_URL}" \
    --build-arg "VITE_CUSTOMER_API_URL=${CUSTOMER_URL}" \
    -f "$SCRIPT_DIR/Dockerfile.customer-site" \
    -t "$CUSTOMER_IMAGE" -t "$CUSTOMER_LATEST" "$REPO_ROOT"

docker push "$ADMIN_IMAGE" && docker push "$ADMIN_LATEST"
docker push "$CUSTOMER_IMAGE" && docker push "$CUSTOMER_LATEST"
info "Images pushed: $BUILD_TAG"

# ── DB Migrations ──────────────────────────────────────────────────────────────
section "Database migrations"
if command -v sqlx &>/dev/null; then
    DB_URL="${DATABASE_URL:-}"
    if [[ -z "$DB_URL" ]]; then
        warn "DATABASE_URL not set — fetching from Key Vault..."
        KV="${KEY_VAULT:-${WEB_APP_NAME_PREFIX}-kv}"
        DB_URL=$(az keyvault secret show --vault-name "$KV" --name "DatabaseUrl" --query value -o tsv 2>/dev/null || true)
    fi
    if [[ -n "$DB_URL" ]]; then
        (cd "$REPO_ROOT/crates/data" && DATABASE_URL="$DB_URL" sqlx migrate run)
        info "Migrations complete."
    else
        warn "Could not determine DATABASE_URL — run migrations manually"
    fi
else
    warn "sqlx CLI not found — run migrations manually: cd crates/data && sqlx migrate run"
fi

# ── Update Web Apps ────────────────────────────────────────────────────────────
section "Deploying to Web Apps"
az webapp config container set \
    --name "$ADMIN_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --docker-custom-image-name "$ADMIN_IMAGE" \
    --docker-registry-server-url "https://${ACR_LOGIN_SERVER}" \
    --docker-registry-server-user "$ACR_USERNAME" \
    --docker-registry-server-password "$ACR_PASSWORD" -o none
az webapp restart --name "$ADMIN_APP" --resource-group "$RESOURCE_GROUP" -o none
info "Admin Web App updated and restarted: $ADMIN_URL"

az webapp config container set \
    --name "$CUSTOMER_APP" \
    --resource-group "$RESOURCE_GROUP" \
    --docker-custom-image-name "$CUSTOMER_IMAGE" \
    --docker-registry-server-url "https://${ACR_LOGIN_SERVER}" \
    --docker-registry-server-user "$ACR_USERNAME" \
    --docker-registry-server-password "$ACR_PASSWORD" -o none
az webapp restart --name "$CUSTOMER_APP" --resource-group "$RESOURCE_GROUP" -o none
info "Customer Web App updated and restarted: $CUSTOMER_URL"

section "Upgrade complete"
echo ""
echo -e "${GREEN}✅  Upgraded to tag: ${BUILD_TAG}${NC}"
echo "  Admin:    $ADMIN_URL"
echo "  Customer: $CUSTOMER_URL"
echo "  Log:      $UPGRADE_LOG"
