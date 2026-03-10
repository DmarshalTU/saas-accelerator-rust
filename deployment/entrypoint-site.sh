#!/bin/sh
set -e
BIN="${1:-admin-api}"
[ -x "/app/$BIN" ] || { echo "ERROR: Missing /app/$BIN"; exit 1; }

echo "[entrypoint] Starting /app/$BIN..."
/app/$BIN &
API_PID=$!

# Give the API a moment to start, then verify it didn't immediately crash
sleep 3
if ! kill -0 "$API_PID" 2>/dev/null; then
    echo "ERROR: /app/$BIN exited immediately — check environment variables and DB connectivity"
    exit 1
fi

echo "[entrypoint] $BIN running as PID $API_PID, starting nginx..."
exec nginx -g "daemon off;"
