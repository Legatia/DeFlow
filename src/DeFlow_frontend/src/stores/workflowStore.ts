import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { WorkflowStore, Workflow, WorkflowExecution, NodeDefinition } from '../types'
import { simpleService } from '../services/simpleService'

export const useWorkflowStore = create<WorkflowStore>()(
  devtools(
    (set, get) => ({
      workflows: [],
      currentWorkflow: undefined,
      executions: [],
      nodeDefinitions: [],
      isLoading: false,
      error: undefined,

      loadWorkflows: async () => {
        set({ isLoading: true, error: undefined })
        try {
          const workflows = await simpleService.listWorkflows()
          set({ workflows, isLoading: false })
        } catch (error) {
          console.error('Failed to load workflows:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to load workflows',
            isLoading: false 
          })
        }
      },

      loadWorkflow: async (id: string) => {
        set({ isLoading: true, error: undefined })
        try {
          const workflow = await simpleService.getWorkflow(id)
          set({ currentWorkflow: workflow, isLoading: false })
        } catch (error) {
          console.error('Failed to load workflow:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to load workflow',
            isLoading: false 
          })
        }
      },

      createWorkflow: async (workflowData: Omit<Workflow, 'id' | 'created_at' | 'updated_at'>) => {
        set({ isLoading: true, error: undefined })
        try {
          const workflowId = await simpleService.createWorkflow(workflowData)
          
          // Reload workflows to get the updated list
          await get().loadWorkflows()
          set({ isLoading: false })
          return workflowId
        } catch (error) {
          console.error('Failed to create workflow:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to create workflow',
            isLoading: false 
          })
          throw error
        }
      },

      updateWorkflow: async (workflow: Workflow) => {
        set({ isLoading: true, error: undefined })
        try {
          await simpleService.updateWorkflow(workflow)
          
          // Update local state
          const workflows = get().workflows.map(w => 
            w.id === workflow.id ? workflow : w
          )
          set({ 
            workflows, 
            currentWorkflow: workflow,
            isLoading: false 
          })
        } catch (error) {
          console.error('Failed to update workflow:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to update workflow',
            isLoading: false 
          })
          throw error
        }
      },

      deleteWorkflow: async (id: string) => {
        set({ isLoading: true, error: undefined })
        try {
          await simpleService.deleteWorkflow(id)
          
          // Update local state
          const workflows = get().workflows.filter(w => w.id !== id)
          const currentWorkflow = get().currentWorkflow?.id === id ? undefined : get().currentWorkflow
          
          set({ 
            workflows, 
            currentWorkflow,
            isLoading: false 
          })
        } catch (error) {
          console.error('Failed to delete workflow:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to delete workflow',
            isLoading: false 
          })
          throw error
        }
      },

      executeWorkflow: async (id: string, triggerData?: Record<string, any>) => {
        set({ isLoading: true, error: undefined })
        try {
          const executionId = await simpleService.startExecution(id, triggerData)
          set({ isLoading: false })
          return executionId
        } catch (error) {
          console.error('Failed to execute workflow:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to execute workflow',
            isLoading: false 
          })
          throw error
        }
      },

      loadExecutions: async (workflowId?: string) => {
        set({ isLoading: true, error: undefined })
        try {
          const executions = await simpleService.listExecutions(workflowId)
          set({ executions, isLoading: false })
        } catch (error) {
          console.error('Failed to load executions:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to load executions',
            isLoading: false 
          })
        }
      },

      loadNodeDefinitions: async () => {
        set({ isLoading: true, error: undefined })
        try {
          const nodeDefinitions = await simpleService.listNodeDefinitions()
          set({ nodeDefinitions, isLoading: false })
        } catch (error) {
          console.error('Failed to load node definitions:', error)
          set({ 
            error: error instanceof Error ? error.message : 'Failed to load node definitions',
            isLoading: false 
          })
        }
      },
    }),
    {
      name: 'workflow-store',
    }
  )
)