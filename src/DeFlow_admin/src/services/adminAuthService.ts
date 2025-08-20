interface AdminSession {
  principal: string;
  isOwner: boolean;
  sessionStart: number;
}

export class AdminAuthService {
  private static readonly OWNER_PRINCIPAL = process.env.VITE_OWNER_PRINCIPAL;
  private static readonly SESSION_KEY = 'deflow_admin_session';
  private static readonly SESSION_DURATION = 4 * 60 * 60 * 1000; // 4 hours

  /**
   * Create a new admin session after authentication
   */
  static async createSession(principal: string): Promise<AdminSession> {
    // SECURITY: Validate owner principal is configured
    if (!this.OWNER_PRINCIPAL) {
      throw new Error('SECURITY: Owner principal not configured. Set VITE_OWNER_PRINCIPAL environment variable.');
    }

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

    // SECURITY: Store encrypted session data
    try {
      const encryptedSession = btoa(JSON.stringify(session)); // Basic encoding (use proper encryption in production)
      sessionStorage.setItem(this.SESSION_KEY, encryptedSession); // Use sessionStorage instead of localStorage
    } catch (error) {
      throw new Error('Failed to create secure session');
    }

    return session;
  }

  /**
   * Get current admin session if valid
   */
  static async getCurrentSession(): Promise<AdminSession | null> {
    try {
      const encryptedSessionData = sessionStorage.getItem(this.SESSION_KEY);
      if (!encryptedSessionData) return null;

      // SECURITY: Decrypt session data
      const sessionData = atob(encryptedSessionData);
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
      console.error('SECURITY: Invalid session data detected, clearing session');
      this.logout();
      return null;
    }
  }

  /**
   * Logout and clear session
   */
  static async logout(): Promise<void> {
    sessionStorage.removeItem(this.SESSION_KEY);
    // Clear any other sensitive data
    sessionStorage.clear();
  }

  /**
   * Check if a principal is the owner
   */
  private static isOwnerPrincipal(principal: string): boolean {
    // SECURITY: No mock bypasses in production
    if (!this.OWNER_PRINCIPAL) {
      console.error('SECURITY: Owner principal not configured');
      return false;
    }
    
    // SECURITY: Strict principal matching only
    if (principal !== this.OWNER_PRINCIPAL) {
      console.warn('SECURITY: Unauthorized principal access attempt:', principal);
      return false;
    }
    
    return true;
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