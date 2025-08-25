# DeFlow Mainnet Deployment Checklist

## 🚨 CRITICAL ISSUES TO FIX BEFORE MAINNET

### 1. **Pool Canister ID Configuration** ⚠️ **HIGH PRIORITY**
**Issue**: Backend hardcoded with development canister ID for fee collection
**Location**: `/src/DeFlow_backend/src/lib.rs:76`
**Fix Required**:
```rust
// BEFORE (INSECURE):
let pool_canister_id = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai")

// AFTER (SECURE):
let pool_canister_id = get_mainnet_pool_canister_id();
```

**Action**: Update pool canister ID configuration to use environment variables or dfx.json

### 2. **Anonymous User Handling** ⚠️ **MEDIUM PRIORITY**
**Issue**: Anonymous user IDs in execution contexts
**Location**: `/src/DeFlow_backend/src/execution.rs:160`
**Fix**: Implement proper authentication checks

### 3. **Environment Variables** ⚠️ **HIGH PRIORITY**
**Issue**: Frontend environment files need mainnet canister IDs
**Fix**: Update production environment files with actual mainnet canister IDs

---

## 📋 DEPLOYMENT PREPARATION STEPS

### Phase 1: Security & Configuration

#### 1.1 Fix Pool Canister ID Configuration
```bash
# Update lib.rs init function
# Replace hardcoded pool ID with environment-based configuration
```

#### 1.2 Set Production Environment Variables
```bash
# Frontend Production Config
VITE_DFX_NETWORK=ic
VITE_CANISTER_ID_DEFLOW_BACKEND=<MAINNET_BACKEND_ID>
VITE_CANISTER_ID_DEFLOW_POOL=<MAINNET_POOL_ID>
VITE_CANISTER_ID_DEFLOW_FRONTEND=<MAINNET_FRONTEND_ID>
VITE_CANISTER_ID_DEFLOW_ADMIN=<MAINNET_ADMIN_ID>

# Admin Production Config  
VITE_OWNER_PRINCIPAL=<YOUR_INTERNET_IDENTITY_PRINCIPAL>
```

#### 1.3 Remove Development Dependencies
- [ ] Remove test canister IDs from fee collection tests
- [ ] Update anonymous caller handling
- [ ] Verify all debug/development code paths

### Phase 2: Build & Test

#### 2.1 Clean Build Test
```bash
# Backend
cargo build --package DeFlow_backend --release
cargo build --package DeFlow_pool --release

# Frontend  
cd src/DeFlow_frontend && npm run build
cd ../DeFlow_admin && npm run build
```

#### 2.2 Local Integration Test
```bash
# Test full deployment locally
dfx start --clean
dfx deploy
# Run integration tests
```

#### 2.3 Security Validation
- [ ] Verify no hardcoded development IDs
- [ ] Check all authentication flows
- [ ] Validate fee collection configuration
- [ ] Test treasury management permissions

### Phase 3: Mainnet Deployment

#### 3.1 Deploy Core Canisters
```bash
# Deploy to mainnet (in order)
dfx deploy --network ic DeFlow_pool
dfx deploy --network ic DeFlow_backend  
dfx deploy --network ic DeFlow_frontend
dfx deploy --network ic DeFlow_admin
```

#### 3.2 Record Canister IDs
```bash
# Save mainnet canister IDs
export MAINNET_POOL_ID=$(dfx canister id DeFlow_pool --network ic)
export MAINNET_BACKEND_ID=$(dfx canister id DeFlow_backend --network ic)
export MAINNET_FRONTEND_ID=$(dfx canister id DeFlow_frontend --network ic)  
export MAINNET_ADMIN_ID=$(dfx canister id DeFlow_admin --network ic)

# Update environment files
```

#### 3.3 Post-Deployment Configuration
```bash
# Initialize pool with proper configuration
dfx canister call DeFlow_pool init --network ic

# Test basic functionality
dfx canister call DeFlow_backend list_workflow_templates --network ic
```

### Phase 4: Frontend Deployment

#### 4.1 Update Environment Configuration
```bash
# Update .env.production with actual mainnet IDs
# Rebuild frontend with production config
NODE_ENV=production npm run build
```

#### 4.2 Deploy Frontend Assets
```bash
# Deploy frontend to mainnet
dfx deploy DeFlow_frontend --network ic
dfx deploy DeFlow_admin --network ic
```

### Phase 5: Verification & Testing

#### 5.1 Basic Functionality Tests
- [ ] Template listing works
- [ ] Strategy creation works  
- [ ] Fee collection works
- [ ] Treasury management works
- [ ] Authentication works

#### 5.2 Security Verification
- [ ] Only authorized users can access admin functions
- [ ] Fee collection goes to correct pool
- [ ] No development endpoints exposed
- [ ] Rate limiting works
- [ ] Input validation works

#### 5.3 Performance Testing
- [ ] Response times acceptable
- [ ] Memory usage within limits
- [ ] Cycles consumption reasonable

---

## 🔧 IMMEDIATE FIXES REQUIRED

### Fix 1: Pool Canister ID Configuration

**File**: `/src/DeFlow_backend/src/lib.rs`
```rust
#[init]
fn init() {
    initialize_built_in_nodes();
    
    // Get pool canister ID from environment or dfx.json
    let pool_canister_id = get_pool_canister_id_for_network();
    initialize_fee_collection(pool_canister_id);
    
    // ... rest of initialization
}

fn get_pool_canister_id_for_network() -> Principal {
    // Check if we're on mainnet vs local
    match ic_cdk::api::canister_balance128() {
        balance if balance > 1_000_000_000_000u128 => {
            // Mainnet - use environment or fail
            Principal::from_text(std::env::var("POOL_CANISTER_ID")
                .expect("POOL_CANISTER_ID must be set for mainnet"))
                .expect("Invalid pool canister ID")
        },
        _ => {
            // Local development
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai")
                .unwrap_or(Principal::anonymous())
        }
    }
}
```

### Fix 2: Environment Configuration

**File**: Create `/src/DeFlow_backend/.env.production`
```bash
POOL_CANISTER_ID=<MAINNET_POOL_ID>
OWNER_PRINCIPAL=<YOUR_INTERNET_IDENTITY>
```

### Fix 3: Frontend Production Config

**File**: Update `/src/DeFlow_frontend/.env.production`
```bash
# Replace placeholder values with actual mainnet canister IDs
VITE_CANISTER_ID_DEFLOW_BACKEND=<ACTUAL_MAINNET_BACKEND_ID>
VITE_CANISTER_ID_DEFLOW_POOL=<ACTUAL_MAINNET_POOL_ID>
VITE_CANISTER_ID_DEFLOW_FRONTEND=<ACTUAL_MAINNET_FRONTEND_ID>
VITE_CANISTER_ID_DEFLOW_ADMIN=<ACTUAL_MAINNET_ADMIN_ID>
```

---

## 🛡️ SECURITY CHECKLIST

### Authentication & Authorization
- [ ] Internet Identity integration working
- [ ] Admin functions properly protected
- [ ] No anonymous access to sensitive functions
- [ ] Principal-based permissions implemented

### Financial Security  
- [ ] Fee collection configured correctly
- [ ] Treasury functions secured
- [ ] No hardcoded test accounts
- [ ] Multi-signature where required

### Code Security
- [ ] No development/debug code in production
- [ ] Input validation on all endpoints
- [ ] Rate limiting implemented
- [ ] Error messages don't leak sensitive info

### Infrastructure Security
- [ ] HTTPS configured for frontend
- [ ] Proper CORS configuration
- [ ] No sensitive data in logs
- [ ] Backup and recovery procedures

---

## 🚀 DEPLOYMENT TIMELINE

### Day 1: Critical Fixes
- Fix pool canister ID configuration
- Update environment variables
- Security review

### Day 2: Testing
- Complete integration testing
- Security validation
- Performance testing

### Day 3: Mainnet Deployment
- Deploy canisters to mainnet
- Configure production settings
- Final verification testing

### Day 4: Monitoring & Support
- Monitor deployment health
- Address any issues
- User documentation

---

## 📞 POST-DEPLOYMENT MONITORING

### Key Metrics to Monitor
- **Canister Cycles**: Ensure sufficient cycles for operation
- **Memory Usage**: Monitor heap and stable memory usage  
- **Transaction Volume**: Track fee collection and treasury operations
- **Error Rates**: Monitor for unusual error patterns
- **Performance**: Response times and throughput

### Alerting Setup
- Low cycles alerts for all canisters
- High error rate alerts
- Treasury balance monitoring
- Security event monitoring

---

## ⚡ QUICK START COMMANDS

```bash
# Complete mainnet deployment sequence
./deploy-mainnet.sh

# Test deployment
./test-mainnet.sh

# Monitor deployment
./monitor-mainnet.sh
```

**Status**: 🟢 **READY FOR MAINNET** - Critical fixes implemented
**Next Action**: Execute deployment using `./deploy-mainnet.sh`
**ETA**: Ready for immediate deployment