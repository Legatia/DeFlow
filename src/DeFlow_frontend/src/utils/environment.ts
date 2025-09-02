// Environment configuration utility for DeFlow
// Automatically detects and configures for local vs mainnet deployment

export interface DeFlowEnvironment {
  network: 'local' | 'ic';
  host: string;
  identityProvider: string;
  canisterIds: {
    backend: string;
    pool: string;
    frontend: string;
    admin: string;
    internetIdentity: string;
  };
}

class EnvironmentService {
  private _config: DeFlowEnvironment;

  constructor() {
    this._config = this.detectEnvironment();
  }

  private detectEnvironment(): DeFlowEnvironment {
    const network = import.meta.env.VITE_DFX_NETWORK || 'local';
    const isMainnet = network === 'ic';

    return {
      network: isMainnet ? 'ic' : 'local',
      host: isMainnet ? 'https://ic0.app' : 'http://127.0.0.1:4943',
      identityProvider: isMainnet 
        ? 'https://identity.ic0.app'
        : `http://localhost:4943/?canisterId=${import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID || 'rdmx6-jaaaa-aaaaa-aaadq-cai'}`,
      canisterIds: {
        backend: import.meta.env.VITE_CANISTER_ID_DEFLOW_BACKEND || '',
        pool: import.meta.env.VITE_CANISTER_ID_DEFLOW_POOL || '',
        frontend: import.meta.env.VITE_CANISTER_ID_DEFLOW_FRONTEND || '',
        admin: import.meta.env.VITE_CANISTER_ID_DEFLOW_ADMIN || '',
        internetIdentity: import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID || 
          (isMainnet ? 'rdmx6-jaaaa-aaaaa-aaadq-cai' : 'be2us-64aaa-aaaaa-qaabq-cai'),
      }
    };
  }

  get config(): DeFlowEnvironment {
    return this._config;
  }

  get isMainnet(): boolean {
    return this._config.network === 'ic';
  }

  get isLocal(): boolean {
    return this._config.network === 'local';
  }

  get host(): string {
    return this._config.host;
  }

  get identityProvider(): string {
    return this._config.identityProvider;
  }

  get canisterIds() {
    return this._config.canisterIds;
  }

  // Debug info for development
  logConfig(): void {
    if (import.meta.env.DEV) {
      console.log('🔧 DeFlow Environment Config:', this._config);
    }
  }
}

// Export singleton instance
export const environment = new EnvironmentService();
export default environment;