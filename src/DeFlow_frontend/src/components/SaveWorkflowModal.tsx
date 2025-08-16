import React, { useState } from 'react'
import { Workflow } from '../types/index'

interface SaveWorkflowModalProps {
  type: 'draft' | 'publish' | 'template'
  onSave: (name: string, category?: string, description?: string) => void
  onCancel: () => void
  currentWorkflow?: Partial<Workflow>
}

const SaveWorkflowModal: React.FC<SaveWorkflowModalProps> = ({
  type,
  onSave,
  onCancel,
  currentWorkflow
}) => {
  const [name, setName] = useState(currentWorkflow?.name || '')
  const [category, setCategory] = useState(currentWorkflow?.metadata?.templateCategory || 'automation')
  const [description, setDescription] = useState(currentWorkflow?.metadata?.templateDescription || '')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) return
    
    if (type === 'template') {
      onSave(name.trim(), category, description.trim())
    } else {
      onSave(name.trim())
    }
  }

  const getModalTitle = () => {
    switch (type) {
      case 'draft': return 'Save as Draft'
      case 'publish': return 'Publish Workflow'
      case 'template': return 'Save as Template'
    }
  }

  const getModalDescription = () => {
    switch (type) {
      case 'draft': return 'Save your work-in-progress. You can continue editing it later.'
      case 'publish': return 'Make your workflow active and ready for execution.'
      case 'template': return 'Create a reusable template that others can use to build similar workflows.'
    }
  }

  const templateCategories = [
    { value: 'automation', label: 'Automation' },
    { value: 'defi', label: 'DeFi Trading' },
    { value: 'social', label: 'Social Media' },
    { value: 'notification', label: 'Notifications' },
    { value: 'data', label: 'Data Processing' },
    { value: 'integration', label: 'Integrations' },
    { value: 'other', label: 'Other' }
  ]

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
        <h2 className="text-xl font-semibold text-gray-900 mb-2">
          {getModalTitle()}
        </h2>
        <p className="text-sm text-gray-600 mb-4">
          {getModalDescription()}
        </p>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Workflow Name */}
          <div>
            <label htmlFor="workflow-name" className="block text-sm font-medium text-gray-700 mb-1">
              {type === 'template' ? 'Template Name' : 'Workflow Name'}
            </label>
            <input
              id="workflow-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={type === 'template' ? 'My Automation Template' : 'My Workflow'}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              required
            />
          </div>

          {/* Template-specific fields */}
          {type === 'template' && (
            <>
              <div>
                <label htmlFor="template-category" className="block text-sm font-medium text-gray-700 mb-1">
                  Category
                </label>
                <select
                  id="template-category"
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                >
                  {templateCategories.map((cat) => (
                    <option key={cat.value} value={cat.value}>
                      {cat.label}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label htmlFor="template-description" className="block text-sm font-medium text-gray-700 mb-1">
                  Description
                </label>
                <textarea
                  id="template-description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Describe what this template does and how to use it..."
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </>
          )}

          {/* Status indicators */}
          <div className="bg-gray-50 rounded-lg p-3">
            <div className="flex items-center text-sm text-gray-600">
              <span className="mr-2">
                {type === 'draft' && '📝'}
                {type === 'publish' && '🚀'}
                {type === 'template' && '📋'}
              </span>
              <span>
                {type === 'draft' && 'Will be saved to your drafts'}
                {type === 'publish' && 'Will be activated for execution'}
                {type === 'template' && 'Will be available in template library'}
              </span>
            </div>
          </div>

          {/* Buttons */}
          <div className="flex space-x-3 pt-2">
            <button
              type="button"
              onClick={onCancel}
              className="flex-1 px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!name.trim()}
              className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {type === 'draft' && 'Save Draft'}
              {type === 'publish' && 'Publish'}
              {type === 'template' && 'Create Template'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default SaveWorkflowModal