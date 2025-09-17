#!/bin/bash

# Environment Switcher Script for DeFlow
# Usage: ./switch-environment.sh [local|mainnet]

set -e

ENV=${1:-"local"}

echo "🔄 Switching DeFlow to $ENV environment..."

case $ENV in
  "local")
    echo "Setting up local development environment..."
    
    # Copy local env files
    if [ -f "src/DeFlow_frontend/.env.local" ]; then
      cp src/DeFlow_frontend/.env.local src/DeFlow_frontend/.env
      echo "✅ Frontend configured for local development"
    fi
    
    if [ -f "src/DeFlow_admin/.env.local" ]; then
      cp src/DeFlow_admin/.env.local src/DeFlow_admin/.env
      echo "✅ Admin configured for local development"
    fi
    
    echo ""
    echo "🛠️  Local environment ready!"
    echo "Run: ./deploy-local.sh"
    ;;
    
  "mainnet")
    echo "Setting up mainnet production environment..."
    
    # Remove local env files (use .env.production instead)
    if [ -f "src/DeFlow_frontend/.env" ]; then
      rm src/DeFlow_frontend/.env
      echo "✅ Frontend will use .env.production"
    fi
    
    if [ -f "src/DeFlow_admin/.env" ]; then
      rm src/DeFlow_admin/.env  
      echo "✅ Admin will use .env.production"
    fi
    
    echo ""
    echo "🚀 Mainnet environment ready!"
    echo "Run: ./deploy-mainnet.sh"
    ;;
    
  *)
    echo "❌ Invalid environment: $ENV"
    echo "Usage: ./switch-environment.sh [local|mainnet]"
    exit 1
    ;;
esac