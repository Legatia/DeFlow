// Subscription management service for tracking user plans and status
export type SubscriptionPlan = 'standard' | 'premium' | 'pro';

export interface UserSubscription {
  plan: SubscriptionPlan;
  status: 'active' | 'expired' | 'cancelled' | 'pending';
  startDate: Date;
  endDate?: Date;
  paymentId?: string;
  autoRenew: boolean;
}

export interface SubscriptionFeatures {
  feeRate: string;
  executionSpeed: 'standard' | 'priority' | 'instant';
  support: 'community' | 'email' | 'priority';
  analytics: boolean;
  apiAccess: boolean;
  customStrategies: boolean;
  portfolioInsurance: boolean;
}

class SubscriptionService {
  private static readonly STORAGE_KEY = 'deflow_user_subscription';
  private static readonly DEFAULT_SUBSCRIPTION: UserSubscription = {
    plan: 'standard',
    status: 'active',
    startDate: new Date(),
    autoRenew: false,
  };

  /**
   * Get current user subscription
   */
  static getCurrentSubscription(): UserSubscription {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (stored) {
        const subscription = JSON.parse(stored);
        // Convert date strings back to Date objects
        subscription.startDate = new Date(subscription.startDate);
        if (subscription.endDate) {
          subscription.endDate = new Date(subscription.endDate);
        }
        
        // Check if subscription has expired
        if (subscription.endDate && new Date() > subscription.endDate) {
          subscription.status = 'expired';
          subscription.plan = 'standard'; // Downgrade to standard when expired
          this.saveSubscription(subscription);
        }
        
        return subscription;
      }
    } catch (error) {
      console.error('Error loading subscription:', error);
    }
    
    return { ...this.DEFAULT_SUBSCRIPTION };
  }

  /**
   * Save subscription to local storage
   */
  private static saveSubscription(subscription: UserSubscription): void {
    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(subscription));
    } catch (error) {
      console.error('Error saving subscription:', error);
    }
  }

  /**
   * Update subscription plan after successful payment
   */
  static activateSubscription(plan: SubscriptionPlan, paymentId: string, durationMonths: number = 1): void {
    const now = new Date();
    const endDate = new Date();
    endDate.setMonth(endDate.getMonth() + durationMonths);

    const subscription: UserSubscription = {
      plan,
      status: 'active',
      startDate: now,
      endDate: plan === 'standard' ? undefined : endDate, // Standard plan doesn't expire
      paymentId,
      autoRenew: plan !== 'standard',
    };

    this.saveSubscription(subscription);
    this.notifySubscriptionChange(subscription);
  }

  /**
   * Cancel subscription (immediately downgrade to standard)
   */
  static cancelSubscription(): void {
    const current = this.getCurrentSubscription();
    if (current.plan !== 'standard') {
      // Immediately downgrade to standard plan
      const standardSubscription: UserSubscription = {
        plan: 'standard',
        status: 'active',
        startDate: new Date(),
        endDate: undefined, // Standard plan doesn't expire
        paymentId: 'cancelled',
        autoRenew: false,
      };
      
      this.saveSubscription(standardSubscription);
      this.notifySubscriptionChange(standardSubscription);
    }
  }

  /**
   * Get subscription features for a plan
   */
  static getSubscriptionFeatures(plan: SubscriptionPlan): SubscriptionFeatures {
    switch (plan) {
      case 'premium':
        return {
          feeRate: '0.25%',
          executionSpeed: 'priority',
          support: 'email',
          analytics: true,
          apiAccess: false,
          customStrategies: false,
          portfolioInsurance: false,
        };
      case 'pro':
        return {
          feeRate: '0.1%',
          executionSpeed: 'instant',
          support: 'priority',
          analytics: true,
          apiAccess: true,
          customStrategies: true,
          portfolioInsurance: true,
        };
      case 'standard':
      default:
        return {
          feeRate: '0.85%',
          executionSpeed: 'standard',
          support: 'community',
          analytics: false,
          apiAccess: false,
          customStrategies: false,
          portfolioInsurance: false,
        };
    }
  }

  /**
   * Check if user has access to a specific feature
   */
  static hasFeature(feature: keyof SubscriptionFeatures): boolean {
    const subscription = this.getCurrentSubscription();
    const features = this.getSubscriptionFeatures(subscription.plan);
    return Boolean(features[feature]);
  }

  /**
   * Get days remaining in current subscription
   */
  static getDaysRemaining(): number | null {
    const subscription = this.getCurrentSubscription();
    if (!subscription.endDate || subscription.plan === 'standard') {
      return null; // No expiration for standard plan
    }

    const now = new Date();
    const timeDiff = subscription.endDate.getTime() - now.getTime();
    const daysDiff = Math.ceil(timeDiff / (1000 * 3600 * 24));
    
    return Math.max(0, daysDiff);
  }

  /**
   * Check if subscription is expiring soon (within 7 days)
   */
  static isExpiringSoon(): boolean {
    const daysRemaining = this.getDaysRemaining();
    return daysRemaining !== null && daysRemaining <= 7;
  }

  /**
   * Get subscription display text for UI
   */
  static getSubscriptionDisplayText(): string {
    const subscription = this.getCurrentSubscription();
    
    switch (subscription.plan) {
      case 'premium':
        return 'Premium Plan';
      case 'pro':
        return 'Pro Plan';
      case 'standard':
      default:
        return 'Standard Plan';
    }
  }

  /**
   * Get subscription status color for UI indicators
   */
  static getSubscriptionColor(): string {
    const subscription = this.getCurrentSubscription();
    
    if (subscription.status === 'expired' || subscription.status === 'cancelled') {
      return 'bg-red-500';
    }
    
    switch (subscription.plan) {
      case 'pro':
        return 'bg-purple-500';
      case 'premium':
        return 'bg-blue-500';
      case 'standard':
      default:
        return 'bg-gray-500';
    }
  }

  /**
   * Subscription change listeners for reactive updates
   */
  private static listeners: ((subscription: UserSubscription) => void)[] = [];

  static addSubscriptionListener(listener: (subscription: UserSubscription) => void): void {
    this.listeners.push(listener);
  }

  static removeSubscriptionListener(listener: (subscription: UserSubscription) => void): void {
    const index = this.listeners.indexOf(listener);
    if (index > -1) {
      this.listeners.splice(index, 1);
    }
  }

  private static notifySubscriptionChange(subscription: UserSubscription): void {
    this.listeners.forEach(listener => {
      try {
        listener(subscription);
      } catch (error) {
        console.error('Error in subscription listener:', error);
      }
    });
  }

  /**
   * Reset subscription to default (for testing/development)
   */
  static resetSubscription(): void {
    localStorage.removeItem(this.STORAGE_KEY);
    this.notifySubscriptionChange({ ...this.DEFAULT_SUBSCRIPTION });
  }
}

export default SubscriptionService;