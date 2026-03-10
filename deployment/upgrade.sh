#!/usr/bin/env bash
# =============================================================================
# SaaS Accelerator – Rust Edition  |  Upgrade Script
# Rebuilds and redeploys containers to existing Admin + Customer Web Apps.
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
section() { echo -e "\n${CYAN}══ $* ══${NC}"; echo -e "\n═══ $* ═══" >> "$UPGRADE_LOG"; }
die()     { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
on_error() { echo -e "${RED}[ERROR]${NC} Upgrade failed at line $1 (exit $2)"; }
trap 'on_error $LINENO $?' ERR

# ── load .env ─────────────────────────────────────────────────────────────────
ENV_FILE="$SCRIPT_DIR/.env"
[[ -f "$ENV_FILE" ]] && { info "Loading $ENV_FILE"; set -a; source "$ENV_FILE"; set +a; }

# ── parameters ────────────────────────────────────────────────────────────────
WEB_APP_NAME_PREFIX="${WEB_APP_NAME_PREFIX:-}"
RESOURCE_GROUP="${RESOURCE_GROUP:-}"
AZURE_SUBSCRIPTION_ID="${AZURE_SUBSCRIPTION_ID:-}"
ACR_NAME="${ACR_NAME:-}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)         WEB_APP_NAME_PREFIX="$2"; shift 2 ;;
        --resource-group) RESOURCE_GROUP="$2"; shift 2 ;;
        --subscription)   AZURE_SUBSCRIPTION_ID="$2"; shift 2 ;;
        --acr)            ACR_NAME="$2"; shift 2 ;;
        *) die "Unknown option: $1" ;;
    esac
done

[[ -z "$WEB_APP_NAME_PREFIX" ]] && die "--prefix is required"
RESOURCE_GROUP="${RESOURCE_GROUP:-$WEB_APP_NAME_PREFIX}"
ACR_NAME="${ACR_NAME:-${WEB_APP_NAME_PREFIX//-/}acr}"
ADMIN_APP="${WEB_APP_NAME_PREFIX}-admin"
CUSTOMER_APP="${WEB_APP_NAME_PREFIX}-portal"
ADMIN_URL="https://${ADMIN_APP}.azurewebsites.net"
CUSTOMER_URL="https://${CUSTOMER_APP}.azurewebsites.net"

section "Upgrade configuration"
echo "  Prefix:         $WEB_APP_NAME_PREFIX"
echo "  Resource group: $RESOURCE_GROUP"
echo "  ACR:            $ACR_NAME"

command -v az &>/dev/null || die "'az' not found."
[[ -n "$AZURE_SUBSCRIPTION_ID" ]] && az account set --subscription "$AZURE_SUBSCRIPTION_ID"

# ── Resolve ACR credentials ────────────────────────────────────────────────────
section "ACR"
ACR_LOGIN_SERVER=$(az acr show --name "$ACR_NAME" --query loginServer -o tsv)
ACR_USER=$(az acr credential show --name "$ACR_NAME" --query username -o tsv)
ACR_PASS=$(az acr credential show --name "$ACR_NAME" --query passwords[0].value -o tsv)
info "ACR: $ACR_LOGIN_SERVER"

BUILD_TAG="$(date +%Y%m%d%H%M%S)"
ADMIN_IMAGE="${ACR_LOGIN_SERVER}/admin-site:${BUILD_TAG}"
CUSTOMER_IMAGE="${ACR_LOGIN_SERVER}/customer-site:${BUILD_TAG}"

# ── Build and push ─────────────────────────────────────────────────────────────
section "Building images (tag: $BUILD_TAG)"

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

    docker push "$ADMIN_IMAGE"
    docker push "${ACR_LOGIN_SERVER}/admin-site:latest"
    docker push "$CUSTOMER_IMAGE"
    docker push "${ACR_LOGIN_SERVER}/customer-site:latest"
else
    info "No Docker daemon — using az acr build..."

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
fi

info "Images pushed: $BUILD_TAG"

# ── Update Web Apps ────────────────────────────────────────────────────────────
section "Deploying to Web Apps"

for app in "$ADMIN_APP" "$CUSTOMER_APP"; do
    image="${ACR_LOGIN_SERVER}/$([ "$app" = "$ADMIN_APP" ] && echo "admin-site" || echo "customer-site"):${BUILD_TAG}"
    info "  Updating $app → $image"
    az webapp config appsettings set \
        --name "$app" \
        --resource-group "$RESOURCE_GROUP" \
        --settings \
            "DOCKER_REGISTRY_SERVER_URL=https://${ACR_LOGIN_SERVER}" \
            "DOCKER_REGISTRY_SERVER_USERNAME=${ACR_USER}" \
            "DOCKER_REGISTRY_SERVER_PASSWORD=${ACR_PASS}" \
            "WEBSITE_DNS_SERVER=168.63.129.16" \
            "WEBSITES_PORT=80" -o none
    az webapp config set \
        --name "$app" \
        --resource-group "$RESOURCE_GROUP" \
        --linux-fx-version "DOCKER|${image}" -o none
    az webapp restart --name "$app" --resource-group "$RESOURCE_GROUP" -o none
    info "  $app restarted."
done

section "Upgrade complete"
echo ""
echo -e "${GREEN}✅  Upgraded to tag: ${BUILD_TAG}${NC}"
echo "  Admin:    $ADMIN_URL"
echo "  Customer: $CUSTOMER_URL"
echo "  Log:      $UPGRADE_LOG"
