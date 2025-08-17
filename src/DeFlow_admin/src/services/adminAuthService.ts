interface AdminSession {
  principal: string;
  isOwner: boolean;
  sessionStart: number;
}

export class AdminAuthService {
  private static readonly OWNER_PRINCIPAL = process.env.VITE_OWNER_PRINCIPAL || 'mock-owner-principal';
  private static readonly SESSION_KEY = 'deflow_admin_session';
  private static readonly SESSION_DURATION = 4 * 60 * 60 * 1000; // 4 hours

  /**
   * Create a new admin session after authentication
   */
  static async createSession(principal: string): Promise<AdminSession> {
    // Verify this is the owner principal
    const isOwner = this.isOwnerPrincipal(principal);
    
    if (!isOwner) {
      throw new Error('Access denied. Only the project owner can access the admin dashboard.');
    }

    const session: AdminSession = {
      principal,
      isOwner: true,
      sessionStart: Date.now()
    };

    // Store session in localStorage (in production, consider more secure storage)
    localStorage.setItem(this.SESSION_KEY, JSON.stringify(session));

    return session;
  }

  /**
   * Get current admin session if valid
   */
  static async getCurrentSession(): Promise<AdminSession | null> {
    try {
      const sessionData = localStorage.getItem(this.SESSION_KEY);
      if (!sessionData) return null;

      const session: AdminSession = JSON.parse(sessionData);
      
      // Check if session is expired
      if (Date.now() - session.sessionStart > this.SESSION_DURATION) {
        this.logout();
        return null;
      }

      // Verify principal is still valid owner
      if (!this.isOwnerPrincipal(session.principal)) {
        this.logout();
        return null;
      }

      return session;
    } catch (error) {
      console.error('Error retrieving admin session:', error);
      this.logout();
      return null;
    }
  }

  /**
   * Logout and clear session
   */
  static async logout(): Promise<void> {
    localStorage.removeItem(this.SESSION_KEY);
  }

  /**
   * Check if a principal is the owner
   */
  private static isOwnerPrincipal(principal: string): boolean {
    // For development, accept any principal that starts with 'mock-owner'
    if (principal.startsWith('mock-owner')) {
      return true;
    }
    
    // In production, check against actual owner principal
    return principal === this.OWNER_PRINCIPAL;
  }

  /**
   * Extend current session
   */
  static async extendSession(): Promise<void> {
    const session = await this.getCurrentSession();
    if (session) {
      session.sessionStart = Date.now();
      localStorage.setItem(this.SESSION_KEY, JSON.stringify(session));
    }
  }

  /**
   * Check if current user has owner privileges
   */
  static async isOwner(): Promise<boolean> {
    const session = await this.getCurrentSession();
    return session?.isOwner || false;
  }
}