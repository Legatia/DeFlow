#!/bin/bash
# DeFlow Mainnet Deployment Script
# Ensures proper security configurations for production deployment

set -e

echo "🚀 DeFlow Mainnet Deployment Script"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if user is ready for mainnet deployment
echo ""
print_warning "⚠️  MAINNET DEPLOYMENT CHECKLIST ⚠️"
echo "This will deploy to Internet Computer mainnet with real cycles costs."
echo ""
echo "Pre-deployment requirements:"
echo "✅ Internet Identity principal set as owner in pool canister"
echo "✅ Production environment files configured"
echo "✅ Code reviewed and tested thoroughly"
echo "✅ Sufficient cycles in wallet for deployment"
echo ""
read -p "Are you ready to proceed with mainnet deployment? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_error "Deployment cancelled by user"
    exit 1
fi

# Check required tools
print_status "Checking required tools..."
if ! command -v dfx &> /dev/null; then
    print_error "dfx is not installed. Please install dfinity SDK first."
    exit 1
fi

if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed. Please install Node.js first."
    exit 1
fi

if ! command -v npm &> /dev/null; then
    print_error "npm is not installed. Please install npm first."
    exit 1
fi

print_success "All required tools are available"

# Check wallet balance
print_status "Checking wallet balance..."
WALLET_BALANCE=$(dfx wallet --network ic balance 2>/dev/null || echo "0")
print_status "Current wallet balance: $WALLET_BALANCE"

if [[ "$WALLET_BALANCE" == "0"* ]]; then
    print_warning "Low wallet balance detected. Ensure you have sufficient cycles for deployment."
fi

# Security checks
print_status "Running security checks..."

# Check for development configurations in production files
if grep -r "localhost\|127.0.0.1\|local" src/*/\.env\.production 2>/dev/null; then
    print_error "Found localhost references in production environment files!"
    exit 1
fi

# Check for debug mode enabled
if grep -r "DEBUG_MODE=true\|debug.*true" src/*/\.env\.production 2>/dev/null; then
    print_error "Debug mode is enabled in production environment files!"
    exit 1
fi

print_success "Security checks passed"

# Build all projects for production
print_status "Regenerating Candid interfaces..."
if ! dfx generate; then
    print_error "Candid interface generation failed"
    exit 1
fi

print_status "Building backend canister..."
cd src/DeFlow_backend
if ! cargo build --release --target wasm32-unknown-unknown; then
    print_error "Backend build failed"
    exit 1
fi
cd ../..

print_status "Building pool canister..."
cd src/DeFlow_pool
if ! cargo build --release --target wasm32-unknown-unknown; then
    print_error "Pool build failed"
    exit 1
fi
cd ../..

print_status "Building frontend..."
cd src/DeFlow_frontend
if ! npm run build:mainnet; then
    print_error "Frontend build failed"
    exit 1
fi
cd ../..

print_status "Building admin panel..."
cd src/DeFlow_admin
if ! npm run build:mainnet; then
    print_error "Admin build failed"
    exit 1
fi
cd ../..

print_success "All builds completed successfully"

# Deploy to mainnet
print_status "🚀 Deploying to Internet Computer mainnet..."

# Set network to ic
export DFX_NETWORK=ic

# Deploy canisters with proper initialization
print_status "Deploying pool canister first..."
if ! dfx deploy --network ic DeFlow_pool; then
    print_error "Pool deployment failed"
    exit 1
fi

print_status "Deploying backend canister with pool connection..."
POOL_CANISTER_ID="7id7i-iqaaa-aaaad-abtva-cai"
if ! dfx deploy --network ic DeFlow_backend --argument "(opt \"$POOL_CANISTER_ID\")"; then
    print_error "Backend deployment failed"
    exit 1
fi

print_status "Deploying frontend..."
if ! dfx deploy --network ic DeFlow_frontend; then
    print_error "Frontend deployment failed"
    exit 1
fi

print_status "Deploying admin panel..."
if ! dfx deploy --network ic DeFlow_admin; then
    print_error "Admin deployment failed"
    exit 1
fi

print_success "🎉 All canisters deployed successfully!"

# Get canister URLs
echo ""
print_status "📋 Deployment Summary"
echo "====================="

BACKEND_ID=$(dfx canister --network ic id DeFlow_backend)
POOL_ID=$(dfx canister --network ic id DeFlow_pool)
FRONTEND_ID=$(dfx canister --network ic id DeFlow_frontend)
ADMIN_ID=$(dfx canister --network ic id DeFlow_admin)

echo ""
echo "🔗 Canister URLs:"
echo "Frontend:  https://${FRONTEND_ID}.icp0.io/"
echo "Admin:     https://${ADMIN_ID}.icp0.io/"
echo "Backend:   https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=${BACKEND_ID}"
echo "Pool:      https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=${POOL_ID}"
echo ""
echo "📱 Canister IDs:"
echo "Backend:   ${BACKEND_ID}"
echo "Pool:      ${POOL_ID}"
echo "Frontend:  ${FRONTEND_ID}"
echo "Admin:     ${ADMIN_ID}"

# Post-deployment security checks
print_status "Running post-deployment security verification..."

# Check if canisters are accessible
if curl -s "https://${FRONTEND_ID}.icp0.io/" > /dev/null; then
    print_success "Frontend is accessible"
else
    print_warning "Frontend accessibility check failed"
fi

if curl -s "https://${ADMIN_ID}.icp0.io/" > /dev/null; then
    print_success "Admin panel is accessible"
else
    print_warning "Admin panel accessibility check failed"
fi

# Final instructions
echo ""
print_success "🎉 Mainnet deployment completed successfully!"
echo ""
print_status "📋 Next Steps:"
echo "1. Test frontend functionality at https://${FRONTEND_ID}.icp0.io/"
echo "2. Test admin panel at https://${ADMIN_ID}.icp0.io/"
echo "3. Initialize pool canister with your Internet Identity principal"
echo "4. Update DNS records if using custom domain"
echo "5. Monitor canister cycles consumption"
echo ""
print_status "🔐 Security Reminders:"
echo "• Only access admin panel from secure networks"
echo "• Regularly monitor canister cycles"
echo "• Keep backup of your Internet Identity seed phrase"
echo "• Monitor audit logs for suspicious activity"
echo ""
print_success "Deployment script completed! 🎊"