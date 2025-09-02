#!/bin/bash

# DeFlow Local Development Deployment Script

set -e  # Exit on any error

echo "🛠️ DeFlow Local Development Deployment"
echo "====================================="

# Start dfx if not running
if ! dfx ping >/dev/null 2>&1; then
    echo "Starting dfx..."
    dfx start --clean --background
fi

# Deploy all canisters locally
echo "🔨 Deploying to local development environment..."

# Deploy in correct order
dfx deploy DeFlow_pool
dfx deploy DeFlow_backend
dfx deploy DeFlow_frontend
dfx deploy DeFlow_admin

# Get canister IDs
POOL_ID=$(dfx canister id DeFlow_pool)
BACKEND_ID=$(dfx canister id DeFlow_backend)
FRONTEND_ID=$(dfx canister id DeFlow_frontend)
ADMIN_ID=$(dfx canister id DeFlow_admin)

echo ""
echo "✅ LOCAL DEPLOYMENT SUCCESSFUL!"
echo "=============================="
echo "📋 Local Canister IDs:"
echo "  Pool:      $POOL_ID"
echo "  Backend:   $BACKEND_ID"
echo "  Frontend:  $FRONTEND_ID"
echo "  Admin:     $ADMIN_ID"
echo ""
echo "🌐 Local URLs:"
echo "  Frontend:  http://$FRONTEND_ID.localhost:4943"
echo "  Admin:     http://$ADMIN_ID.localhost:4943"
echo "  Candid UI: http://localhost:4943/?canisterId=$BACKEND_ID"
echo ""
echo "🔧 Development Commands:"
echo "  npm run dev    # Start frontend dev server"
echo "  dfx logs       # View canister logs"
echo "  dfx stop       # Stop local replica"