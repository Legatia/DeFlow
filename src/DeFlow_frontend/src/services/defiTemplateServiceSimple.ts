// Simple DeFi Template Service - Direct canister calls to avoid BigInt issues
import { BigIntUtils } from '../utils/bigint-utils';
import realProtocolService from './realProtocolService';

// Import polyfill for BigInt handling
import '../utils/bigint-polyfill';

// DeFi Template interfaces
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

class SimpleDefiTemplateService {
  private canisterId: string = 'rdmx6-jaaaa-aaaah-qdrqaq-cai'; // Default local canister ID
  private isInitialized = false;

  async initialize(canisterId?: string): Promise<void> {
    if (this.isInitialized) return;

    try {

      if (canisterId) {
        this.canisterId = canisterId;
      }

      this.isInitialized = true;
    } catch (error) {
      console.error('Failed to initialize Simple DeFi Template service:', error);
      throw error;
    }
  }

  private async ensureInitialized(): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }
  }

  // Make real canister calls to DeFlow backend
  private async callCanister(method: string, args: any[] = []): Promise<any> {
    try {
      // Import the backend canister declarations
      const { createActor } = await import('../../../declarations/DeFlow_backend');
      const { idlFactory } = await import('../../../declarations/DeFlow_backend');
      
      // Create actor for backend canister
      const backendActor = createActor(process.env.VITE_CANISTER_ID_DEFLOW_BACKEND || this.canisterId, {
        agentOptions: {
          host: process.env.NODE_ENV === 'production' ? 'https://ic0.app' : 'http://127.0.0.1:8080'
        }
      });
      
      // Make the actual canister call
      const result = await (backendActor as any)[method](...args);
      
      return result;
    } catch (error) {
      console.error(`Canister call failed for ${method}:`, error);
      throw new Error(`Backend service unavailable: ${method}`);
    }
  }

  // Handle canister response format
  private processCanisterResponse(response: any, method: string): any {
    // Handle different response formats from the canister
    if (response && typeof response === 'object') {
      // If response has 'Ok' field (Result type)
      if ('Ok' in response) {
        return {
          success: true,
          data: response.Ok,
          error: null,
          timestamp: Date.now()
        };
      }
      // If response has 'Err' field (Result type)
      if ('Err' in response) {
        return {
          success: false,
          data: null,
          error: response.Err,
          timestamp: Date.now()
        };
      }
      // Direct response format
      return {
        success: true,
        data: response,
        error: null,
        timestamp: Date.now()
      };
    }
    
    throw new Error(`Invalid response format from ${method}`);
  }

  // Public API methods
  async listWorkflowTemplates(): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      // First try to get real protocol data to update template APYs
      await this.updateTemplatesWithRealData();
      
      const response = await this.callCanister('list_workflow_templates');
      
      const processedResponse = this.processCanisterResponse(response, 'list_workflow_templates');
      
      if (processedResponse.success && processedResponse.data) {
        // Handle different possible data structures
        const templates = processedResponse.data.templates || processedResponse.data || [];
        return this.sanitizeTemplates(Array.isArray(templates) ? templates : []);
      } else {
        throw new Error(processedResponse.error || 'Failed to fetch templates');
      }
    } catch (error) {
      console.error('Error listing workflow templates:', error);
      throw error; // Don't fall back to mock data for production
    }
  }

  async getTemplatesByCategory(category: string): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      const response = await this.callCanister('get_templates_by_category', [category]);
      
      if (response?.success && response?.data?.templates) {
        return this.sanitizeTemplates(response.data.templates);
      } else {
        throw new Error(response?.error || 'Failed to fetch templates by category');
      }
    } catch (error) {
      console.error('Error getting templates by category:', error);
      throw error; // Don't fall back to mock data
    }
  }

  async getTemplateById(templateId: string): Promise<DeFiWorkflowTemplate | null> {
    await this.ensureInitialized();
    
    try {
      const response = await this.callCanister('get_template_by_id', [templateId]);
      
      if (response?.success && response?.data) {
        return this.sanitizeTemplate(response.data);
      } else {
        throw new Error(response?.error || 'Template not found');
      }
    } catch (error) {
      console.error('Error getting template by ID:', error);
      throw error; // Don't fall back to mock data
    }
  }

  async createStrategyFromTemplate(
    templateId: string,
    userId: string,
    capitalAmount: number
  ): Promise<StrategyFromTemplateResponse> {
    await this.ensureInitialized();
    
    const request = {
      template_id: templateId,
      user_id: userId,
      capital_amount: capitalAmount,
    };

    try {
      // Update market data before creating strategy
      await this.updateTemplatesWithRealData();
      
      const response = await this.callCanister('create_strategy_from_simple_template', [request]);
      
      if (response?.success && response?.data) {
        // Enhanced response with real market context
        return {
          ...response.data,
          market_context: {
            current_avg_apy: this.realMarketData.avgYieldAPY,
            market_volatility: this.realMarketData.marketVolatility,
            last_updated: this.realMarketData.lastUpdated
          }
        };
      } else {
        throw new Error(response?.error || 'Failed to create strategy');
      }
    } catch (error) {
      console.error('Error creating strategy from template:', error);
      
      // Return error instead of mock data for production
      throw error;
    }
  }

  async getTemplateRecommendations(
    riskTolerance: number,
    capitalAmount: number,
    experienceLevel: string
  ): Promise<DeFiWorkflowTemplate[]> {
    await this.ensureInitialized();
    
    try {
      const response = await this.callCanister('get_simple_template_recommendations', [
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
      throw error; // Don't fall back to mock data
    }
  }

  async getTemplateCategories(): Promise<string[]> {
    await this.ensureInitialized();
    
    try {
      const response = await this.callCanister('get_template_categories');
      
      if (response?.success && response?.data) {
        return response.data;
      } else {
        throw new Error(response?.error || 'Failed to get categories');
      }
    } catch (error) {
      console.error('Error getting template categories:', error);
      return ['YieldFarming', 'Arbitrage', 'Rebalancing', 'DCA'];
    }
  }

  // Updated templates with real market data
  private realMarketData: { 
    avgYieldAPY: number; 
    avgArbitrageProfit: number; 
    marketVolatility: number; 
    lastUpdated: number;
  } = {
    avgYieldAPY: 5.0,
    avgArbitrageProfit: 1.2,
    marketVolatility: 0.15,
    lastUpdated: 0
  };

  private async updateTemplatesWithRealData(): Promise<void> {
    try {
      // Update market data every 5 minutes
      if (Date.now() - this.realMarketData.lastUpdated < 5 * 60 * 1000) {
        return;
      }

      
      // Get real yield opportunities
      const yieldData = await realProtocolService.getYieldOpportunities();
      if (yieldData.opportunities.length > 0) {
        this.realMarketData.avgYieldAPY = yieldData.market_summary.average_apy;
      }

      // Get real arbitrage opportunities  
      const arbData = await realProtocolService.getArbitrageOpportunities();
      if (arbData.opportunities.length > 0) {
        this.realMarketData.avgArbitrageProfit = arbData.opportunities
          .reduce((sum, opp) => sum + opp.profit_percentage, 0) / arbData.opportunities.length;
      }

      this.realMarketData.lastUpdated = Date.now();
    } catch (error) {
      console.warn('Failed to update templates with real data:', error);
    }
  }

  // REMOVED: All mock template data has been removed for production deployment

  /**
   * Get real-time market data for display
   */
  async getMarketData(): Promise<{ 
    avgYieldAPY: number; 
    avgArbitrageProfit: number; 
    totalTVL: number;
    activeOpportunities: number;
  }> {
    try {
      const [yieldData, arbData] = await Promise.all([
        realProtocolService.getYieldOpportunities(),
        realProtocolService.getArbitrageOpportunities()
      ]);

      return {
        avgYieldAPY: yieldData.market_summary.average_apy,
        avgArbitrageProfit: arbData.opportunities.length > 0 
          ? arbData.opportunities.reduce((sum, opp) => sum + opp.profit_percentage, 0) / arbData.opportunities.length
          : 0,
        totalTVL: yieldData.market_summary.total_tvl,
        activeOpportunities: yieldData.total_count + arbData.total_count
      };
    } catch (error) {
      console.error('Failed to get market data:', error);
      return {
        avgYieldAPY: 5.0,
        avgArbitrageProfit: 1.2,
        totalTVL: 15000000000,
        activeOpportunities: 25
      };
    }
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

  // Helper methods for data sanitization
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
export const simpleDefiTemplateService = new SimpleDefiTemplateService();
export default simpleDefiTemplateService;