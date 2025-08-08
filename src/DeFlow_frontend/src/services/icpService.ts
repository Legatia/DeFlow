// ICP Service with proper BigInt handling
import { Actor, HttpAgent } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';
import { Principal } from '@dfinity/principal';
import { Workflow, WorkflowExecution, NodeDefinition } from '../types';
import { BigIntUtils } from '../utils/bigint-utils';

// Import simple BigInt fix before any @dfinity operations
import '../utils/simple-bigint-fix';

// Type-safe interface for our backend canister
interface BackendCanister {
  greet: (name: string) => Promise<string>;
  create_workflow: (workflow: any) => Promise<string>;
  update_workflow: (workflow: any) => Promise<void>;
  get_workflow: (id: string) => Promise<any>;
  list_workflows: () => Promise<any[]>;
  delete_workflow: (id: string) => Promise<void>;
  start_execution: (workflowId: string, triggerData?: any) => Promise<string>;
  get_execution: (id: string) => Promise<any>;
  list_executions: (workflowId?: string) => Promise<any[]>;
  list_node_types: () => Promise<string[]>;
  get_node_definition: (nodeType: string) => Promise<any>;
}

class ICPService {
  private actor: BackendCanister | null = null;
  private authClient: AuthClient | null = null;
  private isInitialized = false;
  private isLocal = true; // Set to false for mainnet
  private initializationPromise: Promise<void> | null = null;

  async initialize(): Promise<void> {
    if (this.isInitialized) return;
    if (this.initializationPromise) return this.initializationPromise;

    this.initializationPromise = this.doInitialize();
    return this.initializationPromise;
  }

  private async doInitialize(): Promise<void> {

    try {
      console.log('Initializing ICP service...');

      // Initialize auth client
      this.authClient = await AuthClient.create();

      // Create agent with error handling
      const agent = new HttpAgent({
        host: this.isLocal ? 'http://localhost:4943' : 'https://ic0.app',
      });

      // Fetch root key for local development
      if (this.isLocal) {
        try {
          await agent.fetchRootKey();
        } catch (error) {
          console.warn('Failed to fetch root key (this is normal in production):', error);
        }
      }

      // Try to get the canister ID and create actor
      try {
        // For now, we'll use a placeholder canister ID
        // In a real app, this would come from dfx deployment
        const canisterId = this.isLocal 
          ? 'rdmx6-jaaaa-aaaaa-aaadq-cai' // Local canister ID
          : 'rdmx6-jaaaa-aaaaa-aaadq-cai'; // Replace with actual mainnet ID

        // Create actor with safe BigInt handling
        this.actor = Actor.createActor<BackendCanister>(
          // IDL interface - for now we'll use a mock interface
          ({ IDL }) => IDL.Service({
            greet: IDL.Func([IDL.Text], [IDL.Text], ['query']),
            create_workflow: IDL.Func([IDL.Record({})], [IDL.Text], []),
            update_workflow: IDL.Func([IDL.Record({})], [], []),
            get_workflow: IDL.Func([IDL.Text], [IDL.Record({})], ['query']),
            list_workflows: IDL.Func([], [IDL.Vec(IDL.Record({}))], ['query']),
            delete_workflow: IDL.Func([IDL.Text], [], []),
            start_execution: IDL.Func([IDL.Text, IDL.Opt(IDL.Record({}))], [IDL.Text], []),
            get_execution: IDL.Func([IDL.Text], [IDL.Record({})], ['query']),
            list_executions: IDL.Func([IDL.Opt(IDL.Text)], [IDL.Vec(IDL.Record({}))], ['query']),
            list_node_types: IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
            get_node_definition: IDL.Func([IDL.Text], [IDL.Record({})], ['query']),
          }),
          {
            agent,
            canisterId: Principal.fromText(canisterId),
          }
        );

        this.isInitialized = true;
        console.log('ICP service initialized successfully');

      } catch (canisterError) {
        console.warn('Canister initialization failed, using mock mode:', canisterError);
        // Fall back to mock mode if canister is not available
        this.actor = this.createMockActor();
        this.isInitialized = true;
      }

    } catch (error) {
      console.error('Failed to initialize ICP service:', error);
      // Use mock actor as fallback
      this.actor = this.createMockActor();
      this.isInitialized = true;
    }
  }

  private createMockActor(): BackendCanister {
    // Mock actor that simulates the real canister interface
    return {
      greet: async (name: string) => `Hello, ${name}! (Mock mode)`,
      
      create_workflow: async (workflow: any) => {
        console.log('Mock: Creating workflow:', workflow);
        return `workflow_${Date.now()}`;
      },
      
      update_workflow: async (workflow: any) => {
        console.log('Mock: Updating workflow:', workflow);
      },
      
      get_workflow: async (id: string) => {
        console.log('Mock: Getting workflow:', id);
        return {
          id,
          name: 'Mock Workflow',
          description: 'This is a mock workflow',
          nodes: [],
          connections: [],
          triggers: [],
          created_at: BigIntUtils.dateToTimestamp(),
          updated_at: BigIntUtils.dateToTimestamp(),
          active: true
        };
      },
      
      list_workflows: async () => {
        console.log('Mock: Listing workflows');
        return [];
      },
      
      delete_workflow: async (id: string) => {
        console.log('Mock: Deleting workflow:', id);
      },
      
      start_execution: async (workflowId: string, triggerData?: any) => {
        console.log('Mock: Starting execution for workflow:', workflowId);
        return `execution_${Date.now()}`;
      },
      
      get_execution: async (id: string) => {
        console.log('Mock: Getting execution:', id);
        return {
          id,
          workflow_id: 'mock_workflow',
          status: 'completed',
          started_at: BigIntUtils.dateToTimestamp(),
          completed_at: BigIntUtils.dateToTimestamp(),
          node_executions: []
        };
      },
      
      list_executions: async (workflowId?: string) => {
        console.log('Mock: Listing executions for workflow:', workflowId);
        return [];
      },
      
      list_node_types: async () => {
        console.log('Mock: Listing node types');
        return ['trigger', 'action', 'condition'];
      },
      
      get_node_definition: async (nodeType: string) => {
        console.log('Mock: Getting node definition for:', nodeType);
        return {
          node_type: nodeType,
          name: `${nodeType} Node`,
          description: `Mock ${nodeType} node`,
          category: 'mock',
          version: '1.0.0',
          input_schema: [],
          output_schema: [],
          configuration_schema: []
        };
      }
    };
  }

  private async ensureInitialized(): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }
    if (!this.actor) {
      throw new Error('ICP service not properly initialized');
    }
  }

  // Convert backend data to frontend types with safe BigInt handling
  private convertWorkflow(backendWorkflow: any): Workflow {
    return {
      ...backendWorkflow,
      created_at: BigIntUtils.toBigInt(backendWorkflow.created_at || Date.now() * 1_000_000),
      updated_at: BigIntUtils.toBigInt(backendWorkflow.updated_at || Date.now() * 1_000_000)
    };
  }

  private convertExecution(backendExecution: any): WorkflowExecution {
    return {
      ...backendExecution,
      started_at: BigIntUtils.toBigInt(backendExecution.started_at || Date.now() * 1_000_000),
      completed_at: backendExecution.completed_at 
        ? BigIntUtils.toBigInt(backendExecution.completed_at)
        : undefined,
      node_executions: backendExecution.node_executions?.map((ne: any) => ({
        ...ne,
        started_at: ne.started_at 
          ? BigIntUtils.toBigInt(ne.started_at)
          : undefined,
        completed_at: ne.completed_at 
          ? BigIntUtils.toBigInt(ne.completed_at)
          : undefined
      })) || []
    };
  }

  // Public API methods
  async greet(name: string): Promise<string> {
    await this.ensureInitialized();
    return this.actor!.greet(name);
  }

  async createWorkflow(workflow: Omit<Workflow, 'id' | 'created_at' | 'updated_at'>): Promise<string> {
    await this.ensureInitialized();
    
    // Prepare workflow data for backend
    const workflowData = {
      ...workflow,
      created_at: BigIntUtils.dateToTimestamp(),
      updated_at: BigIntUtils.dateToTimestamp()
    };
    
    return this.actor!.create_workflow(workflowData);
  }

  async updateWorkflow(workflow: Workflow): Promise<void> {
    await this.ensureInitialized();
    
    const workflowData = {
      ...workflow,
      updated_at: BigIntUtils.dateToTimestamp()
    };
    
    return this.actor!.update_workflow(workflowData);
  }

  async getWorkflow(id: string): Promise<Workflow> {
    await this.ensureInitialized();
    const backendWorkflow = await this.actor!.get_workflow(id);
    return this.convertWorkflow(backendWorkflow);
  }

  async listWorkflows(): Promise<Workflow[]> {
    await this.ensureInitialized();
    const backendWorkflows = await this.actor!.list_workflows();
    return backendWorkflows.map(w => this.convertWorkflow(w));
  }

  async deleteWorkflow(id: string): Promise<void> {
    await this.ensureInitialized();
    return this.actor!.delete_workflow(id);
  }

  async startExecution(workflowId: string, triggerData?: Record<string, any>): Promise<string> {
    await this.ensureInitialized();
    return this.actor!.start_execution(workflowId, triggerData);
  }

  async getExecution(id: string): Promise<WorkflowExecution> {
    await this.ensureInitialized();
    const backendExecution = await this.actor!.get_execution(id);
    return this.convertExecution(backendExecution);
  }

  async listExecutions(workflowId?: string): Promise<WorkflowExecution[]> {
    await this.ensureInitialized();
    const backendExecutions = await this.actor!.list_executions(workflowId);
    return backendExecutions.map(e => this.convertExecution(e));
  }

  async listNodeDefinitions(): Promise<NodeDefinition[]> {
    await this.ensureInitialized();
    const nodeTypes = await this.actor!.list_node_types();
    const definitions: NodeDefinition[] = [];
    
    for (const nodeType of nodeTypes) {
      try {
        const definition = await this.actor!.get_node_definition(nodeType);
        definitions.push(definition);
      } catch (error) {
        console.warn(`Failed to load definition for ${nodeType}:`, error);
      }
    }
    
    return definitions;
  }

  async getNodeDefinition(nodeType: string): Promise<NodeDefinition> {
    await this.ensureInitialized();
    return this.actor!.get_node_definition(nodeType);
  }

  // Authentication methods
  async login(): Promise<boolean> {
    if (!this.authClient) {
      await this.initialize();
    }

    return new Promise((resolve) => {
      this.authClient!.login({
        identityProvider: this.isLocal 
          ? 'http://localhost:4943/?canisterId=rdmx6-jaaaa-aaaaa-aaadq-cai'
          : 'https://identity.ic0.app',
        onSuccess: () => {
          console.log('Login successful');
          resolve(true);
        },
        onError: (error) => {
          console.error('Login failed:', error);
          resolve(false);
        }
      });
    });
  }

  async logout(): Promise<void> {
    if (this.authClient) {
      await this.authClient.logout();
    }
  }

  async isAuthenticated(): Promise<boolean> {
    if (!this.authClient) {
      await this.initialize();
    }
    return this.authClient!.isAuthenticated();
  }

  async getIdentity(): Promise<any> {
    if (!this.authClient) {
      await this.initialize();
    }
    return this.authClient!.getIdentity();
  }
}

export const icpService = new ICPService();