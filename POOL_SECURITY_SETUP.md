# Pool Canister Security Setup Guide

## Overview
The pool canister now has comprehensive access control to ensure only authorized canisters can perform specific operations.

## Security Architecture

### 🔒 **Access Levels**

1. **Backend Canister Access**
   - Can deposit transaction fees via `deposit_fee()`
   - Rate limited and audited
   - Must be set by owner

2. **Admin Canister Access**
   - Can perform admin operations like subscription payments
   - Pool configuration changes
   - Rate limited and audited

3. **Owner Access**
   - Can set backend and admin canister principals
   - Can add emergency stop principals
   - Can view security audit logs
   - Full control over access control system

4. **Emergency Stop Access**
   - Can pause pool operations in emergencies
   - Must be explicitly granted by owner

## 🚀 **Deployment & Setup Options**

You now have **three ways** to configure pool canister security:

### Option 1: Environment-Based Deployment (Recommended) 🌟

1. **Configure `.env.pool` file:**
```bash
# Edit the .env.pool file with your settings
POOL_BACKEND_CANISTER_PRINCIPAL="rrkah-fqaaa-aaaaa-aaaaq-cai"
POOL_ADMIN_CANISTER_PRINCIPAL="rdmx6-jaaaa-aaaah-qcaiq-cai"
POOL_OWNER_PRINCIPAL="your-owner-principal"
POOL_EMERGENCY_PRINCIPALS="emergency-principal-1,emergency-principal-2"
```

2. **Deploy with auto-configuration:**
```bash
./deploy-pool-with-config.sh
```

### Option 2: Manual Setup Commands

After deploying the pool canister, run these commands as the owner:

#### Set Backend Canister Principal
```bash
dfx canister call DeFlow_pool set_backend_canister '(principal "rrkah-fqaaa-aaaaa-aaaaq-cai")'
```

#### Set Admin Canister Principal
```bash
dfx canister call DeFlow_pool set_admin_canister '(principal "rdmx6-jaaaa-aaaah-qcaiq-cai")'
```

#### Add Emergency Stop Principal (Optional)
```bash
dfx canister call DeFlow_pool add_emergency_stop_principal '(principal "your-emergency-principal-here")'
```

### Option 3: Programmatic Configuration

Apply configuration after deployment:
```bash
dfx canister call DeFlow_pool apply_configuration '(record {
  backend_canister_principal = opt "rrkah-fqaaa-aaaaa-aaaaq-cai";
  admin_canister_principal = opt "rdmx6-jaaaa-aaaah-qcaiq-cai";
  emergency_principals = vec {"emergency-principal-here"};
  max_calls_per_minute = opt (60 : nat32);
  enable_rate_limiting = opt true;
})'
```

### Auto-Discovery Configuration

The pool canister can automatically discover and configure itself from deployed canisters:
```bash
dfx canister call DeFlow_pool auto_configure_from_deployment
```

## 🛡️ **Security Features**

### Rate Limiting
- **Fee Deposits**: Max 60 calls per minute per principal
- **Admin Operations**: Same rate limiting applied
- **Temporary Bans**: 1 hour ban for rate limit violations

### Audit Logging
- All security events are logged with timestamps
- Success and failure attempts tracked
- Only owner can view audit logs

### Access Validation
- Principal format validation
- Anonymous caller rejection
- Management canister blocking
- Rate limit enforcement

## 📊 **Monitoring**

### Security Status Query
```bash
dfx canister call DeFlow_pool get_security_status
```

Returns:
- Backend canister set status
- Admin canister set status
- Emergency stop principals count
- Audit log entry count

### View Current Configuration (Owner Only)
```bash
dfx canister call DeFlow_pool get_current_configuration
```

Shows complete security configuration including all principals and settings.

### View Recent Security Events (Owner Only)
```bash
dfx canister call DeFlow_pool get_security_audit_log '(opt 50)'
```

## ⚠️ **Security Considerations**

1. **Only Owner Can Configure**: Access control setup requires owner principal
2. **No Anonymous Access**: Anonymous principals are blocked
3. **Rate Limited**: All operations are rate limited to prevent abuse
4. **Immutable After Set**: Backend/admin principals should be set once and not changed frequently
5. **Audit Trail**: All security events are permanently logged

## 🧪 **Testing Security**

### Test Unauthorized Access
```bash
# This should fail if called from non-backend canister
dfx canister call DeFlow_pool deposit_fee '(variant {USDC}, 1000000 : nat64, "test_tx_123", principal "rdmx6-jaaaa-aaaah-qcaiq-cai")'
```

### Test Admin Operations
```bash
# This should fail if called from non-admin canister
dfx canister call DeFlow_pool process_subscription_payment '(principal "rdmx6-jaaaa-aaaah-qcaiq-cai", 29.99 : float64)'
```

## 🔧 **Error Messages**

Common security error messages:
- `"Access denied for caller X to perform Y action in method Z"`
- `"Only owner X can perform this action. Caller: Y"`
- `"Rate limit exceeded for caller X. Temporarily banned"`
- `"Cannot set anonymous principal as backend canister"`

## 🏗️ **Integration Example**

In your backend canister's fee collection:

```rust
// This will now work because backend is authorized
let result = InterCanisterManager::call_with_retry(
    pool_canister,
    "deposit_fee",
    (asset, amount, transaction_id, user),
    Some(CallConfig::default()),
).await?;
```

## 🔍 **Troubleshooting**

1. **"Access denied" errors**: Ensure backend/admin canisters are properly set
2. **Rate limiting**: Wait for ban period to expire or implement proper delays
3. **Principal format errors**: Ensure valid principal format (27-63 chars, not anonymous)

The pool canister is now secure and will only accept authorized requests! 🛡️