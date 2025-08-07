import React, { useState, useEffect } from 'react';
import simpleDefiTemplateService, { DeFiWorkflowTemplate } from '../services/defiTemplateServiceSimple';

interface DeFiTemplatesProps {
  onSelectTemplate: (template: DeFiWorkflowTemplate) => void;
  onCreateCustom: () => void;
}

const DeFiTemplates = ({ onSelectTemplate, onCreateCustom }: DeFiTemplatesProps) => {
  const [templates, setTemplates] = useState<DeFiWorkflowTemplate[]>([]);
  const [categories, setCategories] = useState<string[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>('all');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // User preferences for recommendations
  const [riskTolerance, setRiskTolerance] = useState(5);
  const [capitalAmount, setCapitalAmount] = useState(1000);
  const [experienceLevel, setExperienceLevel] = useState('Beginner');
  const [showRecommendations, setShowRecommendations] = useState(false);
  const [recommendations, setRecommendations] = useState<DeFiWorkflowTemplate[]>([]);

  useEffect(() => {
    loadTemplatesAndCategories();
  }, []);

  const loadTemplatesAndCategories = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Initialize the service
      await simpleDefiTemplateService.initialize();
      
      // Load templates and categories in parallel
      const [templatesData, categoriesData] = await Promise.all([
        simpleDefiTemplateService.listWorkflowTemplates(),
        simpleDefiTemplateService.getTemplateCategories()
      ]);
      
      setTemplates(templatesData);
      setCategories(categoriesData);
    } catch (err) {
      console.error('Error loading templates:', err);
      setError('Failed to load DeFi templates. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const loadRecommendations = async () => {
    try {
      setLoading(true);
      const recs = await simpleDefiTemplateService.getTemplateRecommendations(
        riskTolerance,
        capitalAmount,
        experienceLevel
      );
      setRecommendations(recs);
      setShowRecommendations(true);
    } catch (err) {
      console.error('Error loading recommendations:', err);
      setError('Failed to load recommendations. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Filter templates based on selection
  const filteredTemplates = showRecommendations ? recommendations : templates.filter(template => {
    const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;
    const matchesDifficulty = selectedDifficulty === 'all' || template.difficulty.toLowerCase() === selectedDifficulty;
    return matchesCategory && matchesDifficulty;
  });

  const handleCreateStrategy = async (template: DeFiWorkflowTemplate) => {
    try {
      // For now, just select the template. In a full implementation, 
      // you might want to show a capital input dialog here
      onSelectTemplate(template);
    } catch (err) {
      console.error('Error creating strategy:', err);
      setError('Failed to create strategy. Please try again.');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto p-6">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold text-gray-900 mb-2">DeFi Strategy Templates</h1>
        <p className="text-gray-600 text-lg">
          Choose from battle-tested DeFi strategies or create your own
        </p>
      </div>

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800">{error}</p>
          <button 
            onClick={loadTemplatesAndCategories}
            className="mt-2 text-red-600 hover:text-red-800 font-medium"
          >
            Try Again
          </button>
        </div>
      )}

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

      {/* Personalized Recommendations */}
      <div className="mb-8 bg-blue-50 rounded-xl p-6">
        <h3 className="text-xl font-bold text-gray-900 mb-4">Get Personalized Recommendations</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Risk Tolerance (1-10)
            </label>
            <input
              type="range"
              min="1"
              max="10"
              value={riskTolerance}
              onChange={(e) => setRiskTolerance(parseInt(e.target.value))}
              className="w-full"
            />
            <span className="text-sm text-gray-600">{riskTolerance}/10</span>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Capital Amount ($)
            </label>
            <input
              type="number"
              value={capitalAmount}
              onChange={(e) => setCapitalAmount(parseInt(e.target.value) || 0)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              min="50"
              step="50"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Experience Level
            </label>
            <select
              value={experienceLevel}
              onChange={(e) => setExperienceLevel(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="Beginner">Beginner</option>
              <option value="Intermediate">Intermediate</option>
              <option value="Advanced">Advanced</option>
              <option value="Expert">Expert</option>
            </select>
          </div>
        </div>
        <div className="flex gap-3">
          <button
            onClick={loadRecommendations}
            className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Get Recommendations
          </button>
          <button
            onClick={() => {
              setShowRecommendations(false);
              setSelectedCategory('all');
              setSelectedDifficulty('all');
            }}
            className="px-6 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors"
          >
            View All Templates
          </button>
        </div>
      </div>

      {/* Filters */}
      {!showRecommendations && (
        <div className="flex flex-wrap gap-4 mb-6 p-4 bg-gray-50 rounded-xl">
          <div className="flex items-center space-x-2">
            <label className="text-sm font-medium text-gray-700">Category:</label>
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="all">All Categories</option>
              {categories.map(category => (
                <option key={category} value={category}>
                  {category}
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
      )}

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
              className="bg-white rounded-xl border border-gray-200 hover:border-blue-300 hover:shadow-xl transition-all duration-200 cursor-pointer group overflow-hidden"
            >
              {/* Template Header */}
              <div className="p-6 border-b border-gray-100">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center space-x-3">
                    <span className="text-3xl">
                      {simpleDefiTemplateService.getCategoryIcon(template.category)}
                    </span>
                    <div>
                      <h3 className="font-bold text-gray-900 group-hover:text-blue-600 transition-colors">
                        {template.name}
                      </h3>
                      <span className={`inline-block px-2 py-1 text-xs rounded-full ${simpleDefiTemplateService.getDifficultyColor(template.difficulty)}`}>
                        {template.difficulty}
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`inline-block px-2 py-1 text-xs rounded-full ${simpleDefiTemplateService.getRiskColor(template.risk_score)}`}>
                      Risk: {template.risk_score}/10
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
                      {template.estimated_apy.toFixed(1)}%
                    </div>
                    <div className="text-xs text-gray-500">Est. APY</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-blue-600">
                      ${template.min_capital_usd.toLocaleString()}
                    </div>
                    <div className="text-xs text-gray-500">Min Capital</div>
                  </div>
                </div>

                <div className="text-center">
                  <span className={`inline-block px-3 py-1 text-xs rounded-full bg-gray-100 text-gray-700`}>
                    {template.category}
                  </span>
                </div>
              </div>

              {/* Template Actions */}
              <div className="p-6 pt-0">
                <button
                  onClick={() => handleCreateStrategy(template)}
                  className="w-full px-4 py-3 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg hover:from-blue-700 hover:to-purple-700 transition-all duration-200 font-medium group-hover:shadow-md"
                >
                  Create Strategy
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Footer with categories overview */}
      <div className="mt-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {categories.map(category => {
          const categoryTemplates = templates.filter(t => t.category === category);
          const avgAPY = categoryTemplates.length > 0 
            ? (categoryTemplates.reduce((sum, t) => sum + t.estimated_apy, 0) / categoryTemplates.length).toFixed(1)
            : '0.0';
          
          return (
            <div
              key={category}
              onClick={() => {
                setSelectedCategory(category);
                setShowRecommendations(false);
              }}
              className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                selectedCategory === category && !showRecommendations
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
              }`}
            >
              <div className="text-center">
                <div className="text-4xl mb-2">
                  {simpleDefiTemplateService.getCategoryIcon(category)}
                </div>
                <h4 className="font-semibold text-gray-900">{category}</h4>
                <div className="text-sm text-gray-600 mt-1">
                  {categoryTemplates.length} template{categoryTemplates.length !== 1 ? 's' : ''}
                </div>
                <div className="text-sm text-green-600 font-medium">
                  Avg APY: {avgAPY}%
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default DeFiTemplates;