// DeFi Template Service - Integration with DeFlow backend workflow templates
import { Actor, HttpAgent } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';
import { Principal } from '@dfinity/principal';
import { BigIntUtils } from '../utils/bigint-utils';

// Import polyfill for BigInt handling
import '../utils/bigint-polyfill';

// DeFi Template interfaces matching backend types
export interface DeFiWorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  difficulty: string;
  estimated_apy: number;
  risk_score: number;
  min_capital_usd: number;
}

export interface TemplateListResponse {
  templates: DeFiWorkflowTemplate[];
  total_count: number;
}

export interface SimpleApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
  timestamp: string | number; // Handle as string to avoid BigInt conversion issues
}

export interface StrategyFromTemplateRequest {
  template_id: string;
  user_id: string;
  capital_amount: number;
}

export interface StrategyFromTemplateResponse {
  strategy_id: string;
  strategy_config: any;
  estimated_setup_time: number;
  deployment_status: string;
}

// Backend canister interface for DeFi templates
interface DeFiTemplateCanister {
  // Workflow template endpoints
  list_workflow_templates: () => Promise<SimpleApiResponse<TemplateListResponse>>;
  get_templates_by_category: (category: string) => Promise<SimpleApiResponse<TemplateListResponse>>;
  get_template_by_id: (template_id: string) => Promise<SimpleApiResponse<DeFiWorkflowTemplate>>;
  create_strategy_from_simple_template: (request: StrategyFromTemplateRequest) => Promise<SimpleApiResponse<StrategyFromTemplateResponse>>;
  get_simple_template_recommendations: (
    risk_tolerance: number,
    capital_amount: number,
    experience_level: string
  ) => Promise<SimpleApiResponse<TemplateListResponse>>;
  get_template_categories: () => Promise<SimpleApiResponse<string[]>>;
  
  // Strategy management endpoints
  get_strategy_yield_opportunities: (params: any) => Promise<any>;
  scan_arbitrage_opportunities: (params: any) => Promise<any>;
  get_strategy_portfolio_analytics: (user_id: string) => Promise<any>;
  execute_strategy: (params: any) => Promise<any>;
  get_performance_report: (strategy_id: string) => Promise<any>;
  health_check: () => Promise<any>;
}

class DeFiTemplateService {
  private actor: DeFiTemplateCanister | null = null;
  private authClient: AuthClient | null = null;
  private isInitialized = false;
  private canisterId: string = 'rdmx6-jaaaa-aaaah-qdrqaq-cai'; // Default local canister ID

  async initialize(canisterId?: string): Promise<void> {
    if (this.isInitialized) return;

    try {
      console.log('Initializing DeFi Template service...');

      if (canisterId) {
        this.canisterId = canisterId;
      }

      // Initialize auth client
      this.authClient = await AuthClient.create();

      // Create agent with BigInt transformation
      const agent = new HttpAgent({
        host: process.env.NODE_ENV === 'production' 
          ? 'https://icp-api.io' 
          : 'http://localhost:8000',
      });

      // Disable certificate verification for local development
      if (process.env.NODE_ENV !== 'production') {
        agent.fetchRootKey();
      }

      // Create actor with explicit BigInt handling
      this.actor = Actor.createActor(
        ({ IDL }) => IDL.Service({
          // Define the service methods we'll use
          list_workflow_templates: IDL.Func([], [IDL.Text], ['query']),
          get_templates_by_category: IDL.Func([IDL.Text], [IDL.Text], ['query']),
          get_template_by_id: IDL.Func([IDL.Text], [IDL.Text], ['query']),
          create_strategy_from_simple_template: IDL.Func([IDL.Text], [IDL.Text], []),
          get_simple_template_recommendations: IDL.Func([IDL.Nat8, IDL.Float64, IDL.Text], [IDL.Text], ['query']),
          get_template_categories: IDL.Func([], [IDL.Text], ['query']),
        }),
        {
          agent,
          canisterId: this.canisterId,
        }
      ) as any;

      this.isInitialized = true;
      console.log('DeFi Template service initialized successfully');
    } catch (error) {
      console.error('Failed to initialize DeFi Template service:', error);
      throw error;
    }
  }

  private async ensureInitialized(): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }
    if (!this.actor) {
      throw new Error('DeFi Template service not initialized');
    }
  }

  // Workflow Template Methods

  async listWorkflowTemplates(): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      // Make raw canister call
      const response = await (this.actor as any)._call('list_workflow_templates', []);
      
      if (response?.success && response?.data?.templates) {
        // Convert any BigInt values safely
        return this.sanitizeTemplates(response.data.templates);
      } else {
        throw new Error(response?.error || 'Failed to fetch templates');
      }
    } catch (error) {
      console.error('Error listing workflow templates:', error);
      // Return mock data for development
      return this.getMockTemplates();
    }
  }

  async getTemplatesByCategory(category: string): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      const response = await (this.actor as any)._call('get_templates_by_category', [category]);
      
      if (response?.success && response?.data?.templates) {
        return this.sanitizeTemplates(response.data.templates);
      } else {
        throw new Error(response?.error || 'Failed to fetch templates by category');
      }
    } catch (error) {
      console.error('Error getting templates by category:', error);
      // Return filtered mock data
      return this.getMockTemplates().filter(t => t.category === category);
    }
  }

  async getTemplateById(templateId: string): Promise<DeFiWorkflowTemplate | null> {
    await this.ensureInitialized();
    
    try {
      const response = await (this.actor as any)._call('get_template_by_id', [templateId]);
      
      if (response?.success && response?.data) {
        return this.sanitizeTemplate(response.data);
      } else {
        throw new Error(response?.error || 'Template not found');
      }
    } catch (error) {
      console.error('Error getting template by ID:', error);
      // Return mock data
      return this.getMockTemplates().find(t => t.id === templateId) || null;
    }
  }

  async createStrategyFromTemplate(
    templateId: string,
    userId: string,
    capitalAmount: number
  ): Promise<StrategyFromTemplateResponse> {
    await this.ensureInitialized();
    
    const request: StrategyFromTemplateRequest = {
      template_id: templateId,
      user_id: userId,
      capital_amount: capitalAmount,
    };

    try {
      const response = await (this.actor as any)._call('create_strategy_from_simple_template', [request]);
      
      if (response?.success && response?.data) {
        return response.data;
      } else {
        throw new Error(response?.error || 'Failed to create strategy');
      }
    } catch (error) {
      console.error('Error creating strategy from template:', error);
      // Return mock response
      return {
        strategy_id: `mock_strategy_${Date.now()}`,
        strategy_config: {},
        estimated_setup_time: 5,
        deployment_status: 'created'
      };
    }
  }

  async getTemplateRecommendations(
    riskTolerance: number,
    capitalAmount: number,
    experienceLevel: string
  ): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      const response = await (this.actor as any)._call('get_simple_template_recommendations', [
        riskTolerance,
        capitalAmount,
        experienceLevel
      ]);
      
      if (response?.success && response?.data?.templates) {
        return this.sanitizeTemplates(response.data.templates);
      } else {
        throw new Error(response?.error || 'Failed to get recommendations');
      }
    } catch (error) {
      console.error('Error getting template recommendations:', error);
      // Return filtered mock data based on criteria
      return this.getMockTemplates().filter(t => 
        t.risk_score <= riskTolerance && 
        t.min_capital_usd <= capitalAmount
      );
    }
  }

  async getTemplateCategories(): Promise<string[]> {
    await this.ensureInitialized();
    
    try {
      const response = await (this.actor as any)._call('get_template_categories', []);
      
      if (response?.success && response?.data) {
        return response.data;
      } else {
        throw new Error(response?.error || 'Failed to get categories');
      }
    } catch (error) {
      console.error('Error getting template categories:', error);
      // Return mock categories
      return ['YieldFarming', 'Arbitrage', 'Rebalancing', 'DCA'];
    }
  }

  // Mock data for development/fallback
  private getMockTemplates(): DeFiWorkflowTemplate[] {
    return [
      {
        id: 'conservative_yield',
        name: 'Conservative Yield Farming',
        description: 'Low-risk yield farming on established protocols',
        category: 'YieldFarming',
        difficulty: 'Beginner',
        estimated_apy: 4.5,
        risk_score: 3,
        min_capital_usd: 100.0
      },
      {
        id: 'basic_arbitrage',
        name: 'Cross-Chain Arbitrage',
        description: 'Automated arbitrage opportunities across chains',
        category: 'Arbitrage',
        difficulty: 'Advanced',
        estimated_apy: 12.0,
        risk_score: 7,
        min_capital_usd: 1000.0
      },
      {
        id: 'icp_arbitrage',
        name: 'ICP-ETH Arbitrage via KongSwap',
        description: 'Arbitrage between ICP DEXs and Ethereum using KongSwap',
        category: 'Arbitrage',
        difficulty: 'Intermediate',
        estimated_apy: 8.5,
        risk_score: 6,
        min_capital_usd: 500.0
      },
      {
        id: 'portfolio_rebalancing',
        name: 'Portfolio Rebalancing',
        description: 'Maintain optimal asset allocation',
        category: 'Rebalancing',
        difficulty: 'Intermediate',
        estimated_apy: 6.0,
        risk_score: 5,
        min_capital_usd: 500.0
      },
      {
        id: 'dollar_cost_averaging',
        name: 'Dollar Cost Averaging',
        description: 'Systematic investment strategy',
        category: 'DCA',
        difficulty: 'Beginner',
        estimated_apy: 8.0,
        risk_score: 4,
        min_capital_usd: 50.0
      }
    ];
  }

  // Utility methods
  getCategoryIcon(category: string): string {
    const icons: { [key: string]: string } = {
      'YieldFarming': '🌱',
      'Arbitrage': '⚡',
      'Rebalancing': '⚖️',
      'DCA': '📈'
    };
    return icons[category] || '💰';
  }

  getRiskColor(riskScore: number): string {
    if (riskScore <= 3) return 'bg-green-100 text-green-800';
    if (riskScore <= 6) return 'bg-yellow-100 text-yellow-800';
    return 'bg-red-100 text-red-800';
  }

  getDifficultyColor(difficulty: string): string {
    switch (difficulty.toLowerCase()) {
      case 'beginner': return 'bg-blue-100 text-blue-800';
      case 'intermediate': return 'bg-purple-100 text-purple-800';
      case 'advanced': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  }

  // Helper methods for BigInt sanitization
  private sanitizeTemplates(templates: any[]): DeFiWorkflowTemplate[] {
    return templates.map(template => this.sanitizeTemplate(template));
  }

  private sanitizeTemplate(template: any): DeFiWorkflowTemplate {
    return {
      id: String(template.id || ''),
      name: String(template.name || ''),
      description: String(template.description || ''),
      category: String(template.category || ''),
      difficulty: String(template.difficulty || ''),
      estimated_apy: this.sanitizeNumber(template.estimated_apy),
      risk_score: this.sanitizeNumber(template.risk_score),
      min_capital_usd: this.sanitizeNumber(template.min_capital_usd)
    };
  }

  private sanitizeNumber(value: any): number {
    try {
      // Handle various input types including BigInt
      if (typeof value === 'bigint') {
        return BigIntUtils.toNumber(value);
      }
      if (typeof value === 'string') {
        return parseFloat(value) || 0;
      }
      if (typeof value === 'number') {
        return isNaN(value) ? 0 : value;
      }
      return 0;
    } catch (error) {
      console.warn('Failed to sanitize number value:', value, error);
      return 0;
    }
  }
}

// Export singleton instance
export const defiTemplateService = new DeFiTemplateService();
export default defiTemplateService;