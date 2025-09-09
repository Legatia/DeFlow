import React, { useState } from 'react'
import { WorkflowBlockButton } from './WorkflowBlockButton'
import { LiquidGlassButton } from './ui/liquid-glass-button'
import { cn } from '@/lib/utils'
import { getAllNodeTypes, NodeType } from '../types/all-nodes'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import { canAccessNodeType, canDragNode } from '../utils/subscriptionUtils'

// Map node categories to block types for consistency
const getCategoryType = (category: string): 'defi' | 'social' | 'trigger' | 'action' | 'condition' | 'custom' => {
  switch (category) {
    case 'triggers':
      return 'trigger'
    case 'actions':
      return 'action'
    case 'conditions':
      return 'condition'
    case 'integrations':
      return 'social'
    case 'transformations':
    case 'utilities':
    default:
      return 'custom'
  }
}

// Get icon for node category
const getCategoryIcon = (category: string) => {
  switch (category) {
    case 'triggers':
      return '🚀'
    case 'actions':
      return '⚡'
    case 'conditions':
      return '🔀'
    case 'transformations':
      return '🔄'
    case 'integrations':
      return '🔗'
    case 'utilities':
      return '🛠️'
    default:
      return '⚙️'
  }
}

export const EnhancedNodePalette: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState<string>('all')
  const [searchTerm, setSearchTerm] = useState('')
  const [draggedItem, setDraggedItem] = useState<string | null>(null)
  
  const { subscriptionTier } = useEnhancedAuth()
  const allNodeTypes = getAllNodeTypes()

  // Create categories from actual node types
  const categories = [
    { id: 'all', label: 'All Nodes', count: allNodeTypes.length },
    { id: 'triggers', label: 'Triggers', count: allNodeTypes.filter(n => n.category === 'triggers').length },
    { id: 'actions', label: 'Actions', count: allNodeTypes.filter(n => n.category === 'actions').length },
    { id: 'conditions', label: 'Conditions', count: allNodeTypes.filter(n => n.category === 'conditions').length },
    { id: 'integrations', label: 'Integrations', count: allNodeTypes.filter(n => n.category === 'integrations').length },
    { id: 'transformations', label: 'Transform', count: allNodeTypes.filter(n => n.category === 'transformations').length },
    { id: 'utilities', label: 'Utilities', count: allNodeTypes.filter(n => n.category === 'utilities').length }
  ]

  // Filter nodes based on category and search
  const filteredNodes = allNodeTypes.filter(node => {
    const matchesCategory = activeCategory === 'all' || node.category === activeCategory
    const matchesSearch = searchTerm === '' || 
      node.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      node.description.toLowerCase().includes(searchTerm.toLowerCase())
    return matchesCategory && matchesSearch
  })

  const handleDragStart = (event: React.DragEvent, nodeType: NodeType) => {
    if (!canDragNode(subscriptionTier, nodeType)) {
      event.preventDefault()
      return
    }
    setDraggedItem(nodeType.id)
    event.dataTransfer.setData('application/reactflow', nodeType.id)
    event.dataTransfer.effectAllowed = 'move'
  }

  const handleDragEnd = () => {
    setDraggedItem(null)
  }

  return (
    <div className="w-80 h-full bg-gray-900/80 backdrop-blur-lg border-r border-gray-700 flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <h2 className="text-lg font-semibold text-white mb-4">Workflow Nodes</h2>
        
        {/* Search */}
        <div className="mb-4">
          <input
            type="text"
            placeholder="Search nodes..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-3 py-2 bg-gray-800/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent backdrop-blur-sm"
          />
        </div>
        
        {/* Category Tabs */}
        <div className="flex flex-wrap gap-2">
          {categories.map(category => (
            <LiquidGlassButton
              key={category.id}
              size="sm"
              variant={activeCategory === category.id ? 'primary' : 'default'}
              onClick={() => setActiveCategory(category.id)}
              className="text-xs"
            >
              <span className="mr-1">{getCategoryIcon(category.id)}</span>
              {category.label}
              <span className="ml-1 text-xs opacity-60">({category.count})</span>
            </LiquidGlassButton>
          ))}
        </div>
      </div>

      {/* Nodes Grid */}
      <div className="flex-1 p-4 overflow-y-auto">
        {filteredNodes.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            <div className="text-2xl mb-2">🔍</div>
            <div className="text-sm">
              {searchTerm ? `No nodes found for "${searchTerm}"` : 'No nodes in this category'}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredNodes.map((nodeType, index) => {
              const hasAccess = canAccessNodeType(subscriptionTier, nodeType)
              const canDrag = canDragNode(subscriptionTier, nodeType)
              
              return (
                <WorkflowBlockButton
                  key={`${nodeType.id}-${index}`}
                  type={getCategoryType(nodeType.category)}
                  title={nodeType.name}
                  description={nodeType.description}
                  icon={<span className="text-lg">{nodeType.icon}</span>}
                  isDragging={draggedItem === nodeType.id}
                  onDragStart={canDrag ? (event) => handleDragStart(event, nodeType) : undefined}
                  onDragEnd={handleDragEnd}
                  className={cn(
                    "animate-in fade-in-0 slide-in-from-left-2 transition-all duration-200",
                    !hasAccess && "opacity-60 grayscale",
                    !canDrag && "cursor-not-allowed"
                  )}
                  style={{ animationDelay: `${index * 50}ms` }}
                />
              )
            })}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-gray-700 text-center">
        <p className="text-xs text-gray-400">
          Drag nodes to create workflows
        </p>
        {filteredNodes.some(n => !canAccessNodeType(subscriptionTier, n)) && (
          <p className="text-xs text-amber-400 mt-1">
            🔒 Some nodes require subscription upgrade
          </p>
        )}
      </div>
    </div>
  )
}