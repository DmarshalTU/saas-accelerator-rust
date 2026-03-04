#!/usr/bin/env bash
# Smoke test: hit health endpoints of Admin, Customer, and Webhook APIs.
# Start the three APIs first (see docs/TESTING_IN_ACTION.md).

set -e

ADMIN_URL="${ADMIN_URL:-http://localhost:3000}"
CUSTOMER_URL="${CUSTOMER_URL:-http://localhost:3001}"
WEBHOOK_URL="${WEBHOOK_URL:-http://localhost:3002}"

fail() { echo "FAIL: $1"; exit 1; }
ok()   { echo "OK:   $1"; }

echo "Smoke test (Admin=$ADMIN_URL, Customer=$CUSTOMER_URL, Webhook=$WEBHOOK_URL)"
echo ""

# Admin API: GET /health -> 200
code=$(curl -s -o /dev/null -w "%{http_code}" "$ADMIN_URL/health" || true)
if [ "$code" = "200" ]; then
  ok "Admin API /health -> $code"
else
  fail "Admin API /health -> $code (expected 200). Is admin-api running?"
fi

# Customer API: GET /health -> 200
code=$(curl -s -o /dev/null -w "%{http_code}" "$CUSTOMER_URL/health" || true)
if [ "$code" = "200" ]; then
  ok "Customer API /health -> $code"
else
  fail "Customer API /health -> $code (expected 200). Is customer-api running?"
fi

# Customer site webhook: POST /api/webhook/health -> 200 (webhook runs inside customer-api in two-site deploy)
code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$CUSTOMER_URL/api/webhook/health" || true)
if [ "$code" = "200" ]; then
  ok "Customer site POST /api/webhook/health -> $code"
else
  fail "Customer site POST /api/webhook/health -> $code (expected 200). Is customer-api running?"
fi

# Optional: standalone webhook API (only if running)
code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$WEBHOOK_URL/health" 2>/dev/null || echo "000")
if [ "$code" = "200" ]; then
  ok "Standalone webhook API POST /health -> $code"
elif [ "$code" = "000" ]; then
  ok "Standalone webhook not running (optional)"
else
  ok "Standalone webhook returned $code (optional)"
fi

echo ""
echo "All smoke tests passed."
