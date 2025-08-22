// Payment Service for USDC/USDT Stablecoin Payments
// Integrates with DeFlow pool canister payment APIs

interface PaymentMethod {
  id: string;
  chain: string;
  asset: string;
  token_address?: string;
  enabled: boolean;
  min_amount_usd: number;
  max_amount_usd: number;
  processing_fee_bps: number;
  confirmation_blocks: number;
  estimated_settlement_time: number;
}

interface PaymentPurpose {
  Subscription?: { plan: string; duration_months: number };
  WorkflowExecution?: { workflow_id: string; estimated_cost: number };
  PremiumFeatures?: { features: string[] };
  TopUp?: { credits: number };
}

interface PaymentRequest {
  id: string;
  user_principal: string;
  payment_method: PaymentMethod;
  amount: number;
  amount_usd: number;
  fee_amount: number;
  fee_amount_usd: number;
  destination_address: string;
  sender_address: string;
  tx_hash?: string;
  status: PaymentStatus;
  initiated_at: bigint;
  confirmed_at?: bigint;
  expires_at: bigint;
  purpose: PaymentPurpose;
  metadata: PaymentMetadata;
}

interface PaymentMetadata {
  invoice_id?: string;
  notes?: string;
  tags: string[];
  refund_policy: RefundPolicy;
}

type PaymentStatus = 'Created' | 'WaitingConfirmation' | 'Confirmed' | 'Failed' | 'Expired' | 'Refunded';

type RefundPolicy = 
  | { NoRefund: null }
  | { FullRefund: { within_hours: number } }
  | { PartialRefund: { percentage: number; within_hours: number } }
  | { CustomTerms: { terms: string } };

export class PaymentService {
  private static poolCanisterId = process.env.VITE_CANISTER_ID_DEFLOW_POOL;

  /**
   * Get all supported payment methods (USDC/USDT across chains)
   */
  static async getSupportedPaymentMethods(): Promise<PaymentMethod[]> {
    try {
      // Mock implementation for development
      
      return [
        {
          id: 'polygon_usdc',
          chain: 'Polygon',
          asset: 'USDC',
          token_address: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174',
          enabled: true,
          min_amount_usd: 1.0,
          max_amount_usd: 10000.0,
          processing_fee_bps: 75, // 0.75%
          confirmation_blocks: 20,
          estimated_settlement_time: 300, // 5 minutes
        },
        {
          id: 'polygon_usdt',
          chain: 'Polygon',
          asset: 'USDT',
          token_address: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F',
          enabled: true,
          min_amount_usd: 1.0,
          max_amount_usd: 10000.0,
          processing_fee_bps: 75, // 0.75%
          confirmation_blocks: 20,
          estimated_settlement_time: 300, // 5 minutes
        },
        {
          id: 'arbitrum_usdc',
          chain: 'Arbitrum',
          asset: 'USDC',
          token_address: '0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8',
          enabled: true,
          min_amount_usd: 1.0,
          max_amount_usd: 10000.0,
          processing_fee_bps: 50, // 0.5%
          confirmation_blocks: 1,
          estimated_settlement_time: 60, // 1 minute
        },
        {
          id: 'base_usdc',
          chain: 'Base',
          asset: 'USDC',
          token_address: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913',
          enabled: true,
          min_amount_usd: 1.0,
          max_amount_usd: 10000.0,
          processing_fee_bps: 50, // 0.5%
          confirmation_blocks: 1,
          estimated_settlement_time: 60, // 1 minute
        },
        {
          id: 'ethereum_usdc',
          chain: 'Ethereum',
          asset: 'USDC',
          token_address: '0xA0b86a33E6441b5cBb5b9c7e9a8e49A44A2a1c6f',
          enabled: true,
          min_amount_usd: 5.0, // Higher minimum due to gas costs
          max_amount_usd: 10000.0,
          processing_fee_bps: 100, // 1%
          confirmation_blocks: 12,
          estimated_settlement_time: 900, // 15 minutes
        },
        {
          id: 'ethereum_usdt',
          chain: 'Ethereum',
          asset: 'USDT',
          token_address: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
          enabled: true,
          min_amount_usd: 5.0, // Higher minimum due to gas costs
          max_amount_usd: 10000.0,
          processing_fee_bps: 100, // 1%
          confirmation_blocks: 12,
          estimated_settlement_time: 900, // 15 minutes
        },
      ];
    } catch (error) {
      console.error('Failed to get payment methods:', error);
      throw new Error('Failed to load payment methods');
    }
  }

  /**
   * Create a payment request for subscription or other purposes
   */
  static async createPaymentRequest(
    paymentMethodId: string,
    amountUsd: number,
    purpose: PaymentPurpose,
    senderAddress: string
  ): Promise<PaymentRequest> {
    try {
      // TODO: Call pool canister create_payment_request function
      // await poolCanister.create_payment_request({
      //   paymentMethodId,
      //   amountUsd,
      //   purpose,
      //   senderAddress
      // });

      // Mock implementation for development
      const paymentMethods = await this.getSupportedPaymentMethods();
      const paymentMethod = paymentMethods.find(pm => pm.id === paymentMethodId);
      
      if (!paymentMethod) {
        throw new Error('Payment method not found');
      }

      // Calculate fees
      const feeAmountUsd = amountUsd * (paymentMethod.processing_fee_bps / 10000);
      const totalAmountUsd = amountUsd + feeAmountUsd;

      // Mock treasury address (in production, this comes from pool canister)
      const treasuryAddresses = {
        'polygon_usdc': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
        'polygon_usdt': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
        'arbitrum_usdc': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
        'base_usdc': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
        'ethereum_usdc': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
        'ethereum_usdt': '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6',
      };

      const destinationAddress = treasuryAddresses[paymentMethodId as keyof typeof treasuryAddresses] || 
                                 '0x742e3B7e6a7a5e3f7bF4d3E6BaA8A5e3F7B4F3d6';

      const currentTime = BigInt(Date.now() * 1000000); // Convert to nanoseconds
      const paymentId = `pay_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      const paymentRequest: PaymentRequest = {
        id: paymentId,
        user_principal: 'mock-user-principal',
        payment_method: paymentMethod,
        amount: totalAmountUsd, // For stablecoins, amount ≈ USD value
        amount_usd: totalAmountUsd,
        fee_amount: feeAmountUsd,
        fee_amount_usd: feeAmountUsd,
        destination_address: destinationAddress,
        sender_address: senderAddress,
        status: 'Created',
        initiated_at: currentTime,
        expires_at: currentTime + BigInt(24 * 60 * 60 * 1000000000), // 24 hours
        purpose,
        metadata: {
          tags: ['stablecoin-payment', paymentMethod.chain.toLowerCase()],
          refund_policy: { FullRefund: { within_hours: 24 } }
        }
      };

      return paymentRequest;
    } catch (error) {
      console.error('Failed to create payment request:', error);
      throw new Error(`Failed to create payment request: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Confirm payment with transaction hash
   */
  static async confirmPayment(paymentId: string, txHash: string): Promise<void> {
    try {
      // TODO: Call pool canister confirm_payment function
      // await poolCanister.confirm_payment(paymentId, txHash);

      // Mock implementation for development
      // In production, this would call the pool canister confirm_payment function
      
      // Simulate processing time
      await new Promise(resolve => setTimeout(resolve, 1000));

    } catch (error) {
      console.error('Failed to confirm payment:', error);
      throw new Error(`Failed to confirm payment: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get payment status
   */
  static async getPaymentStatus(paymentId: string): Promise<PaymentStatus> {
    try {
      // TODO: Call pool canister get_payment_status function
      // return await poolCanister.get_payment_status(paymentId);

      // Mock implementation for development
      // In production, this would call the pool canister get_payment_status function
      return 'WaitingConfirmation';
    } catch (error) {
      console.error('Failed to get payment status:', error);
      throw new Error('Failed to get payment status');
    }
  }

  /**
   * Get user's payment history
   */
  static async getUserPayments(userPrincipal: string): Promise<PaymentRequest[]> {
    try {
      // TODO: Call pool canister get_user_payments function
      // return await poolCanister.get_user_payments(userPrincipal);

      // Mock implementation for development
      // In production, this would call the pool canister get_user_payments function
      return [];
    } catch (error) {
      console.error('Failed to get user payments:', error);
      return [];
    }
  }

  /**
   * Calculate payment details including fees
   */
  static calculatePaymentDetails(paymentMethod: PaymentMethod, amountUsd: number) {
    const feeAmountUsd = amountUsd * (paymentMethod.processing_fee_bps / 10000);
    const totalAmountUsd = amountUsd + feeAmountUsd;

    return {
      baseAmount: amountUsd,
      feeAmount: feeAmountUsd,
      totalAmount: totalAmountUsd,
      feePercentage: paymentMethod.processing_fee_bps / 100,
      estimatedTime: paymentMethod.estimated_settlement_time,
      confirmationBlocks: paymentMethod.confirmation_blocks
    };
  }

  /**
   * Get recommended payment method based on amount and urgency
   */
  static async getRecommendedPaymentMethod(
    amountUsd: number, 
    urgency: 'low' | 'medium' | 'high' = 'medium'
  ): Promise<PaymentMethod | null> {
    try {
      const methods = await this.getSupportedPaymentMethods();
      const eligibleMethods = methods.filter(
        method => method.enabled && 
                 amountUsd >= method.min_amount_usd && 
                 amountUsd <= method.max_amount_usd
      );

      if (eligibleMethods.length === 0) {
        return null;
      }

      // Sort by different criteria based on urgency
      switch (urgency) {
        case 'high':
          // Prioritize speed (lowest settlement time)
          return eligibleMethods.sort((a, b) => a.estimated_settlement_time - b.estimated_settlement_time)[0];
        
        case 'low':
          // Prioritize cost (lowest fees)
          return eligibleMethods.sort((a, b) => a.processing_fee_bps - b.processing_fee_bps)[0];
        
        default:
          // Balanced approach: consider both speed and cost
          return eligibleMethods.sort((a, b) => {
            const scoreA = a.processing_fee_bps + (a.estimated_settlement_time / 60); // Normalize time to minutes
            const scoreB = b.processing_fee_bps + (b.estimated_settlement_time / 60);
            return scoreA - scoreB;
          })[0];
      }
    } catch (error) {
      console.error('Failed to get recommended payment method:', error);
      return null;
    }
  }

  /**
   * Get chain-specific payment instructions
   */
  static getPaymentInstructions(paymentMethod: PaymentMethod): string[] {
    const instructions = [
      `Send ${paymentMethod.asset} to the provided address`,
      `Network: ${paymentMethod.chain}`,
      `Minimum confirmations: ${paymentMethod.confirmation_blocks}`,
      `Estimated time: ${Math.round(paymentMethod.estimated_settlement_time / 60)} minutes`,
    ];

    if (paymentMethod.token_address) {
      instructions.push(`Token contract: ${paymentMethod.token_address}`);
    }

    switch (paymentMethod.chain) {
      case 'Polygon':
        instructions.push('Make sure to use Polygon network, not Ethereum mainnet');
        instructions.push('Gas fees are typically under $0.01');
        break;
      case 'Arbitrum':
        instructions.push('Use Arbitrum One network');
        instructions.push('Fast and low-cost Layer 2 solution');
        break;
      case 'Base':
        instructions.push('Use Base network (Coinbase L2)');
        instructions.push('Very low fees and fast confirmations');
        break;
      case 'Ethereum':
        instructions.push('Check gas fees before sending - they can be high');
        instructions.push('Consider using L2 networks for smaller amounts');
        break;
    }

    return instructions;
  }
}

export type { PaymentMethod, PaymentRequest, PaymentPurpose, PaymentStatus };