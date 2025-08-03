import { useState } from 'react'
import { NODE_TYPES, NodeCategory } from '../types/nodes'

const CATEGORIES: { id: NodeCategory; name: string; icon: string }[] = [
  { id: 'triggers', name: 'Triggers', icon: '🚀' },
  { id: 'actions', name: 'Actions', icon: '⚡' },
  { id: 'conditions', name: 'Conditions', icon: '🔀' },
  { id: 'transformations', name: 'Transform', icon: '🔄' },
  { id: 'integrations', name: 'Integrations', icon: '🔗' },
  { id: 'utilities', name: 'Utilities', icon: '🛠️' }
]

const NodePalette = () => {
  const [selectedCategory, setSelectedCategory] = useState<NodeCategory>('triggers')
  const [searchTerm, setSearchTerm] = useState('')

  // Filter nodes based on category and search
  const filteredNodes = NODE_TYPES.filter(node => {
    const matchesCategory = node.category === selectedCategory
    const matchesSearch = searchTerm === '' || 
      node.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      node.description.toLowerCase().includes(searchTerm.toLowerCase())
    return matchesCategory && matchesSearch
  })

  const onDragStart = (event: React.DragEvent, nodeTypeId: string) => {
    event.dataTransfer.setData('application/reactflow', nodeTypeId)
    event.dataTransfer.effectAllowed = 'move'
  }

  return (
    <div className="h-full flex flex-col">
      {/* Search */}
      <div className="p-3 border-b border-gray-200">
        <input
          type="text"
          placeholder="Search nodes..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {/* Category Tabs */}
      <div className="border-b border-gray-200">
        <div className="grid grid-cols-2 gap-1 p-2">
          {CATEGORIES.map((category) => {
            const nodeCount = NODE_TYPES.filter(n => n.category === category.id).length
            return (
              <button
                key={category.id}
                onClick={() => setSelectedCategory(category.id)}
                className={`
                  p-2 text-xs rounded-lg text-left transition-colors
                  ${selectedCategory === category.id
                    ? 'bg-blue-100 text-blue-700 border border-blue-200'
                    : 'bg-gray-50 text-gray-600 hover:bg-gray-100 border border-gray-200'
                  }
                `}
              >
                <div className="flex items-center space-x-1">
                  <span>{category.icon}</span>
                  <span className="font-medium">{category.name}</span>
                </div>
                <div className="text-gray-500 mt-1">
                  {nodeCount} node{nodeCount !== 1 ? 's' : ''}
                </div>
              </button>
            )
          })}
        </div>
      </div>

      {/* Node List */}
      <div className="flex-1 overflow-y-auto p-2">
        {filteredNodes.length === 0 ? (
          <div className="text-center text-gray-500 py-8">
            <div className="text-2xl mb-2">🔍</div>
            <div className="text-sm">
              {searchTerm ? `No nodes found for "${searchTerm}"` : 'No nodes in this category'}
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            {filteredNodes.map((nodeType) => (
              <div
                key={nodeType.id}
                draggable
                onDragStart={(event) => onDragStart(event, nodeType.id)}
                className={`
                  p-3 rounded-lg border-2 border-dashed border-gray-300 
                  cursor-grab active:cursor-grabbing
                  hover:border-gray-400 hover:bg-gray-50
                  transition-all duration-200
                  group
                `}
                style={{ 
                  borderLeftColor: nodeType.color,
                  borderLeftWidth: '4px',
                  borderLeftStyle: 'solid'
                }}
              >
                <div className="flex items-start space-x-2">
                  <div 
                    className="w-8 h-8 rounded-lg flex items-center justify-center text-white text-sm font-medium"
                    style={{ backgroundColor: nodeType.color }}
                  >
                    {nodeType.icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm text-gray-900 truncate">
                      {nodeType.name}
                    </div>
                    <div className="text-xs text-gray-600 mt-1 line-clamp-2">
                      {nodeType.description}
                    </div>
                    
                    {/* Connection info */}
                    <div className="flex items-center space-x-3 mt-2 text-xs text-gray-500">
                      <div className="flex items-center space-x-1">
                        <div className="w-2 h-2 rounded-full bg-blue-400"></div>
                        <span>{nodeType.inputs.length} in</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <div className="w-2 h-2 rounded-full bg-green-400"></div>
                        <span>{nodeType.outputs.length} out</span>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Drag indicator */}
                <div className="mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <div className="text-xs text-gray-400 text-center">
                    Drag to canvas
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Tips */}
      <div className="p-3 border-t border-gray-200 bg-blue-50">
        <div className="text-xs text-blue-600">
          <div className="font-medium mb-1">💡 Tips:</div>
          <ul className="space-y-1 text-blue-600">
            <li>• Drag nodes to the canvas</li>
            <li>• Connect nodes by dragging from outputs to inputs</li>
            <li>• Click nodes to configure them</li>
            <li>• Use Delete key to remove selected nodes</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default NodePalette