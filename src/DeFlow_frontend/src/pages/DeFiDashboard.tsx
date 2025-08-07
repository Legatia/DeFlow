import React, { useState, useEffect } from 'react';
import DeFiTemplates from '../components/DeFiTemplates';
import StrategyCreationFlow from '../components/StrategyCreationFlow';
import simpleDefiTemplateService, { 
  DeFiWorkflowTemplate, 
  StrategyFromTemplateResponse 
} from '../services/defiTemplateServiceSimple';

interface ActiveStrategy extends StrategyFromTemplateResponse {
  template: DeFiWorkflowTemplate;
  created_at: string;
  status: 'active' | 'paused' | 'completed';
  current_value: number;
  total_return: number;
  roi_percentage: number;
}

type ViewMode = 'templates' | 'create' | 'dashboard';

const DeFiDashboard = () => {
  const [viewMode, setViewMode] = useState<ViewMode>('templates');
  const [selectedTemplate, setSelectedTemplate] = useState<DeFiWorkflowTemplate | null>(null);
  const [activeStrategies, setActiveStrategies] = useState<ActiveStrategy[]>([]);
  const [portfolioValue, setPortfolioValue] = useState(0);
  const [totalReturn, setTotalReturn] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadPortfolioData();
  }, []);

  const loadPortfolioData = async () => {
    try {
      setLoading(true);
      
      // In a real app, this would fetch from the backend
      // For now, we'll use mock data
      const mockStrategies: ActiveStrategy[] = JSON.parse(
        localStorage.getItem('defi_strategies') || '[]'
      );
      
      setActiveStrategies(mockStrategies);
      
      // Calculate portfolio metrics
      const totalValue = mockStrategies.reduce((sum, strategy) => sum + strategy.current_value, 0);
      const totalRet = mockStrategies.reduce((sum, strategy) => sum + strategy.total_return, 0);
      
      setPortfolioValue(totalValue);
      setTotalReturn(totalRet);
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
    // In a full implementation, this would open the workflow builder
    alert('Custom strategy builder coming soon! For now, please choose from our templates.');
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

      {/* Portfolio Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Total Portfolio Value</h3>
          <div className="text-2xl font-bold text-gray-900">
            ${portfolioValue.toLocaleString()}
          </div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Total Return</h3>
          <div className={`text-2xl font-bold ${totalReturn >= 0 ? 'text-green-600' : 'text-red-600'}`}>
            ${totalReturn.toLocaleString()}
          </div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-sm font-medium text-gray-500 mb-2">Active Strategies</h3>
          <div className="text-2xl font-bold text-gray-900">
            {activeStrategies.filter(s => s.status === 'active').length}
          </div>
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
                
                <div className="mt-4 grid grid-cols-4 gap-4 text-sm">
                  <div>
                    <span className="text-gray-500">Category:</span>
                    <span className="ml-2 font-medium">{strategy.template.category}</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Risk:</span>
                    <span className="ml-2 font-medium">{strategy.template.risk_score}/10</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Target APY:</span>
                    <span className="ml-2 font-medium text-green-600">
                      {strategy.template.estimated_apy}%
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500">Status:</span>
                    <span className="ml-2 font-medium capitalize">{strategy.deployment_status}</span>
                  </div>
                </div>
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