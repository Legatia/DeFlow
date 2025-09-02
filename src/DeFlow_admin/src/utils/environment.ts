// Environment configuration utility for DeFlow Admin
// Automatically detects and configures for local vs mainnet deployment

export interface DeFlowAdminEnvironment {
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
  ownerPrincipal: string;
  environment: 'development' | 'production';
}

class AdminEnvironmentService {
  private _config: DeFlowAdminEnvironment;

  constructor() {
    this._config = this.detectEnvironment();
  }

  private detectEnvironment(): DeFlowAdminEnvironment {
    const network = process.env.VITE_DFX_NETWORK || process.env.DFX_NETWORK || 'local';
    const isMainnet = network === 'ic';

    return {
      network: isMainnet ? 'ic' : 'local',
      host: isMainnet ? 'https://ic0.app' : 'http://127.0.0.1:8080',
      identityProvider: isMainnet 
        ? 'https://identity.ic0.app'
        : `http://localhost:4943/?canisterId=${process.env.VITE_INTERNET_IDENTITY_CANISTER_ID || 'rdmx6-jaaaa-aaaaa-aaadq-cai'}`,
      canisterIds: {
        backend: process.env.VITE_CANISTER_ID_DEFLOW_BACKEND || '',
        pool: process.env.VITE_CANISTER_ID_DEFLOW_POOL || '',
        frontend: process.env.VITE_CANISTER_ID_DEFLOW_FRONTEND || '',
        admin: process.env.VITE_CANISTER_ID_DEFLOW_ADMIN || '',
        internetIdentity: process.env.VITE_INTERNET_IDENTITY_CANISTER_ID || 
          (isMainnet ? 'rdmx6-jaaaa-aaaaa-aaadq-cai' : 'be2us-64aaa-aaaaa-qaabq-cai'),
      },
      ownerPrincipal: process.env.VITE_OWNER_PRINCIPAL || '',
      environment: isMainnet ? 'production' : 'development',
    };
  }

  get config(): DeFlowAdminEnvironment {
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

  get ownerPrincipal(): string {
    return this._config.ownerPrincipal;
  }

  // Debug info for development
  logConfig(): void {
    if (this._config.environment === 'development') {
      console.log('🔧 DeFlow Admin Environment Config:', this._config);
    }
  }
}

// Export singleton instance
export const adminEnvironment = new AdminEnvironmentService();
export default adminEnvironment;