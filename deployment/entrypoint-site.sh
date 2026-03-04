#!/bin/sh
set -e
BIN="${1:-admin-api}"
[ -x "/app/$BIN" ] || { echo "Missing /app/$BIN"; exit 1; }
/app/$BIN &
exec nginx -g "daemon off;"
