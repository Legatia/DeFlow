import React, { useState, useEffect } from 'react';
import DeFiTemplates from '../components/DeFiTemplates';
import StrategyCreationFlow from '../components/StrategyCreationFlow';
import CustomStrategyBuilder from '../components/CustomStrategyBuilder';
import simpleDefiTemplateService, { 
  DeFiWorkflowTemplate, 
  StrategyFromTemplateResponse 
} from '../services/defiTemplateServiceSimple';
import realProtocolService from '../services/realProtocolService';
import { StrategyConfig } from '../types/defi-strategy';

interface ActiveStrategy extends StrategyFromTemplateResponse {
  template: DeFiWorkflowTemplate;
  created_at: string;
  status: 'active' | 'paused' | 'completed';
  current_value: number;
  total_return: number;
  roi_percentage: number;
  market_data?: {
    current_apy?: number;
    protocol_health?: string;
    last_execution?: string;
  };
}

interface MarketOverview {
  totalTVL: number;
  avgYieldAPY: number;
  activeOpportunities: number;
  protocolsOnline: number;
}

type ViewMode = 'templates' | 'create' | 'dashboard' | 'custom';

const DeFiDashboard = () => {
  const [viewMode, setViewMode] = useState<ViewMode>('templates');
  const [selectedTemplate, setSelectedTemplate] = useState<DeFiWorkflowTemplate | null>(null);
  const [activeStrategies, setActiveStrategies] = useState<ActiveStrategy[]>([]);
  const [portfolioValue, setPortfolioValue] = useState(0);
  const [totalReturn, setTotalReturn] = useState(0);
  const [loading, setLoading] = useState(false);
  const [marketOverview, setMarketOverview] = useState<MarketOverview>({
    totalTVL: 0,
    avgYieldAPY: 0,
    activeOpportunities: 0,
    protocolsOnline: 0
  });

  useEffect(() => {
    loadPortfolioData();
  }, []);

  const loadPortfolioData = async () => {
    try {
      setLoading(true);
      
      // Load stored strategies
      const storedStrategies: ActiveStrategy[] = JSON.parse(
        localStorage.getItem('defi_strategies') || '[]'
      );
      
      // Get real market data to enhance stored strategies
      const [yieldData, arbitrageData, protocolHealth, marketData] = await Promise.all([
        realProtocolService.getYieldOpportunities().catch(() => null),
        realProtocolService.getArbitrageOpportunities().catch(() => null),
        realProtocolService.getProtocolHealth().catch(() => null),
        simpleDefiTemplateService.getMarketData().catch(() => ({
          avgYieldAPY: 5.0,
          avgArbitrageProfit: 1.2,
          totalTVL: 15000000000,
          activeOpportunities: 25
        }))
      ]);
      
      // Enhance strategies with real market data
      const enhancedStrategies = storedStrategies.map(strategy => {
        let currentAPY = strategy.template.estimated_apy;
        let protocolStatus = 'unknown';
        
        if (yieldData && strategy.template.category === 'YieldFarming') {
          const matchingOpportunity = yieldData.opportunities.find(opp => 
            opp.protocol.toLowerCase().includes(strategy.template.id.toLowerCase())
          );
          if (matchingOpportunity) {
            currentAPY = matchingOpportunity.apy;
            protocolStatus = 'healthy';
          }
        }
        
        return {
          ...strategy,
          market_data: {
            current_apy: currentAPY,
            protocol_health: protocolStatus,
            last_execution: new Date().toISOString()
          }
        };
      });
      
      setActiveStrategies(enhancedStrategies);
      
      // Calculate portfolio metrics
      const totalValue = enhancedStrategies.reduce((sum, strategy) => sum + strategy.current_value, 0);
      const totalRet = enhancedStrategies.reduce((sum, strategy) => sum + strategy.total_return, 0);
      
      setPortfolioValue(totalValue);
      setTotalReturn(totalRet);
      
      // Set market overview
      const protocolsOnline = [
        protocolHealth?.aave_status === 'healthy' ? 1 : 0,
        protocolHealth?.uniswap_status === 'healthy' ? 1 : 0,
        protocolHealth?.compound_status === 'healthy' ? 1 : 0,
        protocolHealth?.curve_status === 'healthy' ? 1 : 0
      ].reduce((sum, status) => sum + status, 0);
      
      setMarketOverview({
        totalTVL: marketData.totalTVL,
        avgYieldAPY: marketData.avgYieldAPY,
        activeOpportunities: marketData.activeOpportunities,
        protocolsOnline
      });
      
    } catch (error) {
      console.error('Error loading portfolio data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleTemplateSelect = (template: DeFiWorkflowTemplate) => {
    setSelectedTemplate(template);
    setViewMode('create');
  };

  const handleStrategyCreated = async (strategy: StrategyFromTemplateResponse) => {
    try {
      if (!selectedTemplate) return;

      // Create active strategy object
      const newStrategy: ActiveStrategy = {
        ...strategy,
        template: selectedTemplate,
        created_at: new Date().toISOString(),
        status: 'active',
        current_value: 1000, // Mock initial value
        total_return: 0,
        roi_percentage: 0
      };

      // Save to localStorage (in real app, save to backend)
      const existingStrategies = JSON.parse(localStorage.getItem('defi_strategies') || '[]');
      const updatedStrategies = [...existingStrategies, newStrategy];
      localStorage.setItem('defi_strategies', JSON.stringify(updatedStrategies));

      setActiveStrategies(updatedStrategies);
      setViewMode('dashboard');
      setSelectedTemplate(null);

      // Show success message
      alert('🎉 Strategy created successfully! You can now monitor it in your dashboard.');
    } catch (error) {
      console.error('Error saving strategy:', error);
    }
  };

  const handleCreateCustom = () => {
    setViewMode('custom');
  };

  const handleCustomStrategyCreated = async (strategyConfig: StrategyConfig) => {
    try {
      // Here we would call the backend API to create the custom strategy
      console.log('Creating custom strategy:', strategyConfig);
      
      // Mock the creation process for now
      const mockStrategy: ActiveStrategy = {
        strategy_id: `custom-${Date.now()}`,
        strategy_config: strategyConfig,
        estimated_setup_time: 300, // 5 minutes
        deployment_status: 'ready',
        template: {
          id: 'custom',
          name: strategyConfig.name,
          description: strategyConfig.description,
          category: 'Custom',
          difficulty: 'Advanced',
          estimated_apy: 10.0, // Placeholder
          risk_score: strategyConfig.risk_level,
          min_capital_usd: 100
        },
        created_at: new Date().toISOString(),
        status: 'active',
        current_value: strategyConfig.max_allocation_usd,
        total_return: 0,
        roi_percentage: 0
      };

      // Save to localStorage (in real app, would save to backend)
      const existingStrategies = JSON.parse(localStorage.getItem('defi_strategies') || '[]');
      const updatedStrategies = [...existingStrategies, mockStrategy];
      localStorage.setItem('defi_strategies', JSON.stringify(updatedStrategies));

      setActiveStrategies(updatedStrategies);
      setViewMode('dashboard');

      alert('🎉 Custom strategy created successfully! You can now monitor it in your dashboard.');
    } catch (error) {
      console.error('Error creating custom strategy:', error);
      alert('Failed to create custom strategy. Please try again.');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-100 text-green-800';
      case 'paused': return 'bg-yellow-100 text-yellow-800';
      case 'completed': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const renderDashboard = () => (
    <div className="max-w-7xl mx-auto p-6">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">DeFi Portfolio Dashboard</h1>
            <p className="text-gray-600 mt-1">Monitor and manage your automated DeFi strategies</p>
          </div>
          <button
            onClick={() => setViewMode('templates')}
            className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Add New Strategy
          </button>
        </div>
      </div>

      {/* Market Overview */}
      <div className="bg-gradient-to-r from-blue-600 to-purple-600 rounded-xl p-6 mb-8 text-white">
        <h2 className="text-xl font-semibold mb-4">DeFi Market Overview</h2>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="text-center">
            <div className="text-2xl font-bold">${(marketOverview.totalTVL / 1e9).toFixed(1)}B</div>
            <div className="text-blue-100 text-sm">Total TVL</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{marketOverview.avgYieldAPY.toFixed(1)}%</div>
            <div className="text-blue-100 text-sm">Avg APY</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{marketOverview.activeOpportunities}</div>
            <div className="text-blue-100 text-sm">Opportunities</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{marketOverview.protocolsOnline}/4</div>
            <div className="text-blue-100 text-sm">Protocols Online</div>
          </div>
        </div>
      </div>

      {/* Portfolio Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Total Portfolio Value</h3>
          <div className="text-2xl font-bold text-gray-900">
            ${portfolioValue.toLocaleString()}
          </div>
          {loading && <div className="text-xs text-gray-400 mt-1">Updating...</div>}
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Total Return</h3>
          <div className={`text-2xl font-bold ${totalReturn >= 0 ? 'text-green-600' : 'text-red-600'}`}>
            ${totalReturn.toLocaleString()}
          </div>
          {loading && <div className="text-xs text-gray-400 mt-1">Updating...</div>}
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Active Strategies</h3>
          <div className="text-2xl font-bold text-gray-900">
            {activeStrategies.filter(s => s.status === 'active').length}
          </div>
          {loading && <div className="text-xs text-gray-400 mt-1">Updating...</div>}
        </div>
      </div>

      {/* Active Strategies */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">Your Strategies</h2>
        </div>
        
        {activeStrategies.length === 0 ? (
          <div className="p-12 text-center">
            <div className="text-6xl mb-4">📊</div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No strategies yet</h3>
            <p className="text-gray-600 mb-4">
              Create your first DeFi strategy to start earning automated yields
            </p>
            <button
              onClick={() => setViewMode('templates')}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Browse Templates
            </button>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {activeStrategies.map((strategy) => (
              <div key={strategy.strategy_id} className="p-6 hover:bg-gray-50 transition-colors">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="text-3xl">
                      {simpleDefiTemplateService.getCategoryIcon(strategy.template.category)}
                    </div>
                    <div>
                      <h3 className="font-semibold text-gray-900">
                        {strategy.template.name}
                      </h3>
                      <p className="text-sm text-gray-600">
                        Created {new Date(strategy.created_at).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-6">
                    <div className="text-right">
                      <div className="text-sm text-gray-500">Current Value</div>
                      <div className="font-semibold text-gray-900">
                        ${strategy.current_value.toLocaleString()}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-gray-500">Return</div>
                      <div className={`font-semibold ${
                        strategy.total_return >= 0 ? 'text-green-600' : 'text-red-600'
                      }`}>
                        ${strategy.total_return.toLocaleString()} ({strategy.roi_percentage.toFixed(1)}%)
                      </div>
                    </div>
                    <span className={`px-3 py-1 text-xs rounded-full ${getStatusColor(strategy.status)}`}>
                      {strategy.status}
                    </span>
                  </div>
                </div>
                
                <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                  <div>
                    <span className="text-gray-500">Category:</span>
                    <span className="ml-2 font-medium">{strategy.template.category}</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Risk:</span>
                    <span className="ml-2 font-medium">{strategy.template.risk_score}/10</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Current APY:</span>
                    <span className="ml-2 font-medium text-green-600">
                      {strategy.market_data?.current_apy?.toFixed(1) || strategy.template.estimated_apy}%
                    </span>
                    {strategy.market_data?.current_apy && strategy.market_data.current_apy !== strategy.template.estimated_apy && (
                      <span className={`ml-1 text-xs ${strategy.market_data.current_apy > strategy.template.estimated_apy ? 'text-green-500' : 'text-red-500'}`}>
                        ({strategy.market_data.current_apy > strategy.template.estimated_apy ? '+' : ''}{(strategy.market_data.current_apy - strategy.template.estimated_apy).toFixed(1)}%)
                      </span>
                    )}
                  </div>
                  <div>
                    <span className="text-gray-500">Protocol:</span>
                    <span className="ml-2 font-medium capitalize">
                      {strategy.deployment_status}
                      {strategy.market_data?.protocol_health && (
                        <span className={`ml-1 text-xs px-1 rounded ${
                          strategy.market_data.protocol_health === 'healthy' 
                            ? 'bg-green-100 text-green-700' 
                            : 'bg-yellow-100 text-yellow-700'
                        }`}>
                          {strategy.market_data.protocol_health}
                        </span>
                      )}
                    </span>
                  </div>
                </div>
                
                {strategy.market_data?.last_execution && (
                  <div className="mt-3 text-xs text-gray-500">
                    Last updated: {new Date(strategy.market_data.last_execution).toLocaleString()}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );

  if (viewMode === 'create' && selectedTemplate) {
    return (
      <StrategyCreationFlow
        template={selectedTemplate}
        onStrategyCreated={handleStrategyCreated}
        onCancel={() => {
          setViewMode('templates');
          setSelectedTemplate(null);
        }}
      />
    );
  }

  if (viewMode === 'custom') {
    return (
      <CustomStrategyBuilder
        onStrategyCreated={handleCustomStrategyCreated}
        onCancel={() => setViewMode('templates')}
      />
    );
  }

  if (viewMode === 'templates') {
    return (
      <div>
        {/* Navigation */}
        {activeStrategies.length > 0 && (
          <div className="bg-white border-b border-gray-200 sticky top-0 z-10">
            <div className="max-w-7xl mx-auto px-6 py-4">
              <button
                onClick={() => setViewMode('dashboard')}
                className="flex items-center text-blue-600 hover:text-blue-700 font-medium"
              >
                ← Back to Dashboard
              </button>
            </div>
          </div>
        )}
        
        <DeFiTemplates
          onSelectTemplate={handleTemplateSelect}
          onCreateCustom={handleCreateCustom}
        />
      </div>
    );
  }

  return renderDashboard();
};

export default DeFiDashboard;