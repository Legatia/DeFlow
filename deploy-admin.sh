#!/bin/bash

# DeFlow Admin Deployment Script
# This ensures environment variables are properly loaded during build

echo "🚀 Deploying DeFlow Admin with proper environment setup..."

# Change to admin directory
cd src/DeFlow_admin

# Export environment variables
export VITE_OWNER_PRINCIPAL="3nubw-txlgy-w47fs-o7fwl-a4xnt-2mmu2-q5sl3-jswu2-4xsyj-fbsbp-vqe"
export VITE_CANISTER_ID_DEFLOW_POOL="umunu-kh777-77774-qaaca-cai"
export VITE_CANISTER_ID_DEFLOW_BACKEND="u6s2n-gx777-77774-qaaba-cai"
export VITE_CANISTER_ID_DEFLOW_FRONTEND="uzt4z-lp777-77774-qaabq-cai"
export VITE_CANISTER_ID_DEFLOW_ADMIN="uxrrr-q7777-77774-qaaaq-cai"
export DFX_NETWORK="local"

echo "📋 Environment variables:"
echo "  VITE_OWNER_PRINCIPAL=$VITE_OWNER_PRINCIPAL"
echo "  DFX_NETWORK=$DFX_NETWORK"

# Build with environment variables
echo "🔨 Building admin frontend..."
npm run build

# Go back to root and deploy
cd ../..
echo "🚀 Deploying to local network..."
dfx deploy DeFlow_admin

echo "✅ Deployment complete!"
echo "🌐 Access: http://uxrrr-q7777-77774-qaaaq-cai.localhost:8080/"