#!/bin/bash

# DeFlow Mainnet Deployment Script
# CRITICAL: Only run this when ready for production deployment

set -e  # Exit on any error

echo "🚨 MAINNET DEPLOYMENT - PRODUCTION ONLY"
echo "========================================="

# Verify we're ready for mainnet
read -p "⚠️  Are you sure you want to deploy to MAINNET? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "❌ Deployment cancelled"
    exit 1
fi

# Check if we have production environment
if [ ! -f "src/DeFlow_admin/.env.production" ]; then
    echo "❌ Missing .env.production file!"
    echo "📝 Please copy .env.production.example and configure your mainnet settings"
    exit 1
fi

# Load production environment
echo "🔧 Loading production environment..."
source src/DeFlow_admin/.env.production

# Validate critical environment variables
if [ -z "$VITE_OWNER_PRINCIPAL" ] || [ "$VITE_OWNER_PRINCIPAL" = "YOUR_MAINNET_PRINCIPAL_HERE" ]; then
    echo "❌ VITE_OWNER_PRINCIPAL not configured in .env.production"
    echo "📝 Please set your mainnet Internet Identity principal"
    exit 1
fi

echo "📋 Production Configuration:"
echo "  Owner Principal: $VITE_OWNER_PRINCIPAL"
echo "  Network: $DFX_NETWORK"
echo "  Environment: $VITE_ENVIRONMENT"

# Confirm final time
read -p "🚀 Deploy to mainnet with above settings? (yes/no): " final_confirm
if [ "$final_confirm" != "yes" ]; then
    echo "❌ Deployment cancelled"
    exit 1
fi

# Switch to mainnet
echo "🌐 Switching to mainnet..."
export DFX_NETWORK=ic

# Deploy backend canisters first
echo "🔨 Deploying backend canisters to mainnet..."

# Deploy pool first
dfx deploy --network ic DeFlow_pool

# Get pool ID and set it as environment variable for backend deployment
POOL_ID=$(dfx canister --network ic id DeFlow_pool)
echo "✅ Pool canister deployed: $POOL_ID"

# Deploy backend with pool canister ID
POOL_CANISTER_ID=$POOL_ID dfx deploy --network ic DeFlow_backend

# Get deployed canister IDs
POOL_ID=$(dfx canister --network ic id DeFlow_pool)
BACKEND_ID=$(dfx canister --network ic id DeFlow_backend)

echo "✅ Backend canisters deployed:"
echo "  Pool: $POOL_ID"
echo "  Backend: $BACKEND_ID"

# Update environment with actual canister IDs
echo "📝 Updating environment with deployed canister IDs..."
sed -i.bak "s/<ACTUAL_MAINNET_POOL_ID>/$POOL_ID/g" src/DeFlow_admin/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_BACKEND_ID>/$BACKEND_ID/g" src/DeFlow_admin/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_POOL_ID>/$POOL_ID/g" src/DeFlow_frontend/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_BACKEND_ID>/$BACKEND_ID/g" src/DeFlow_frontend/.env.production
sed -i.bak "s/<MAINNET_POOL_ID>/$POOL_ID/g" src/DeFlow_backend/.env.production

# Build admin with production settings
cd src/DeFlow_admin
echo "🔨 Building admin frontend for production..."

# Export all production variables
export VITE_OWNER_PRINCIPAL="$VITE_OWNER_PRINCIPAL"
export VITE_CANISTER_ID_DEFLOW_POOL="$POOL_ID"
export VITE_CANISTER_ID_DEFLOW_BACKEND="$BACKEND_ID"
export VITE_INTERNET_IDENTITY_CANISTER_ID="rdmx6-jaaaa-aaaah-qcaiq-cai"
export DFX_NETWORK="ic"
export NODE_ENV="production"

npm run build

# Return to root and deploy admin
cd ../..
echo "🚀 Deploying admin canister to mainnet..."
dfx deploy --network ic DeFlow_admin DeFlow_frontend

# Get all canister IDs
ADMIN_ID=$(dfx canister --network ic id DeFlow_admin)
FRONTEND_ID=$(dfx canister --network ic id DeFlow_frontend)

# Final update of environment files with remaining IDs
sed -i.bak "s/<ACTUAL_MAINNET_ADMIN_ID>/$ADMIN_ID/g" src/DeFlow_admin/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_FRONTEND_ID>/$FRONTEND_ID/g" src/DeFlow_admin/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_ADMIN_ID>/$ADMIN_ID/g" src/DeFlow_frontend/.env.production
sed -i.bak "s/<ACTUAL_MAINNET_FRONTEND_ID>/$FRONTEND_ID/g" src/DeFlow_frontend/.env.production

echo ""
echo "🎉 MAINNET DEPLOYMENT SUCCESSFUL!"
echo "=================================="
echo "📋 Deployed Canister IDs:"
echo "  Pool:      $POOL_ID"
echo "  Backend:   $BACKEND_ID"
echo "  Frontend:  $FRONTEND_ID"
echo "  Admin:     $ADMIN_ID"
echo ""
echo "🌐 Access URLs:"
echo "  Frontend:  https://$FRONTEND_ID.ic0.app"
echo "  Admin:     https://$ADMIN_ID.ic0.app"
echo ""
echo "🔐 Next Steps:"
echo "1. Test Internet Identity login on admin dashboard"
echo "2. Initialize chain fusion: call initialize_chain_fusion()"
echo "3. Activate pool: call activate_pool()"
echo "4. Monitor canister cycles and performance"
echo ""
echo "⚠️  SECURITY REMINDER:"
echo "   - Only YOU can access the admin dashboard (owner principal)"
echo "   - Keep your Internet Identity secure"
echo "   - Monitor canister cycles regularly"