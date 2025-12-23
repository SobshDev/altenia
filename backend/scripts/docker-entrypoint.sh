#!/bin/bash
set -e

echo "========================================"
echo "  Altenia Backend Startup"
echo "========================================"

# Run migrations
/app/scripts/run-migrations.sh

echo ""
echo "==> Starting Altenia backend server..."
exec /app/altenia-backend
