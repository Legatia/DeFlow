# Ramp Network Integration Guide for DeFlow

This comprehensive guide covers integrating Ramp Network's fiat-to-crypto on-ramp solution into the DeFlow platform, enabling users to pay for subscriptions with traditional payment methods while maintaining the platform's fully on-chain architecture.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Design](#architecture-design)
3. [Technical Requirements](#technical-requirements)
4. [Integration Methods](#integration-methods)
5. [Smart Contract Integration](#smart-contract-integration)
6. [User Experience Flow](#user-experience-flow)
7. [Security Considerations](#security-considerations)
8. [Implementation Plan](#implementation-plan)
9. [Testing Strategy](#testing-strategy)
10. [Troubleshooting](#troubleshooting)

---

## Overview

### What is Ramp Network?

Ramp Network is a fiat-to-crypto on-ramp and off-ramp solution that allows users to buy and sell cryptocurrency directly within dApps. It aggregates multiple liquidity sources, payment methods, and provides a seamless user experience for crypto transactions.

### Why Ramp Network for DeFlow?

- **True On-Chain Settlement**: Crypto is sent directly to user wallets, not merchant accounts
- **Maintains Decentralization**: Users retain custody of their cryptocurrency
- **Embedded Experience**: Users never leave the DeFlow platform
- **Multiple Payment Methods**: Cards, bank transfers, Apple Pay, Google Pay, PIX
- **Bi-directional**: Supports both on-ramp (fiat → crypto) and off-ramp (crypto → fiat)
- **Wide Asset Support**: BTC, ETH, ERC-20 tokens, and multiple blockchain networks

### DeFlow Integration Benefits

1. **Preserves On-Chain Architecture**: Payments settle on-chain after fiat conversion
2. **User-Friendly**: Traditional payment methods for crypto newcomers
3. **No Custody Issues**: Users control their crypto from purchase to payment
4. **Subscription Flexibility**: Pay with crypto earned from other activities
5. **Global Reach**: Supports multiple fiat currencies and payment methods

---

## Architecture Design

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   DeFlow User   │    │  Ramp Network   │    │ DeFlow Backend  │
│                 │    │                 │    │                 │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ 1. Select Plan  │───▶│ 2. Fiat Payment │    │ 6. Verify TX    │
│ 8. Use Premium  │    │ 3. Buy Crypto   │    │ 7. Activate Sub │
└─────────────────┘    │ 4. Send to      │    └─────────────────┘
         ▲              │    Wallet       │             ▲
         │              └─────────────────┘             │
         │                       │                      │
         │              ┌─────────────────┐             │
         └──────────────│ 5. On-Chain TX  │─────────────┘
                        │ (User → DeFlow) │
                        └─────────────────┘
```

### Payment Flow Architecture

```typescript
interface PaymentFlowArchitecture {
  // 1. User initiates payment
  userIntent: {
    planType: 'premium' | 'enterprise'
    billingCycle: 'monthly' | 'yearly'
    amount: number
    currency: 'USD' | 'EUR' | 'GBP'
  }
  
  // 2. Ramp integration
  rampConfig: {
    method: 'widget' | 'api'
    targetAsset: 'USDC' | 'ICP' | 'ETH'
    userWallet: string
    webhookUrl: string
  }
  
  // 3. On-chain verification
  verification: {
    transactionHash: string
    amount: bigint
    fromAddress: string
    toAddress: string
    timestamp: number
  }
  
  // 4. Subscription activation
  subscription: {
    userId: string
    planType: string
    activationDate: number
    expirationDate: number
    transactionProof: string
  }
}
```

---

## Technical Requirements

### Prerequisites

1. **Ramp Network Account**
   - Business registration with Ramp Network
   - API key and webhook URL setup
   - KYC/AML compliance review

2. **DeFlow Infrastructure**
   - Internet Computer Protocol (ICP) canisters
   - User wallet integration
   - Transaction monitoring system
   - Subscription management system

3. **Frontend Dependencies**
   ```json
   {
     "@ramp-network/ramp-instant-sdk": "^4.0.0",
     "web3": "^4.0.0",
     "ic-agent": "^0.20.0"
   }
   ```

4. **Backend Dependencies**
   ```motoko
   // Motoko dependencies for ICP canisters
   import Principal "mo:base/Principal";
   import Time "mo:base/Time";
   import HashMap "mo:base/HashMap";
   import Result "mo:base/Result";
   ```

### Environment Configuration

```typescript
// Environment variables required
interface RampEnvironmentConfig {
  RAMP_API_KEY: string              // Host API key from Ramp Network
  RAMP_WEBHOOK_SECRET: string       // Webhook signature verification
  RAMP_ENVIRONMENT: 'sandbox' | 'production'
  
  // DeFlow specific
  DEFLOW_CANISTER_ID: string        // Main DeFlow canister
  DEFLOW_SUBSCRIPTION_CANISTER: string // Subscription management
  ICP_NETWORK_URL: string           // ICP network endpoint
  
  // Supported assets
  SUPPORTED_CRYPTO_ASSETS: string[] // ['USDC_POLYGON', 'ICP', 'ETH']
  DEFAULT_CRYPTO_ASSET: string      // 'USDC_POLYGON'
}
```

---

## Integration Methods

### Method 1: Embedded Widget Integration (Recommended)

The embedded widget provides the best user experience by keeping users within the DeFlow platform.

#### Frontend Implementation

```typescript
// services/rampService.ts
import { RampInstantSDK } from '@ramp-network/ramp-instant-sdk'

export interface RampConfig {
  hostApiKey: string
  userAddress: string
  swapAmount: string
  swapAsset: string
  finalUrl: string
  webhookStatusUrl: string
}

export class RampService {
  private sdk: RampInstantSDK | null = null
  
  constructor(private config: RampConfig) {}
  
  initializeWidget() {
    this.sdk = new RampInstantSDK({
      hostApiKey: this.config.hostApiKey,
      hostAppName: 'DeFlow',
      hostLogoUrl: 'https://deflow.app/logo.png',
      
      // User and transaction details
      userAddress: this.config.userAddress,
      swapAmount: this.config.swapAmount,
      swapAsset: this.config.swapAsset,
      
      // Theming
      theme: 'dark',
      variant: 'embedded',
      
      // Callbacks
      finalUrl: this.config.finalUrl,
      webhookStatusUrl: this.config.webhookStatusUrl,
      
      // Event handlers
      onPurchaseCreated: this.handlePurchaseCreated.bind(this),
      onPurchaseFailed: this.handlePurchaseFailed.bind(this),
      onWidgetClose: this.handleWidgetClose.bind(this)
    })
  }
  
  show() {
    if (!this.sdk) {
      throw new Error('Ramp SDK not initialized')
    }
    this.sdk.show()
  }
  
  hide() {
    if (this.sdk) {
      this.sdk.unsubscribe()
    }
  }
  
  private handlePurchaseCreated(event: any) {
    console.log('Purchase created:', event)
    // Track purchase in DeFlow analytics
    this.trackPurchaseEvent('created', event)
  }
  
  private handlePurchaseFailed(event: any) {
    console.log('Purchase failed:', event)
    // Handle failure (show error message, redirect, etc.)
    this.trackPurchaseEvent('failed', event)
  }
  
  private handleWidgetClose() {
    console.log('Widget closed')
    // Clean up resources
  }
  
  private trackPurchaseEvent(type: string, event: any) {
    // Integrate with DeFlow analytics
    // Store purchase attempt in local storage for recovery
  }
}
```

#### React Component Integration

```typescript
// components/RampPaymentWidget.tsx
import React, { useEffect, useState } from 'react'
import { useAuth } from '../hooks/useAuth'
import { RampService } from '../services/rampService'

interface RampPaymentWidgetProps {
  planType: 'premium' | 'enterprise'
  amount: number
  currency: string
  onSuccess: (transactionData: any) => void
  onError: (error: any) => void
}

export const RampPaymentWidget: React.FC<RampPaymentWidgetProps> = ({
  planType,
  amount,
  currency,
  onSuccess,
  onError
}) => {
  const { user } = useAuth()
  const [rampService, setRampService] = useState<RampService | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  
  useEffect(() => {
    if (user?.walletAddress) {
      const config = {
        hostApiKey: process.env.REACT_APP_RAMP_API_KEY!,
        userAddress: user.walletAddress,
        swapAmount: amount.toString(),
        swapAsset: 'USDC_POLYGON', // or based on user preference
        finalUrl: `${window.location.origin}/payment/success`,
        webhookStatusUrl: `${process.env.REACT_APP_API_URL}/ramp/webhook`
      }
      
      const service = new RampService(config)
      service.initializeWidget()
      setRampService(service)
    }
    
    return () => {
      rampService?.hide()
    }
  }, [user?.walletAddress, amount])
  
  const handlePaymentClick = () => {
    if (!rampService) {
      onError(new Error('Payment service not initialized'))
      return
    }
    
    setIsLoading(true)
    rampService.show()
  }
  
  return (
    <div className="ramp-payment-widget">
      <div className="payment-summary">
        <h3>Complete Your Payment</h3>
        <div className="plan-details">
          <p><strong>Plan:</strong> {planType.charAt(0).toUpperCase() + planType.slice(1)}</p>
          <p><strong>Amount:</strong> {currency} {amount}</p>
          <p><strong>Billing:</strong> Monthly</p>
        </div>
      </div>
      
      <button
        onClick={handlePaymentClick}
        disabled={isLoading || !rampService}
        className="ramp-payment-button"
      >
        {isLoading ? (
          <span>🔄 Loading Payment...</span>
        ) : (
          <span>💳 Pay with Card/Bank → Crypto</span>
        )}
      </button>
      
      <div className="payment-info">
        <p>✅ Your payment will be converted to cryptocurrency</p>
        <p>🔒 Crypto will be sent directly to your wallet</p>
        <p>⚡ Subscription activates automatically after payment</p>
      </div>
    </div>
  )
}
```

### Method 2: API Integration with Custom UI

For more control over the user experience, you can integrate directly with Ramp's REST API.

```typescript
// services/rampAPIService.ts
export class RampAPIService {
  private baseUrl = 'https://api.ramp.network/api'
  private apiKey: string
  
  constructor(apiKey: string) {
    this.apiKey = apiKey
  }
  
  async getAvailableAssets(): Promise<RampAsset[]> {
    const response = await fetch(`${this.baseUrl}/host-api/assets`, {
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json'
      }
    })
    
    if (!response.ok) {
      throw new Error(`Failed to fetch assets: ${response.statusText}`)
    }
    
    return response.json()
  }
  
  async getQuote(params: QuoteParams): Promise<RampQuote> {
    const queryParams = new URLSearchParams({
      cryptoAssetSymbol: params.asset,
      fiatValue: params.amount.toString(),
      fiatCurrency: params.currency,
      paymentMethodType: params.paymentMethod,
      userIp: params.userIp || ''
    })
    
    const response = await fetch(
      `${this.baseUrl}/host-api/purchase?${queryParams}`,
      {
        headers: {
          'Authorization': `Bearer ${this.apiKey}`,
          'Content-Type': 'application/json'
        }
      }
    )
    
    if (!response.ok) {
      throw new Error(`Failed to get quote: ${response.statusText}`)
    }
    
    return response.json()
  }
  
  async createPurchase(params: PurchaseParams): Promise<RampPurchase> {
    const response = await fetch(`${this.baseUrl}/host-api/purchase`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        userAddress: params.userAddress,
        cryptoAssetSymbol: params.asset,
        fiatValue: params.amount,
        fiatCurrency: params.currency,
        paymentMethodType: params.paymentMethod,
        returnUrl: params.returnUrl,
        webhookUrl: params.webhookUrl
      })
    })
    
    if (!response.ok) {
      throw new Error(`Failed to create purchase: ${response.statusText}`)
    }
    
    return response.json()
  }
}

// Type definitions
interface RampAsset {
  symbol: string
  name: string
  type: 'NATIVE' | 'ERC20'
  decimals: number
  chain: string
  address?: string
  logoUrl: string
}

interface QuoteParams {
  asset: string
  amount: number
  currency: string
  paymentMethod: string
  userIp?: string
}

interface RampQuote {
  cryptoAmount: string
  fiatCurrency: string
  fiatValue: number
  assetInfo: RampAsset
  paymentMethodType: string
  baseRampFee: number
  networkFee: number
  appliedFee: number
}

interface PurchaseParams {
  userAddress: string
  asset: string
  amount: number
  currency: string
  paymentMethod: string
  returnUrl: string
  webhookUrl: string
}

interface RampPurchase {
  id: string
  endTime: string
  asset: RampAsset
  receiverAddress: string
  cryptoAmount: string
  fiatCurrency: string
  fiatValue: number
  paymentMethodType: string
  status: 'PENDING' | 'COMPLETED' | 'FAILED'
  actions: {
    type: 'PAY'
    url: string
  }[]
}
```

---

## Smart Contract Integration

### ICP Canister for Subscription Management

```motoko
// canisters/subscription.mo
import Principal "mo:base/Principal";
import Time "mo:base/Time";
import HashMap "mo:base/HashMap";
import Result "mo:base/Result";
import Debug "mo:base/Debug";
import Int "mo:base/Int";

actor SubscriptionManager {
  
  // Types
  public type SubscriptionPlan = {
    #Premium;
    #Enterprise;
  };
  
  public type SubscriptionStatus = {
    #Active;
    #Inactive;
    #Pending;
    #Expired;
  };
  
  public type Subscription = {
    user: Principal;
    plan: SubscriptionPlan;
    status: SubscriptionStatus;
    startDate: Int;
    endDate: Int;
    transactionHash: Text;
    amount: Nat;
    currency: Text;
  };
  
  public type PaymentVerification = {
    transactionHash: Text;
    amount: Nat;
    fromAddress: Text;
    toAddress: Text;
    timestamp: Int;
    rampPurchaseId: Text;
  };
  
  // State
  private stable var subscriptions : [(Principal, Subscription)] = [];
  private var subscriptionMap = HashMap.fromIter<Principal, Subscription>(
    subscriptions.vals(), 0, Principal.equal, Principal.hash
  );
  
  private stable var pendingPayments : [(Text, PaymentVerification)] = [];
  private var pendingPaymentMap = HashMap.fromIter<Text, PaymentVerification>(
    pendingPayments.vals(), 0, Text.equal, Text.hash
  );
  
  // Constants
  private let PREMIUM_PRICE_USD : Nat = 2999; // $29.99 in cents
  private let ENTERPRISE_PRICE_USD : Nat = 9999; // $99.99 in cents
  private let SUBSCRIPTION_DURATION : Int = 30 * 24 * 60 * 60 * 1000000000; // 30 days in nanoseconds
  
  // Admin principals (set during deployment)
  private stable var admins : [Principal] = [];
  
  // Webhook endpoint for Ramp Network
  public func processRampWebhook(
    purchaseId: Text,
    status: Text,
    transactionHash: Text,
    amount: Nat,
    userAddress: Text,
    metadata: ?Text
  ) : async Result.Result<Text, Text> {
    
    // Verify webhook signature (implement signature verification)
    // if (!verifyWebhookSignature(signature, payload)) {
    //   return #err("Invalid webhook signature");
    // };
    
    switch (status) {
      case ("COMPLETED") {
        await handleSuccessfulPayment(purchaseId, transactionHash, amount, userAddress, metadata);
      };
      case ("FAILED") {
        await handleFailedPayment(purchaseId, metadata);
      };
      case (_) {
        Debug.print("Unknown webhook status: " # status);
      };
    };
    
    #ok("Webhook processed successfully")
  };
  
  private func handleSuccessfulPayment(
    purchaseId: Text,
    transactionHash: Text,
    amount: Nat,
    userAddress: Text,
    metadata: ?Text
  ) : async () {
    
    // Extract user principal from metadata
    let userPrincipal = switch (metadata) {
      case (?meta) {
        // Parse metadata to extract user principal
        extractUserPrincipalFromMetadata(meta)
      };
      case null { return; }; // Cannot process without user identification
    };
    
    // Verify transaction on-chain (implement chain-specific verification)
    let verification : PaymentVerification = {
      transactionHash = transactionHash;
      amount = amount;
      fromAddress = userAddress;
      toAddress = ""; // DeFlow treasury address
      timestamp = Time.now();
      rampPurchaseId = purchaseId;
    };
    
    // Determine subscription plan based on amount
    let plan = if (amount >= ENTERPRISE_PRICE_USD) {
      #Enterprise
    } else if (amount >= PREMIUM_PRICE_USD) {
      #Premium
    } else {
      return; // Amount too low
    };
    
    // Create or update subscription
    let now = Time.now();
    let subscription : Subscription = {
      user = userPrincipal;
      plan = plan;
      status = #Active;
      startDate = now;
      endDate = now + SUBSCRIPTION_DURATION;
      transactionHash = transactionHash;
      amount = amount;
      currency = "USD";
    };
    
    subscriptionMap.put(userPrincipal, subscription);
    
    Debug.print("Subscription activated for user: " # Principal.toText(userPrincipal));
  };
  
  private func handleFailedPayment(purchaseId: Text, metadata: ?Text) : async () {
    // Log failed payment for investigation
    Debug.print("Payment failed for purchase: " # purchaseId);
    
    // Optionally notify user or trigger retry mechanism
  };
  
  // Public queries
  public query func getSubscription(user: Principal) : async ?Subscription {
    subscriptionMap.get(user)
  };
  
  public query func isSubscriptionActive(user: Principal) : async Bool {
    switch (subscriptionMap.get(user)) {
      case (?subscription) {
        subscription.status == #Active and Time.now() < subscription.endDate
      };
      case null { false };
    };
  };
  
  public func renewSubscription(user: Principal, transactionHash: Text) : async Result.Result<Text, Text> {
    // Implement subscription renewal logic
    #ok("Subscription renewed successfully")
  };
  
  // Administrative functions
  public func addAdmin(newAdmin: Principal) : async Result.Result<Text, Text> {
    // Only existing admins can add new admins
    if (not isAdmin(caller())) {
      return #err("Unauthorized: Only admins can add new admins");
    };
    
    admins := Array.append(admins, [newAdmin]);
    #ok("Admin added successfully")
  };
  
  private func isAdmin(principal: Principal) : Bool {
    Array.find<Principal>(admins, func(admin) = Principal.equal(admin, principal)) != null
  };
  
  private func extractUserPrincipalFromMetadata(metadata: Text) : Principal {
    // Implement metadata parsing to extract user principal
    // This would contain the user's principal ID passed from the frontend
    Principal.fromText(metadata) // Simplified - add proper error handling
  };
  
  private func caller() : Principal {
    // Get the calling principal
    // This is a placeholder - use the actual caller identification in IC
    Principal.fromText("2vxsx-fae") // Placeholder
  };
  
  // System functions for upgrades
  system func preupgrade() {
    subscriptions := Iter.toArray(subscriptionMap.entries());
    pendingPayments := Iter.toArray(pendingPaymentMap.entries());
  };
  
  system func postupgrade() {
    subscriptions := [];
    pendingPayments := [];
  };
}
```

### Webhook Verification Service

```typescript
// services/webhookVerification.ts
import crypto from 'crypto'

export class RampWebhookVerification {
  constructor(private webhookSecret: string) {}
  
  verifySignature(payload: string, signature: string): boolean {
    const expectedSignature = crypto
      .createHmac('sha256', this.webhookSecret)
      .update(payload)
      .digest('hex')
    
    return crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSignature, 'hex')
    )
  }
  
  async processWebhook(request: WebhookRequest): Promise<WebhookResponse> {
    const { payload, signature } = request
    
    if (!this.verifySignature(payload, signature)) {
      throw new Error('Invalid webhook signature')
    }
    
    const webhookData = JSON.parse(payload)
    
    // Process based on event type
    switch (webhookData.type) {
      case 'PURCHASE_COMPLETED':
        return await this.handlePurchaseCompleted(webhookData)
      case 'PURCHASE_FAILED':
        return await this.handlePurchaseFailed(webhookData)
      default:
        throw new Error(`Unknown webhook event type: ${webhookData.type}`)
    }
  }
  
  private async handlePurchaseCompleted(data: any): Promise<WebhookResponse> {
    // Call ICP canister to activate subscription
    const result = await this.callSubscriptionCanister({
      purchaseId: data.purchase.id,
      status: 'COMPLETED',
      transactionHash: data.purchase.cryptoTxHash,
      amount: data.purchase.fiatValue,
      userAddress: data.purchase.receiverAddress,
      metadata: data.purchase.hostApiKey // Contains user identification
    })
    
    return { success: true, message: 'Subscription activated' }
  }
  
  private async handlePurchaseFailed(data: any): Promise<WebhookResponse> {
    // Log failure and potentially notify user
    console.error('Purchase failed:', data.purchase.id)
    
    return { success: true, message: 'Failure logged' }
  }
  
  private async callSubscriptionCanister(params: any) {
    // Integrate with IC Agent to call canister
    // Implementation depends on your IC integration setup
  }
}

interface WebhookRequest {
  payload: string
  signature: string
}

interface WebhookResponse {
  success: boolean
  message: string
}
```

---

## User Experience Flow

### Complete Payment Journey

```typescript
// User journey state management
interface PaymentJourney {
  // Step 1: Plan Selection
  planSelection: {
    plan: 'premium' | 'enterprise'
    billing: 'monthly' | 'yearly'
    amount: number
    currency: string
  }
  
  // Step 2: Payment Method Choice
  paymentMethod: {
    type: 'crypto' | 'fiat-to-crypto'
    provider?: 'ramp' | 'direct'
  }
  
  // Step 3: Ramp Integration
  rampFlow: {
    status: 'pending' | 'processing' | 'completed' | 'failed'
    purchaseId?: string
    transactionHash?: string
    cryptoAmount?: string
    asset?: string
  }
  
  // Step 4: On-Chain Verification
  verification: {
    status: 'pending' | 'verified' | 'failed'
    blockConfirmations?: number
    estimatedTime?: number
  }
  
  // Step 5: Subscription Activation
  activation: {
    status: 'pending' | 'active' | 'failed'
    activationDate?: Date
    expirationDate?: Date
    features?: string[]
  }
}
```

### Frontend Payment Flow Component

```typescript
// components/PaymentFlow.tsx
import React, { useState, useEffect } from 'react'
import { PaymentJourney } from '../types/payment'
import { RampPaymentWidget } from './RampPaymentWidget'
import { PaymentStatus } from './PaymentStatus'

export const PaymentFlow: React.FC = () => {
  const [journey, setJourney] = useState<PaymentJourney>({
    planSelection: {
      plan: 'premium',
      billing: 'monthly',
      amount: 29.99,
      currency: 'USD'
    },
    paymentMethod: { type: 'fiat-to-crypto', provider: 'ramp' },
    rampFlow: { status: 'pending' },
    verification: { status: 'pending' },
    activation: { status: 'pending' }
  })
  
  const [currentStep, setCurrentStep] = useState<number>(1)
  
  const handleRampSuccess = (transactionData: any) => {
    setJourney(prev => ({
      ...prev,
      rampFlow: {
        status: 'completed',
        purchaseId: transactionData.purchaseId,
        transactionHash: transactionData.cryptoTxHash,
        cryptoAmount: transactionData.cryptoAmount,
        asset: transactionData.asset
      }
    }))
    setCurrentStep(4) // Move to verification step
    startVerificationPolling(transactionData.cryptoTxHash)
  }
  
  const handleRampError = (error: any) => {
    setJourney(prev => ({
      ...prev,
      rampFlow: { status: 'failed' }
    }))
    // Show error message to user
  }
  
  const startVerificationPolling = (txHash: string) => {
    // Poll for transaction confirmation and subscription activation
    const pollInterval = setInterval(async () => {
      try {
        const verification = await checkTransactionStatus(txHash)
        const subscription = await checkSubscriptionStatus()
        
        if (verification.confirmed && subscription.active) {
          setJourney(prev => ({
            ...prev,
            verification: { status: 'verified', blockConfirmations: verification.confirmations },
            activation: {
              status: 'active',
              activationDate: subscription.activationDate,
              expirationDate: subscription.expirationDate,
              features: subscription.features
            }
          }))
          setCurrentStep(5)
          clearInterval(pollInterval)
        }
      } catch (error) {
        console.error('Verification polling error:', error)
      }
    }, 5000) // Poll every 5 seconds
    
    // Clear polling after 10 minutes
    setTimeout(() => clearInterval(pollInterval), 600000)
  }
  
  return (
    <div className="payment-flow">
      <div className="progress-indicator">
        <ProgressBar currentStep={currentStep} totalSteps={5} />
      </div>
      
      {currentStep === 1 && (
        <PlanSelection
          selected={journey.planSelection}
          onSelect={(plan) => {
            setJourney(prev => ({ ...prev, planSelection: plan }))
            setCurrentStep(2)
          }}
        />
      )}
      
      {currentStep === 2 && (
        <PaymentMethodSelection
          onSelect={(method) => {
            setJourney(prev => ({ ...prev, paymentMethod: method }))
            setCurrentStep(3)
          }}
        />
      )}
      
      {currentStep === 3 && (
        <RampPaymentWidget
          planType={journey.planSelection.plan}
          amount={journey.planSelection.amount}
          currency={journey.planSelection.currency}
          onSuccess={handleRampSuccess}
          onError={handleRampError}
        />
      )}
      
      {currentStep >= 4 && (
        <PaymentStatus journey={journey} />
      )}
    </div>
  )
}
```

### Real-Time Status Updates

```typescript
// hooks/usePaymentStatus.ts
import { useEffect, useState } from 'react'
import { PaymentJourney } from '../types/payment'

export const usePaymentStatus = (transactionHash?: string) => {
  const [status, setStatus] = useState<PaymentStatus>('pending')
  const [confirmations, setConfirmations] = useState<number>(0)
  const [estimatedTime, setEstimatedTime] = useState<number>(0)
  
  useEffect(() => {
    if (!transactionHash) return
    
    const pollStatus = async () => {
      try {
        // Check blockchain for transaction status
        const txStatus = await checkTransactionOnChain(transactionHash)
        setConfirmations(txStatus.confirmations)
        
        // Check DeFlow backend for subscription status
        const subStatus = await checkSubscriptionStatus()
        
        if (txStatus.confirmed && subStatus.active) {
          setStatus('completed')
        } else if (txStatus.failed) {
          setStatus('failed')
        } else {
          setStatus('pending')
          // Estimate remaining time based on confirmations
          const remaining = Math.max(0, (6 - txStatus.confirmations) * 2) // ~2 min per confirmation
          setEstimatedTime(remaining)
        }
      } catch (error) {
        console.error('Status polling error:', error)
      }
    }
    
    // Poll every 30 seconds
    const interval = setInterval(pollStatus, 30000)
    pollStatus() // Initial check
    
    return () => clearInterval(interval)
  }, [transactionHash])
  
  return { status, confirmations, estimatedTime }
}

async function checkTransactionOnChain(txHash: string) {
  // Implement blockchain-specific transaction checking
  // This would vary based on the crypto asset used (ETH, BTC, etc.)
}

async function checkSubscriptionStatus() {
  // Call DeFlow backend to check subscription activation
  const response = await fetch('/api/subscription/status')
  return response.json()
}

type PaymentStatus = 'pending' | 'processing' | 'completed' | 'failed'
```

---

## Security Considerations

### Webhook Security

```typescript
// middleware/webhookSecurity.ts
import crypto from 'crypto'
import rateLimit from 'express-rate-limit'

// Rate limiting for webhook endpoints
export const webhookRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
  message: 'Too many webhook requests from this IP',
  standardHeaders: true,
  legacyHeaders: false
})

// Webhook signature verification middleware
export const verifyWebhookSignature = (secret: string) => {
  return (req: any, res: any, next: any) => {
    const signature = req.headers['x-ramp-signature']
    const payload = JSON.stringify(req.body)
    
    if (!signature) {
      return res.status(401).json({ error: 'Missing signature' })
    }
    
    const expectedSignature = crypto
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex')
    
    if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expectedSignature))) {
      return res.status(401).json({ error: 'Invalid signature' })
    }
    
    next()
  }
}

// IP whitelist for Ramp Network
const RAMP_IP_WHITELIST = [
  '52.58.171.90',
  '52.29.115.207',
  // Add Ramp Network's webhook IPs
]

export const validateRampIP = (req: any, res: any, next: any) => {
  const clientIP = req.ip || req.connection.remoteAddress
  
  if (!RAMP_IP_WHITELIST.includes(clientIP)) {
    return res.status(403).json({ error: 'Unauthorized IP address' })
  }
  
  next()
}
```

### Data Protection

```typescript
// utils/dataProtection.ts
import CryptoJS from 'crypto-js'

export class DataProtection {
  private static encryptionKey = process.env.ENCRYPTION_KEY!
  
  // Encrypt sensitive user data
  static encryptUserData(data: any): string {
    return CryptoJS.AES.encrypt(JSON.stringify(data), this.encryptionKey).toString()
  }
  
  // Decrypt sensitive user data
  static decryptUserData(encryptedData: string): any {
    const bytes = CryptoJS.AES.decrypt(encryptedData, this.encryptionKey)
    return JSON.parse(bytes.toString(CryptoJS.enc.Utf8))
  }
  
  // Hash user identifiers for analytics (GDPR compliant)
  static hashUserIdentifier(identifier: string): string {
    return CryptoJS.SHA256(identifier + process.env.HASH_SALT).toString()
  }
  
  // Validate and sanitize webhook data
  static sanitizeWebhookData(data: any): any {
    // Remove any potentially malicious data
    // Validate all fields against expected schema
    // Return cleaned data
    return {
      purchaseId: this.sanitizeString(data.purchaseId),
      status: this.sanitizeEnum(data.status, ['COMPLETED', 'FAILED', 'PENDING']),
      amount: this.sanitizeNumber(data.amount),
      currency: this.sanitizeString(data.currency),
      transactionHash: this.sanitizeString(data.transactionHash),
      userAddress: this.sanitizeAddress(data.userAddress)
    }
  }
  
  private static sanitizeString(value: any): string {
    return typeof value === 'string' ? value.trim().slice(0, 255) : ''
  }
  
  private static sanitizeNumber(value: any): number {
    const num = parseFloat(value)
    return isNaN(num) ? 0 : num
  }
  
  private static sanitizeEnum(value: any, allowedValues: string[]): string {
    return allowedValues.includes(value) ? value : allowedValues[0]
  }
  
  private static sanitizeAddress(value: any): string {
    // Validate crypto address format
    if (typeof value !== 'string') return ''
    
    // Basic validation - implement proper address validation
    return /^[a-zA-Z0-9]{20,100}$/.test(value) ? value : ''
  }
}
```

### Transaction Verification

```motoko
// canisters/transactionVerifier.mo
import Principal "mo:base/Principal";
import Time "mo:base/Time";
import Result "mo:base/Result";
import HashMap "mo:base/HashMap";

actor TransactionVerifier {
  
  public type VerificationResult = {
    #Valid: { amount: Nat; timestamp: Int; confirmations: Nat };
    #Invalid: Text;
    #Pending: { confirmations: Nat; required: Nat };
  };
  
  public type TransactionRecord = {
    hash: Text;
    amount: Nat;
    fromAddress: Text;
    toAddress: Text;
    timestamp: Int;
    verified: Bool;
    blockHeight: Nat;
  };
  
  private stable var verifiedTransactions : [(Text, TransactionRecord)] = [];
  private var transactionMap = HashMap.fromIter<Text, TransactionRecord>(
    verifiedTransactions.vals(), 0, Text.equal, Text.hash
  );
  
  // Verify transaction on supported blockchain
  public func verifyTransaction(
    txHash: Text,
    expectedAmount: Nat,
    expectedToAddress: Text,
    blockchain: Text
  ) : async VerificationResult {
    
    // Check if already verified
    switch (transactionMap.get(txHash)) {
      case (?record) {
        if (record.verified) {
          return #Valid({
            amount = record.amount;
            timestamp = record.timestamp;
            confirmations = 6; // Assume confirmed if stored
          });
        };
      };
      case null {};
    };
    
    // Verify based on blockchain
    switch (blockchain) {
      case ("ETHEREUM") {
        await verifyEthereumTransaction(txHash, expectedAmount, expectedToAddress)
      };
      case ("POLYGON") {
        await verifyPolygonTransaction(txHash, expectedAmount, expectedToAddress)
      };
      case ("ICP") {
        await verifyICPTransaction(txHash, expectedAmount, expectedToAddress)
      };
      case (_) {
        #Invalid("Unsupported blockchain: " # blockchain)
      };
    };
  };
  
  private func verifyEthereumTransaction(
    txHash: Text,
    expectedAmount: Nat,
    expectedToAddress: Text
  ) : async VerificationResult {
    // Implement Ethereum transaction verification
    // This would involve calling an external API or oracle
    
    // For now, return a placeholder
    #Valid({
      amount = expectedAmount;
      timestamp = Time.now();
      confirmations = 6;
    })
  };
  
  private func verifyPolygonTransaction(
    txHash: Text,
    expectedAmount: Nat,
    expectedToAddress: Text
  ) : async VerificationResult {
    // Implement Polygon transaction verification
    #Valid({
      amount = expectedAmount;
      timestamp = Time.now();
      confirmations = 6;
    })
  };
  
  private func verifyICPTransaction(
    txHash: Text,
    expectedAmount: Nat,
    expectedToAddress: Text
  ) : async VerificationResult {
    // Implement ICP transaction verification using IC ledger
    #Valid({
      amount = expectedAmount;
      timestamp = Time.now();
      confirmations = 1; // ICP has fast finality
    })
  };
  
  // Store verified transaction
  public func storeVerifiedTransaction(record: TransactionRecord) : async Bool {
    transactionMap.put(record.hash, record);
    true
  };
  
  // System functions
  system func preupgrade() {
    verifiedTransactions := Iter.toArray(transactionMap.entries());
  };
  
  system func postupgrade() {
    verifiedTransactions := [];
  };
}
```

---

## Implementation Plan

### Phase 1: Foundation Setup (Week 1-2)

#### Week 1: Environment and Basic Integration
- [ ] **Day 1-2**: Ramp Network account setup and API key generation
- [ ] **Day 3-4**: Frontend SDK integration and basic widget implementation
- [ ] **Day 5-7**: Backend webhook endpoint creation and signature verification

#### Week 2: Core Payment Flow
- [ ] **Day 1-3**: React components for payment flow and status tracking
- [ ] **Day 4-5**: ICP canister development for subscription management
- [ ] **Day 6-7**: Integration testing with Ramp sandbox environment

### Phase 2: Smart Contract Integration (Week 3-4)

#### Week 3: Blockchain Verification
- [ ] **Day 1-3**: Transaction verification system for multiple blockchains
- [ ] **Day 4-5**: On-chain subscription activation logic
- [ ] **Day 6-7**: Webhook processing and canister communication

#### Week 4: Testing and Security
- [ ] **Day 1-3**: Security implementation (signature verification, rate limiting)
- [ ] **Day 4-5**: Comprehensive testing with various payment scenarios
- [ ] **Day 6-7**: Error handling and edge case management

### Phase 3: User Experience Enhancement (Week 5-6)

#### Week 5: UI/UX Improvements
- [ ] **Day 1-3**: Payment flow optimization and user feedback systems
- [ ] **Day 4-5**: Real-time status updates and notification system
- [ ] **Day 6-7**: Mobile responsiveness and accessibility improvements

#### Week 6: Advanced Features
- [ ] **Day 1-3**: Multi-currency support and asset selection
- [ ] **Day 4-5**: Subscription renewal and upgrade flows
- [ ] **Day 6-7**: Analytics and monitoring integration

### Phase 4: Production Deployment (Week 7-8)

#### Week 7: Production Preparation
- [ ] **Day 1-3**: Production environment setup and configuration
- [ ] **Day 4-5**: Load testing and performance optimization
- [ ] **Day 6-7**: Security audit and penetration testing

#### Week 8: Launch and Monitoring
- [ ] **Day 1-3**: Production deployment and monitoring setup
- [ ] **Day 4-5**: User acceptance testing and feedback collection
- [ ] **Day 6-7**: Bug fixes and performance tuning

### Development Milestones

| Milestone | Description | Timeline | Success Criteria |
|-----------|-------------|----------|------------------|
| **M1** | Basic Integration | Week 2 | Widget shows, webhook receives events |
| **M2** | Payment Processing | Week 4 | End-to-end payment completes successfully |
| **M3** | Smart Contract Integration | Week 6 | Subscription auto-activates after payment |
| **M4** | Production Ready | Week 8 | System handles production load securely |

---

## Testing Strategy

### Unit Testing

```typescript
// tests/rampService.test.ts
import { RampService } from '../services/rampService'
import { jest } from '@jest/globals'

describe('RampService', () => {
  let rampService: RampService
  
  beforeEach(() => {
    const config = {
      hostApiKey: 'test-api-key',
      userAddress: '0x123...abc',
      swapAmount: '29.99',
      swapAsset: 'USDC_POLYGON',
      finalUrl: 'https://test.com/success',
      webhookStatusUrl: 'https://test.com/webhook'
    }
    rampService = new RampService(config)
  })
  
  test('should initialize widget with correct config', () => {
    const initializeSpy = jest.spyOn(rampService, 'initializeWidget')
    rampService.initializeWidget()
    expect(initializeSpy).toHaveBeenCalled()
  })
  
  test('should handle purchase created event', () => {
    const trackSpy = jest.spyOn(rampService as any, 'trackPurchaseEvent')
    const event = { purchaseId: 'test-123', status: 'CREATED' }
    
    ;(rampService as any).handlePurchaseCreated(event)
    expect(trackSpy).toHaveBeenCalledWith('created', event)
  })
  
  test('should handle purchase failed event', () => {
    const trackSpy = jest.spyOn(rampService as any, 'trackPurchaseEvent')
    const event = { purchaseId: 'test-123', error: 'Payment declined' }
    
    ;(rampService as any).handlePurchaseFailed(event)
    expect(trackSpy).toHaveBeenCalledWith('failed', event)
  })
})
```

### Integration Testing

```typescript
// tests/paymentFlow.integration.test.ts
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { PaymentFlow } from '../components/PaymentFlow'
import { RampService } from '../services/rampService'

// Mock Ramp SDK
jest.mock('@ramp-network/ramp-instant-sdk', () => ({
  RampInstantSDK: jest.fn().mockImplementation(() => ({
    show: jest.fn(),
    hide: jest.fn(),
    unsubscribe: jest.fn()
  }))
}))

describe('Payment Flow Integration', () => {
  test('should complete full payment journey', async () => {
    render(<PaymentFlow />)
    
    // Step 1: Plan selection
    expect(screen.getByText('Select Plan')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Premium'))
    
    // Step 2: Payment method selection
    await waitFor(() => {
      expect(screen.getByText('Payment Method')).toBeInTheDocument()
    })
    fireEvent.click(screen.getByText('Pay with Card/Bank → Crypto'))
    
    // Step 3: Ramp widget
    await waitFor(() => {
      expect(screen.getByText('Complete Your Payment')).toBeInTheDocument()
    })
    
    // Simulate successful payment
    const rampButton = screen.getByRole('button', { name: /pay with card/i })
    fireEvent.click(rampButton)
    
    // Verify widget initialization
    expect(RampInstantSDK).toHaveBeenCalledWith(
      expect.objectContaining({
        hostAppName: 'DeFlow',
        swapAmount: '29.99'
      })
    )
  })
  
  test('should handle payment failure gracefully', async () => {
    render(<PaymentFlow />)
    
    // Navigate to payment step
    fireEvent.click(screen.getByText('Premium'))
    await waitFor(() => fireEvent.click(screen.getByText('Pay with Card/Bank → Crypto')))
    
    // Simulate payment failure
    const rampService = new RampService({} as any)
    ;(rampService as any).handlePurchaseFailed({ error: 'Card declined' })
    
    await waitFor(() => {
      expect(screen.getByText(/payment failed/i)).toBeInTheDocument()
    })
  })
})
```

### End-to-End Testing

```typescript
// tests/e2e/paymentFlow.e2e.test.ts
import { test, expect } from '@playwright/test'

test.describe('Ramp Payment Integration E2E', () => {
  test('complete payment flow with Ramp sandbox', async ({ page }) => {
    // Navigate to DeFlow payment page
    await page.goto('/payment')
    
    // Select premium plan
    await page.click('[data-testid="premium-plan"]')
    await page.click('[data-testid="select-plan"]')
    
    // Choose Ramp payment method
    await page.click('[data-testid="ramp-payment"]')
    
    // Wait for Ramp widget to load
    await page.waitForSelector('[data-testid="ramp-widget"]')
    
    // Interact with Ramp widget (if testable in sandbox)
    // Note: This depends on Ramp's testing capabilities
    
    // Verify payment success state
    await page.waitForSelector('[data-testid="payment-success"]', { timeout: 60000 })
    
    // Check that subscription is activated
    await page.goto('/dashboard')
    await expect(page.locator('[data-testid="premium-badge"]')).toBeVisible()
  })
  
  test('handle payment cancellation', async ({ page }) => {
    await page.goto('/payment')
    
    // Start payment flow
    await page.click('[data-testid="premium-plan"]')
    await page.click('[data-testid="select-plan"]')
    await page.click('[data-testid="ramp-payment"]')
    
    // Cancel payment (simulate user closing widget)
    await page.click('[data-testid="cancel-payment"]')
    
    // Verify user is returned to plan selection
    await expect(page.locator('[data-testid="plan-selection"]')).toBeVisible()
  })
})
```

### Load Testing

```typescript
// tests/load/webhookLoad.test.ts
import { test } from '@playwright/test'

test.describe('Webhook Load Testing', () => {
  test('handle concurrent webhook requests', async ({ request }) => {
    const webhookUrl = 'https://api.deflow.app/ramp/webhook'
    const payload = {
      type: 'PURCHASE_COMPLETED',
      purchase: {
        id: 'test-purchase-123',
        status: 'COMPLETED',
        cryptoTxHash: '0xabc123...',
        fiatValue: 2999,
        receiverAddress: '0x456def...'
      }
    }
    
    // Send 100 concurrent webhook requests
    const requests = Array.from({ length: 100 }, (_, i) => 
      request.post(webhookUrl, {
        data: { ...payload, purchase: { ...payload.purchase, id: `test-${i}` } }
      })
    )
    
    const responses = await Promise.all(requests)
    
    // Verify all requests were handled successfully
    responses.forEach(response => {
      expect(response.status()).toBe(200)
    })
  })
})
```

---

## Troubleshooting

### Common Issues and Solutions

#### 1. Widget Not Loading

**Problem**: Ramp widget doesn't appear or fails to initialize

**Possible Causes**:
- Invalid API key
- Incorrect user address format
- Network connectivity issues
- CSP (Content Security Policy) blocking

**Solutions**:
```typescript
// Debug widget initialization
const debugRampWidget = () => {
  console.log('API Key valid:', !!process.env.REACT_APP_RAMP_API_KEY)
  console.log('User address format:', /^0x[a-fA-F0-9]{40}$/.test(userAddress))
  console.log('Network connectivity:', navigator.onLine)
  
  // Check CSP headers
  console.log('CSP headers:', document.querySelector('meta[http-equiv="Content-Security-Policy"]'))
}

// Add CSP whitelist for Ramp
// In your HTML head:
// <meta http-equiv="Content-Security-Policy" 
//       content="script-src 'self' 'unsafe-inline' *.ramp.network; 
//                frame-src 'self' *.ramp.network;
//                connect-src 'self' *.ramp.network;">
```

#### 2. Webhook Not Receiving Events

**Problem**: DeFlow backend doesn't receive webhook notifications from Ramp

**Diagnostic Steps**:
```typescript
// Webhook debugging endpoint
app.post('/debug/webhook', (req, res) => {
  console.log('Headers:', req.headers)
  console.log('Body:', req.body)
  console.log('IP:', req.ip)
  console.log('User-Agent:', req.get('User-Agent'))
  
  res.status(200).json({ received: true, timestamp: Date.now() })
})

// Test webhook connectivity
const testWebhookConnectivity = async () => {
  try {
    const response = await fetch('https://api.deflow.app/debug/webhook', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ test: true })
    })
    console.log('Webhook reachable:', response.ok)
  } catch (error) {
    console.error('Webhook unreachable:', error)
  }
}
```

**Common Solutions**:
- Verify webhook URL is publicly accessible
- Check firewall settings
- Ensure HTTPS is configured properly
- Validate webhook signature verification

#### 3. Transaction Verification Failures

**Problem**: Payments complete in Ramp but subscription doesn't activate

**Debugging Process**:
```typescript
// Transaction verification debugging
const debugTransactionVerification = async (txHash: string) => {
  console.log('Verifying transaction:', txHash)
  
  // Check multiple block explorers
  const explorers = [
    `https://etherscan.io/api?module=proxy&action=eth_getTransactionByHash&txhash=${txHash}`,
    `https://api.polygonscan.com/api?module=proxy&action=eth_getTransactionByHash&txhash=${txHash}`
  ]
  
  for (const explorer of explorers) {
    try {
      const response = await fetch(explorer)
      const data = await response.json()
      console.log('Explorer data:', data)
    } catch (error) {
      console.error('Explorer error:', error)
    }
  }
  
  // Check canister state
  const subscription = await checkCanisterSubscription(userId)
  console.log('Canister subscription state:', subscription)
}
```

#### 4. User Experience Issues

**Problem**: Users confused about payment process or status

**UX Improvements**:
```typescript
// Enhanced status communication
const PaymentStatusTracker = ({ transactionHash }: { transactionHash: string }) => {
  const [status, setStatus] = useState({
    step: 1,
    message: 'Payment processing...',
    eta: '2-5 minutes'
  })
  
  const statusUpdates = [
    { step: 1, message: 'Payment received by Ramp Network', eta: '1-2 minutes' },
    { step: 2, message: 'Converting fiat to cryptocurrency', eta: '1-2 minutes' },
    { step: 3, message: 'Sending crypto to your wallet', eta: '1-2 minutes' },
    { step: 4, message: 'Verifying on-chain transaction', eta: '2-10 minutes' },
    { step: 5, message: 'Activating your subscription', eta: '30 seconds' },
    { step: 6, message: 'Complete! Premium features unlocked', eta: 'Now' }
  ]
  
  return (
    <div className="payment-status-tracker">
      {statusUpdates.map((update, index) => (
        <div 
          key={index}
          className={`status-step ${index <= status.step ? 'completed' : 'pending'}`}
        >
          <div className="step-indicator">
            {index < status.step ? '✅' : index === status.step ? '🔄' : '⏳'}
          </div>
          <div className="step-content">
            <p className="step-message">{update.message}</p>
            {index === status.step && (
              <p className="step-eta">Estimated time: {update.eta}</p>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
```

### Performance Monitoring

```typescript
// Performance monitoring for Ramp integration
class RampPerformanceMonitor {
  private metrics: Map<string, number> = new Map()
  
  startTimer(operation: string) {
    this.metrics.set(`${operation}_start`, Date.now())
  }
  
  endTimer(operation: string) {
    const start = this.metrics.get(`${operation}_start`)
    if (start) {
      const duration = Date.now() - start
      this.metrics.set(`${operation}_duration`, duration)
      console.log(`${operation} took ${duration}ms`)
      
      // Send to analytics
      this.sendMetric(operation, duration)
    }
  }
  
  private sendMetric(operation: string, duration: number) {
    // Send to your analytics service
    fetch('/api/analytics/timing', {
      method: 'POST',
      body: JSON.stringify({
        metric: `ramp.${operation}`,
        duration,
        timestamp: Date.now()
      })
    })
  }
}

// Usage
const monitor = new RampPerformanceMonitor()

// Track widget load time
monitor.startTimer('widget_load')
rampService.show()
// ... when widget loads
monitor.endTimer('widget_load')

// Track payment completion time
monitor.startTimer('payment_completion')
// ... when payment completes
monitor.endTimer('payment_completion')
```

### Error Recovery Strategies

```typescript
// Automatic retry mechanism for failed payments
class PaymentRecoveryService {
  private retryAttempts = new Map<string, number>()
  private maxRetries = 3
  
  async processPaymentWithRetry(paymentData: any): Promise<boolean> {
    const attemptKey = paymentData.userId + '_' + paymentData.timestamp
    const currentAttempts = this.retryAttempts.get(attemptKey) || 0
    
    if (currentAttempts >= this.maxRetries) {
      console.error('Max retry attempts reached for payment:', attemptKey)
      await this.escalateToManualReview(paymentData)
      return false
    }
    
    try {
      await this.processPayment(paymentData)
      this.retryAttempts.delete(attemptKey) // Success - clear retry count
      return true
    } catch (error) {
      console.error(`Payment attempt ${currentAttempts + 1} failed:`, error)
      this.retryAttempts.set(attemptKey, currentAttempts + 1)
      
      // Exponential backoff
      const backoffDelay = Math.pow(2, currentAttempts) * 1000 // 1s, 2s, 4s
      setTimeout(() => {
        this.processPaymentWithRetry(paymentData)
      }, backoffDelay)
      
      return false
    }
  }
  
  private async escalateToManualReview(paymentData: any) {
    // Create support ticket
    await fetch('/api/support/tickets', {
      method: 'POST',
      body: JSON.stringify({
        type: 'payment_failure',
        priority: 'high',
        data: paymentData,
        timestamp: Date.now()
      })
    })
    
    // Notify user
    await this.notifyUser(paymentData.userId, {
      title: 'Payment Processing Issue',
      message: 'We\'re reviewing your payment. You\'ll receive an update within 24 hours.',
      type: 'warning'
    })
  }
}
```

---

This comprehensive documentation provides everything needed to successfully integrate Ramp Network into DeFlow while maintaining the platform's fully on-chain architecture. The integration preserves decentralization by ensuring crypto assets flow directly to user wallets, while providing the familiar payment experience users expect.