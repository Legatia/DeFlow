# 🚀 DeFlow Admin - Mainnet Deployment Guide

## ⚠️ CRITICAL: PRODUCTION DEPLOYMENT ONLY

This guide is for deploying the DeFlow admin dashboard to the Internet Computer mainnet. **Only proceed when you are ready for production deployment.**

## 🔒 Security Prerequisites

### 1. Internet Identity Setup
1. **Create Production Identity**: https://identity.ic0.app
2. **Secure Your Anchor**: Write down your recovery phrase
3. **Copy Your Principal**: You'll need this for the deployment

### 2. Cycles Preparation
- **Minimum Required**: 10T cycles per canister (~40T total)
- **Recommended**: 50T+ cycles for production deployment
- **Purchase Cycles**: https://cycles-faucet.dfinity.org/ (testnet) or exchanges (mainnet)

## 📋 Pre-Deployment Checklist

### ✅ Code Security
- [ ] All mock data removed ✅
- [ ] All placeholder principals removed ✅
- [ ] Real canister integration implemented ✅
- [ ] CSP policy configured for production ✅
- [ ] Authentication hardened for mainnet ✅

### ✅ Environment Setup
- [ ] Production environment variables configured
- [ ] Internet Identity principal obtained
- [ ] Cycles wallet funded
- [ ] Network connectivity verified

## 🚀 Deployment Steps

### Step 1: Configure Production Environment

```bash
# Copy production environment template
cp src/DeFlow_admin/.env.production.example src/DeFlow_admin/.env.production

# Edit with your settings
nano src/DeFlow_admin/.env.production
```

**Required Configuration:**
```bash
# YOUR ACTUAL INTERNET IDENTITY PRINCIPAL
VITE_OWNER_PRINCIPAL=your-actual-principal-from-identity-ic0-app

# Production Settings
DFX_NETWORK=ic
NODE_ENV=production
VITE_ENVIRONMENT=production
VITE_INTERNET_IDENTITY_CANISTER_ID=rdmx6-jaaaa-aaaah-qcaiq-cai
```

### Step 2: Deploy to Mainnet

```bash
# Run the automated mainnet deployment script
./deploy-mainnet.sh
```

The script will:
1. ✅ Validate configuration
2. ✅ Deploy backend canisters
3. ✅ Build frontend with production settings
4. ✅ Deploy frontend canisters
5. ✅ Provide access URLs

### Step 3: Post-Deployment Setup

1. **Access Admin Dashboard**:
   ```
   https://YOUR_ADMIN_CANISTER_ID.ic0.app
   ```

2. **Login with Internet Identity**:
   - Click "Login with Internet Identity"
   - Authenticate with your mainnet II
   - Verify you have owner access

3. **Initialize Chain Fusion**:
   ```bash
   # Via Candid interface or admin dashboard
   dfx canister --network ic call DeFlow_pool initialize_chain_fusion
   ```

4. **Activate Pool**:
   ```bash
   # After chain fusion is initialized
   dfx canister --network ic call DeFlow_pool activate_pool
   ```

## 🔐 Security Features (Production Ready)

### ✅ Authentication
- **Real Internet Identity**: No development bypasses
- **Principal Validation**: Only configured owner can access
- **Session Security**: Encrypted session storage with expiration
- **Multi-layer Validation**: Environment and runtime checks

### ✅ Network Security
- **Production CSP**: Restrictive Content Security Policy
- **HTTPS Only**: All connections encrypted
- **Frame Protection**: X-Frame-Options: DENY
- **Content Sniffing Protection**: X-Content-Type-Options: nosniff

### ✅ Canister Security
- **Owner-Only Access**: All admin functions require owner principal
- **Input Validation**: All parameters validated
- **Error Handling**: No sensitive information leaked
- **Audit Logging**: All admin actions logged

## 📊 Expected Production URLs

After deployment, you'll have:

```
🌐 Frontend App:    https://{frontend-id}.ic0.app
🔧 Admin Dashboard: https://{admin-id}.ic0.app
🔍 Pool Candid:     https://{pool-id}.ic0.app/_/candid
🔍 Backend Candid:  https://{backend-id}.ic0.app/_/candid
```

## 🎯 Post-Deployment Monitoring

### 1. Canister Health
```bash
# Check canister status
dfx canister --network ic status --all

# Monitor cycles
dfx wallet --network ic balance
```

### 2. Functionality Testing
- [ ] Admin login works
- [ ] Treasury data loads
- [ ] Pool analytics display
- [ ] Team withdrawals function
- [ ] All security checks pass

### 3. Performance Monitoring
- [ ] Response times < 2 seconds
- [ ] Memory usage stable
- [ ] No error rates
- [ ] CSP policies working

## ⚠️ Production Warnings

### 🚨 Critical Reminders
- **ONE-TIME SETUP**: Owner principal cannot be changed after deployment
- **BACKUP IDENTITY**: Secure your Internet Identity anchor
- **CYCLES MONITORING**: Set up cycle monitoring alerts
- **ACCESS CONTROL**: Only you can access admin functions

### 🔒 Security Best Practices
- **Regular Updates**: Keep canisters updated
- **Cycle Management**: Monitor and top up cycles
- **Access Logging**: Review admin access logs
- **Backup Procedures**: Document recovery procedures

## 🆘 Troubleshooting

### Common Issues

**1. "Owner principal not configured"**
- Check `.env.production` has correct `VITE_OWNER_PRINCIPAL`
- Verify principal matches your Internet Identity

**2. "Cannot connect to canister"**
- Ensure canisters are deployed: `dfx canister --network ic status --all`
- Check canister IDs in environment variables

**3. "Internet Identity login fails"**
- Verify you're using https://identity.ic0.app
- Check browser doesn't block popups
- Ensure network connection stable

**4. "CSP violations"**
- Check `.ic-assets.json5` is properly configured
- Verify production CSP allows necessary domains

### Getting Help
- **IC Forum**: https://forum.dfinity.org/
- **IC Discord**: https://discord.gg/cA7y6ezyE2
- **Documentation**: https://internetcomputer.org/docs/

## 🎉 Success Checklist

After successful deployment:
- [ ] ✅ All canisters deployed and healthy
- [ ] ✅ Admin dashboard accessible via HTTPS
- [ ] ✅ Internet Identity login working
- [ ] ✅ Real treasury data displaying
- [ ] ✅ All admin functions operational
- [ ] ✅ Security policies active
- [ ] ✅ Cycle monitoring set up

**🚀 Congratulations! Your DeFlow admin dashboard is now live on the Internet Computer mainnet!**