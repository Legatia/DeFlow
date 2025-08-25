#!/bin/bash

# 🔗 DeFlow Admin-Pool Connection Verification Script

echo "🔍 DeFlow Admin-Pool Connection Verification"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
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

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Step 1: Check if DFX is running
echo -e "\n${BLUE}📡 Step 1: Network Connectivity${NC}"
dfx ping > /dev/null 2>&1
print_status $? "DFX replica is running"

# Step 2: Check canister status
echo -e "\n${BLUE}🏗️  Step 2: Canister Status${NC}"

# Check if canisters exist
dfx canister id DeFlow_pool > /dev/null 2>&1
if [ $? -eq 0 ]; then
    POOL_ID=$(dfx canister id DeFlow_pool)
    print_status 0 "DeFlow_pool canister found: $POOL_ID"
    
    # Check pool canister status
    dfx canister status DeFlow_pool > /dev/null 2>&1
    print_status $? "DeFlow_pool canister is running"
else
    print_status 1 "DeFlow_pool canister not found - run 'dfx deploy'"
    exit 1
fi

dfx canister id DeFlow_admin > /dev/null 2>&1
if [ $? -eq 0 ]; then
    ADMIN_ID=$(dfx canister id DeFlow_admin)
    print_status 0 "DeFlow_admin canister found: $ADMIN_ID"
else
    print_warning "DeFlow_admin canister not deployed yet"
fi

# Step 3: Check pool API endpoints
echo -e "\n${BLUE}🔌 Step 3: Pool API Connectivity${NC}"

# Test basic pool state
POOL_STATE=$(dfx canister call DeFlow_pool get_pool_state 2>/dev/null)
if [ $? -eq 0 ]; then
    print_status 0 "get_pool_state endpoint working"
else
    print_status 1 "get_pool_state endpoint failed"
fi

# Test financial overview
FINANCIAL_OVERVIEW=$(dfx canister call DeFlow_pool get_financial_overview 2>/dev/null)
if [ $? -eq 0 ]; then
    print_status 0 "get_financial_overview endpoint working"
else
    print_status 1 "get_financial_overview endpoint failed"
fi

# Test chain distribution
CHAIN_DIST=$(dfx canister call DeFlow_pool get_chain_distribution 2>/dev/null)
if [ $? -eq 0 ]; then
    print_status 0 "get_chain_distribution endpoint working"
else
    print_status 1 "get_chain_distribution endpoint failed"
fi

# Step 4: Check admin authentication
echo -e "\n${BLUE}🔐 Step 4: Admin Authentication${NC}"

# Check if user is pool owner
CURRENT_PRINCIPAL=$(dfx identity get-principal)
print_info "Current identity: $CURRENT_PRINCIPAL"

POOL_OWNER=$(dfx canister call DeFlow_pool get_pool_owner 2>/dev/null)
if [ $? -eq 0 ]; then
    print_status 0 "Successfully retrieved pool owner"
    
    # Extract owner principal from result
    OWNER_PRINCIPAL=$(echo $POOL_OWNER | grep -o 'principal "[^"]*"' | cut -d'"' -f2)
    
    if [ "$CURRENT_PRINCIPAL" = "$OWNER_PRINCIPAL" ]; then
        print_status 0 "Current identity is pool owner - full admin access granted"
    else
        print_warning "Current identity is NOT pool owner"
        print_info "Pool owner: $OWNER_PRINCIPAL"
        print_info "To gain admin access, run: dfx canister call DeFlow_pool set_pool_owner '(principal \"$CURRENT_PRINCIPAL\")'"
    fi
else
    print_status 1 "Failed to retrieve pool owner"
fi

# Step 5: Environment check
echo -e "\n${BLUE}⚙️  Step 5: Environment Configuration${NC}"

# Check if admin .env exists
if [ -f "src/DeFlow_admin/.env" ]; then
    print_status 0 "Admin .env file exists"
    
    # Check if it has the pool canister ID
    if grep -q "VITE_CANISTER_ID_DEFLOW_POOL" src/DeFlow_admin/.env; then
        CONFIGURED_POOL_ID=$(grep "VITE_CANISTER_ID_DEFLOW_POOL" src/DeFlow_admin/.env | cut -d'=' -f2)
        if [ "$CONFIGURED_POOL_ID" = "$POOL_ID" ]; then
            print_status 0 "Pool canister ID correctly configured in .env"
        else
            print_warning "Pool canister ID mismatch in .env"
            print_info "Expected: $POOL_ID"
            print_info "Found: $CONFIGURED_POOL_ID"
        fi
    else
        print_warning "Pool canister ID not configured in .env"
    fi
else
    print_warning "Admin .env file not found"
    print_info "Create one with: echo 'VITE_CANISTER_ID_DEFLOW_POOL=$POOL_ID' > src/DeFlow_admin/.env"
fi

# Step 6: Quick setup if needed
echo -e "\n${BLUE}🚀 Step 6: Quick Setup${NC}"

# Offer to create .env if missing
if [ ! -f "src/DeFlow_admin/.env" ]; then
    echo -e "\n${YELLOW}Would you like to create the admin .env file now? (y/n)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        mkdir -p src/DeFlow_admin
        cat > src/DeFlow_admin/.env << EOF
# DeFlow Pool Connection
VITE_CANISTER_ID_DEFLOW_POOL=$POOL_ID
VITE_CANISTER_ID_DEFLOW_BACKEND=$(dfx canister id DeFlow_backend 2>/dev/null || echo "")

# Network Configuration
DFX_NETWORK=local
VITE_DFX_NETWORK=local
VITE_HOST=http://127.0.0.1:8080

# Admin Authentication
VITE_ADMIN_MODE=true
VITE_ENVIRONMENT=development
EOF
        print_status 0 "Created admin .env file"
    fi
fi

# Offer to set as pool owner if not already
if [ "$CURRENT_PRINCIPAL" != "$OWNER_PRINCIPAL" ]; then
    echo -e "\n${YELLOW}Would you like to set yourself as pool owner for admin access? (y/n)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        dfx canister call DeFlow_pool set_pool_owner "(principal \"$CURRENT_PRINCIPAL\")" > /dev/null 2>&1
        if [ $? -eq 0 ]; then
            print_status 0 "Successfully set as pool owner"
        else
            print_status 1 "Failed to set as pool owner"
        fi
    fi
fi

# Step 7: Launch instructions
echo -e "\n${BLUE}🎯 Step 7: Launch Instructions${NC}"

print_info "To start the admin frontend:"
echo -e "${GREEN}cd src/DeFlow_admin${NC}"
echo -e "${GREEN}npm install${NC}"
echo -e "${GREEN}dfx generate${NC}"
echo -e "${GREEN}npm run dev${NC}"

print_info "Then open: http://localhost:3000"

echo -e "\n${GREEN}🎉 Connection verification complete!${NC}"

# Summary
echo -e "\n${BLUE}📋 Summary${NC}"
echo "Pool Canister: $POOL_ID"
echo "Admin Identity: $CURRENT_PRINCIPAL"
echo "Pool Owner: $OWNER_PRINCIPAL"

if [ "$CURRENT_PRINCIPAL" = "$OWNER_PRINCIPAL" ]; then
    echo -e "${GREEN}✅ Ready for admin access!${NC}"
else
    echo -e "${YELLOW}⚠️  Set pool owner for full admin access${NC}"
fi