import React, { useState, useEffect } from 'react';
import { WORKFLOW_TEMPLATES, TEMPLATE_CATEGORIES, WorkflowTemplate } from '../data/workflowTemplates';
import { useNavigate } from 'react-router-dom';

interface DeFiTemplatesProps {
  onSelectTemplate?: (template: any) => void;
  onCreateCustom: () => void;
}

const DeFiTemplates = ({ onSelectTemplate, onCreateCustom }: DeFiTemplatesProps) => {
  const navigate = useNavigate();
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>('all');
  
  // No DeFi templates available - return empty array
  const defiTemplates: WorkflowTemplate[] = [];
  
  // Get unique subcategories from DeFi templates based on their tags
  const defiCategories = ['all', ...new Set(defiTemplates.flatMap(template => 
    (template.tags || []).filter(tag => ['arbitrage', 'yield-farming', 'rebalancing', 'cross-chain', 'solana'].includes(tag))
  ))];

  // Filter templates based on selection with safety checks
  const filteredTemplates = defiTemplates.filter(template => {
    if (!template || !template.tags || !Array.isArray(template.tags)) return false;
    const matchesCategory = selectedCategory === 'all' || template.tags.includes(selectedCategory);
    const matchesDifficulty = selectedDifficulty === 'all' || template.difficulty === selectedDifficulty;
    return matchesCategory && matchesDifficulty;
  });

  const handleCreateStrategy = (template: WorkflowTemplate) => {
    // Navigate to workflow editor with the template
    navigate(`/workflows/new?template=${template.id}`);
  };

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'beginner': return 'bg-green-100 text-green-800';
      case 'intermediate': return 'bg-yellow-100 text-yellow-800';
      case 'advanced': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getEstimatedAPY = (template: WorkflowTemplate) => {
    if (!template.tags || !Array.isArray(template.tags)) return '6-10%';
    if (template.tags.includes('yield-farming')) return '8-15%';
    if (template.tags.includes('arbitrage')) return '5-12%';
    if (template.tags.includes('rebalancing')) return '4-8%';
    return '6-10%';
  };

  const getRiskScore = (template: WorkflowTemplate) => {
    if (!template.difficulty) return 5;
    if (template.difficulty === 'beginner') return 3;
    if (template.difficulty === 'intermediate') return 5;
    return 7;
  };

  const getMinCapital = (template: WorkflowTemplate) => {
    if (!template.tags || !Array.isArray(template.tags)) return '$500';
    if (template.tags.includes('solana')) return '$100';
    if (template.tags.includes('arbitrage')) return '$1,000';
    return '$500';
  };

  return (
    <div className="max-w-7xl mx-auto p-6">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold text-gray-900 mb-2">DeFi Strategy Templates</h1>
        <p className="text-gray-600 text-lg">
          Choose from battle-tested DeFi strategies or create your own
        </p>
      </div>

      {/* Create Custom Strategy Option */}
      <div className="mb-8">
        <div 
          onClick={onCreateCustom}
          className="bg-gradient-to-r from-purple-500 to-pink-600 rounded-xl p-6 text-white cursor-pointer hover:from-purple-600 hover:to-pink-700 transition-all duration-200 transform hover:scale-105 shadow-lg"
        >
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-2xl font-bold mb-2">Create Custom Strategy</h3>
              <p className="text-purple-100 text-lg">
                Build advanced DeFi strategies with our visual workflow builder
              </p>
            </div>
            <div className="text-5xl">🚀</div>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-4 mb-6 p-4 bg-gray-50 rounded-xl">
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">Category:</label>
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="all">All Categories</option>
            {defiCategories.filter(cat => cat !== 'all').map(category => (
              <option key={category} value={category}>
                {category.charAt(0).toUpperCase() + category.slice(1).replace('-', ' ')}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">Difficulty:</label>
          <select
            value={selectedDifficulty}
            onChange={(e) => setSelectedDifficulty(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="all">All Levels</option>
            <option value="beginner">Beginner</option>
            <option value="intermediate">Intermediate</option>
            <option value="advanced">Advanced</option>
          </select>
        </div>

        <div className="ml-auto text-sm text-gray-600">
          {filteredTemplates.length} template{filteredTemplates.length !== 1 ? 's' : ''} found
        </div>
      </div>

      {/* Templates Grid */}
      {filteredTemplates.length === 0 ? (
        <div className="text-center py-12">
          <div className="text-6xl mb-4">🔍</div>
          <h3 className="text-xl font-medium text-gray-900 mb-2">No templates found</h3>
          <p className="text-gray-600">Try adjusting your filters or create a custom strategy</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredTemplates.map((template) => (
            <div
              key={template.id}
              onClick={() => handleCreateStrategy(template)}
              className="bg-white rounded-xl border border-gray-200 hover:border-blue-300 hover:shadow-xl transition-all duration-200 cursor-pointer group overflow-hidden"
            >
              {/* Template Header */}
              <div className="p-6 border-b border-gray-100">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center space-x-3">
                    <span className="text-3xl">💰</span>
                    <div>
                      <h3 className="font-bold text-gray-900 group-hover:text-blue-600 transition-colors">
                        {template.name}
                      </h3>
                      <span className={`inline-block px-2 py-1 text-xs rounded-full ${getDifficultyColor(template.difficulty)}`}>
                        {template.difficulty}
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`inline-block px-2 py-1 text-xs rounded-full ${getRiskScore(template) <= 3 ? 'bg-green-100 text-green-800' : getRiskScore(template) <= 6 ? 'bg-yellow-100 text-yellow-800' : 'bg-red-100 text-red-800'}`}>
                      Risk: {getRiskScore(template)}/10
                    </div>
                  </div>
                </div>
                <p className="text-sm text-gray-600 line-clamp-2">
                  {template.description}
                </p>
              </div>

              {/* Template Metrics */}
              <div className="p-6">
                <div className="grid grid-cols-2 gap-4 mb-4">
                  <div className="text-center">
                    <div className="text-2xl font-bold text-green-600">
                      {getEstimatedAPY(template)}
                    </div>
                    <div className="text-xs text-gray-500">Est. APY</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-blue-600">
                      {getMinCapital(template)}
                    </div>
                    <div className="text-xs text-gray-500">Min Capital</div>
                  </div>
                </div>

                <div className="text-center mb-4">
                  <span className="text-xs text-gray-500">{template.estimatedTime || '15 minutes'}</span>
                </div>

                <div className="flex flex-wrap gap-1 mb-4">
                  {(template.tags || []).slice(0, 3).map(tag => (
                    <span key={tag} className="inline-block px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-700">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>

              {/* Template Actions */}
              <div className="p-6 pt-0">
                <button
                  className="w-full px-4 py-3 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg hover:from-blue-700 hover:to-purple-700 transition-all duration-200 font-medium group-hover:shadow-md"
                >
                  Create Strategy
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default DeFiTemplates;