#!/bin/bash

# Deploy Pool Canister with Environment-Based Configuration
# This script deploys the pool canister and automatically configures it
# using either the .env.pool file or explicit configuration parameters

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration file path
ENV_FILE=".env.pool"

echo -e "${BLUE}🚀 DeFlow Pool Canister Deployment with Auto-Configuration${NC}"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "src/DeFlow_pool/src/lib.rs" ]; then
    echo -e "${RED}❌ Error: Must run from project root directory${NC}"
    exit 1
fi

# Load environment configuration if available
if [ -f "$ENV_FILE" ]; then
    echo -e "${GREEN}📄 Loading configuration from $ENV_FILE${NC}"
    source "$ENV_FILE"
else
    echo -e "${YELLOW}⚠️  No $ENV_FILE found, will use auto-discovery${NC}"
fi

# Function to get canister ID
get_canister_id() {
    local canister_name=$1
    if [ -f ".dfx/local/canister_ids.json" ]; then
        cat .dfx/local/canister_ids.json | grep -A 1 "\"$canister_name\"" | grep "local" | cut -d'"' -f4
    fi
}

# Deploy the pool canister
echo -e "${BLUE}📦 Deploying Pool Canister...${NC}"

if [ -n "$POOL_OWNER_PRINCIPAL" ]; then
    echo -e "${GREEN}👤 Using owner principal from config: $POOL_OWNER_PRINCIPAL${NC}"
    # Deploy with owner configuration
    dfx deploy DeFlow_pool --argument "(opt record {
        owner_principal = opt \"$POOL_OWNER_PRINCIPAL\";
        backend_canister_principal = $([ -n "$POOL_BACKEND_CANISTER_PRINCIPAL" ] && echo "opt \"$POOL_BACKEND_CANISTER_PRINCIPAL\"" || echo "null");
        admin_canister_principal = $([ -n "$POOL_ADMIN_CANISTER_PRINCIPAL" ] && echo "opt \"$POOL_ADMIN_CANISTER_PRINCIPAL\"" || echo "null");
        emergency_principals = vec {};
        authorized_fee_collectors = vec {};
        readonly_access = vec {};
        max_calls_per_minute = opt (${POOL_MAX_CALLS_PER_MINUTE:-60} : nat32);
        ban_duration_hours = opt (${POOL_BAN_DURATION_HOURS:-1} : nat32);
        enable_rate_limiting = opt ${POOL_ENABLE_RATE_LIMITING:-true};
        enable_audit_logging = opt ${POOL_ENABLE_AUDIT_LOGGING:-true};
        enable_emergency_stop = opt ${POOL_ENABLE_EMERGENCY_STOP:-true};
        dev_mode = opt ${POOL_DEV_MODE:-true};
    })"
else
    echo -e "${YELLOW}🔄 Deploying with auto-discovery (no owner specified)${NC}"
    dfx deploy DeFlow_pool
fi

# Get the deployed pool canister ID
POOL_CANISTER_ID=$(get_canister_id "DeFlow_pool")

if [ -z "$POOL_CANISTER_ID" ]; then
    echo -e "${RED}❌ Failed to get pool canister ID${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Pool canister deployed: $POOL_CANISTER_ID${NC}"

# Wait a moment for initialization to complete
sleep 2

# Check if auto-configuration is enabled
if [ "${POOL_AUTO_CONFIGURE:-true}" == "true" ]; then
    echo -e "${BLUE}🔧 Running auto-configuration from deployed canisters...${NC}"

    # Try auto-configuration
    if dfx canister call DeFlow_pool auto_configure_from_deployment 2>/dev/null; then
        echo -e "${GREEN}✅ Auto-configuration completed successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Auto-configuration failed, will try manual setup${NC}"

        # Fallback to manual configuration if needed
        BACKEND_ID=$(get_canister_id "DeFlow_backend")
        ADMIN_ID=$(get_canister_id "DeFlow_admin")

        if [ -n "$BACKEND_ID" ]; then
            echo -e "${BLUE}🔗 Setting backend canister: $BACKEND_ID${NC}"
            dfx canister call DeFlow_pool set_backend_canister "(principal \"$BACKEND_ID\")" || echo -e "${YELLOW}⚠️  Failed to set backend canister${NC}"
        fi

        if [ -n "$ADMIN_ID" ]; then
            echo -e "${BLUE}🔗 Setting admin canister: $ADMIN_ID${NC}"
            dfx canister call DeFlow_pool set_admin_canister "(principal \"$ADMIN_ID\")" || echo -e "${YELLOW}⚠️  Failed to set admin canister${NC}"
        fi
    fi
fi

# Display security status
echo -e "${BLUE}🔍 Checking security status...${NC}"
dfx canister call DeFlow_pool get_security_status

# Display current configuration (only works if called by owner)
echo -e "${BLUE}📋 Current configuration:${NC}"
dfx canister call DeFlow_pool get_current_configuration 2>/dev/null || echo -e "${YELLOW}ℹ️  Configuration details are owner-only${NC}"

# Final summary
echo "=================================================="
echo -e "${GREEN}🎉 Pool Canister Deployment Complete!${NC}"
echo ""
echo "📌 Canister Details:"
echo "   Pool Canister ID: $POOL_CANISTER_ID"
echo "   Network: ${DFX_NETWORK:-local}"
echo "   Security Features: Rate limiting, Audit logging, Access control"
echo ""

if [ -f "$ENV_FILE" ]; then
    echo "📝 Configuration loaded from: $ENV_FILE"
else
    echo "🔄 Configuration auto-discovered from deployed canisters"
fi

echo ""
echo "🔧 Available Commands:"
echo "   dfx canister call DeFlow_pool get_security_status"
echo "   dfx canister call DeFlow_pool get_current_configuration"
echo "   dfx canister call DeFlow_pool get_security_audit_log '(opt 10)'"
echo ""
echo -e "${GREEN}✨ Pool canister is ready for secure operation!${NC}"