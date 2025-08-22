#!/bin/bash

# 🚀 DeFlow Admin Quick Setup Script

echo "🚀 DeFlow Admin Frontend Setup"
echo "=============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        return 1
    fi
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_step() {
    echo -e "\n${BLUE}🔄 $1${NC}"
}

# Step 1: Check prerequisites
print_step "Step 1: Checking Prerequisites"

if ! command -v dfx &> /dev/null; then
    print_status 1 "DFX not found. Please install from https://smartcontracts.org/docs/quickstart/local-quickstart.html"
    exit 1
fi
print_status 0 "DFX found"

if ! command -v npm &> /dev/null; then
    print_status 1 "npm not found. Please install Node.js"
    exit 1
fi
print_status 0 "npm found"

# Step 2: Start DFX if not running
print_step "Step 2: Starting DFX"

if ! dfx ping > /dev/null 2>&1; then
    print_info "Starting DFX replica..."
    dfx start --background --clean
    sleep 5
fi

dfx ping > /dev/null 2>&1
print_status $? "DFX replica running"

# Step 3: Deploy canisters
print_step "Step 3: Deploying Canisters"

# Deploy pool first
print_info "Deploying DeFlow_pool..."
dfx deploy DeFlow_pool
print_status $? "DeFlow_pool deployed"

# Deploy backend
print_info "Deploying DeFlow_backend..."
dfx deploy DeFlow_backend
print_status $? "DeFlow_backend deployed"

# Get canister IDs
POOL_ID=$(dfx canister id DeFlow_pool)
BACKEND_ID=$(dfx canister id DeFlow_backend)

print_info "Pool Canister ID: $POOL_ID"
print_info "Backend Canister ID: $BACKEND_ID"

# Step 4: Set up admin authentication
print_step "Step 4: Setting Up Admin Authentication"

CURRENT_PRINCIPAL=$(dfx identity get-principal)
print_info "Current Principal: $CURRENT_PRINCIPAL"

print_info "Setting you as pool owner..."
dfx canister call DeFlow_pool set_pool_owner "(principal \"$CURRENT_PRINCIPAL\")" > /dev/null 2>&1
print_status $? "Pool owner set"

# Step 5: Create admin environment
print_step "Step 5: Creating Admin Environment"

mkdir -p src/DeFlow_admin

cat > src/DeFlow_admin/.env << EOF
# DeFlow Pool Connection
VITE_CANISTER_ID_DEFLOW_POOL=$POOL_ID
VITE_CANISTER_ID_DEFLOW_BACKEND=$BACKEND_ID

# Network Configuration
DFX_NETWORK=local
VITE_DFX_NETWORK=local
VITE_HOST=http://127.0.0.1:8080

# Admin Authentication
VITE_ADMIN_MODE=true
VITE_ENVIRONMENT=development
EOF

print_status 0 "Environment file created"

# Step 6: Install admin dependencies
print_step "Step 6: Installing Admin Dependencies"

cd src/DeFlow_admin
print_info "Installing npm packages..."
npm install > /dev/null 2>&1
print_status $? "Dependencies installed"

# Step 7: Generate canister declarations
print_step "Step 7: Generating Canister Declarations"

dfx generate > /dev/null 2>&1
print_status $? "Declarations generated"

# Step 8: Add some test data to pool
print_step "Step 8: Adding Test Data"

cd ../..

print_info "Adding test liquidity..."
dfx canister call DeFlow_pool add_liquidity '(variant { Ethereum }, variant { ETH }, 1000000000000000000 : nat64)' > /dev/null 2>&1
print_status $? "Test liquidity added"

print_info "Adding test fee revenue..."
dfx canister call DeFlow_pool add_fee_revenue '(variant { Ethereum }, variant { ETH }, 500000000000000000 : nat64)' > /dev/null 2>&1
print_status $? "Test fee revenue added"

# Step 9: Deploy admin frontend
print_step "Step 9: Building Admin Frontend"

cd src/DeFlow_admin
npm run build > /dev/null 2>&1
if [ $? -eq 0 ]; then
    print_status 0 "Admin frontend built"
    
    cd ../..
    dfx deploy DeFlow_admin > /dev/null 2>&1
    print_status $? "Admin frontend deployed"
    
    ADMIN_ID=$(dfx canister id DeFlow_admin)
    print_info "Admin Canister ID: $ADMIN_ID"
else
    print_status 1 "Admin frontend build failed"
fi

# Step 10: Success summary
print_step "🎉 Setup Complete!"

echo -e "\n${GREEN}✅ DeFlow Admin successfully set up!${NC}"
echo ""
echo -e "${BLUE}📋 Summary:${NC}"
echo "• Pool Canister: $POOL_ID"
echo "• Backend Canister: $BACKEND_ID"
echo "• Admin Canister: ${ADMIN_ID:-"Not deployed"}"
echo "• Admin Principal: $CURRENT_PRINCIPAL"
echo ""
echo -e "${BLUE}🚀 To start development:${NC}"
echo -e "${GREEN}cd src/DeFlow_admin${NC}"
echo -e "${GREEN}npm run dev${NC}"
echo ""
echo -e "${BLUE}🌐 Then open:${NC}"
echo -e "${GREEN}http://localhost:3000${NC}"
echo ""
echo -e "${BLUE}🔗 Or access deployed admin:${NC}"
if [ ! -z "$ADMIN_ID" ]; then
    echo -e "${GREEN}http://127.0.0.1:8080/?canisterId=$ADMIN_ID${NC}"
fi
echo ""
echo -e "${BLUE}📊 Admin Features Available:${NC}"
echo "• 💰 Treasury Management - Real-time balance monitoring"
echo "• 🏊 Pool Management - Liquidity and fee tracking" 
echo "• 📊 System Health - Canister and network status"
echo "• 🔥 Emergency Controls - Pool termination (if needed)"

# Optional: Start dev server
echo ""
echo -e "${YELLOW}Would you like to start the development server now? (y/n)${NC}"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    cd src/DeFlow_admin
    print_info "Starting admin development server..."
    echo -e "${GREEN}Opening http://localhost:3000${NC}"
    npm run dev
fi