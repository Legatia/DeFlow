interface AdminSession {
  principal: string;
  isOwner: boolean;
  isTeamMember: boolean;
  sessionStart: number;
}

interface TeamMember {
  principal: string;
  addedBy: string;
  addedAt: number;
  role: 'admin' | 'member';
  status: 'active' | 'inactive';
  earningPercentage: number; // Percentage of earnings (0-100)
}

interface PendingApproval {
  id: string;
  candidatePrincipal: string;
  requestedBy: string;
  requestedAt: number;
  role: 'admin' | 'member';
  status: 'pending' | 'approved' | 'rejected';
}

export class AdminAuthService {
  private static readonly OWNER_PRINCIPAL = process.env.VITE_OWNER_PRINCIPAL;
  private static readonly SESSION_KEY = 'deflow_admin_session';
  private static readonly TEAM_MEMBERS_KEY = 'deflow_team_members';
  private static readonly PENDING_APPROVALS_KEY = 'deflow_pending_approvals';
  private static readonly SESSION_DURATION = 4 * 60 * 60 * 1000; // 4 hours

  /**
   * Create a new admin session after authentication
   */
  static async createSession(principal: string): Promise<AdminSession> {
    // INITIAL SETUP MODE: If no owner is configured, this becomes the setup flow
    if (!this.OWNER_PRINCIPAL || this.OWNER_PRINCIPAL.trim() === '') {
      console.log('SETUP MODE: No owner configured yet, allowing initial setup for principal:', principal);
      const session: AdminSession = {
        principal,
        isOwner: false, // Will become true after owner is set
        isTeamMember: false,
        sessionStart: Date.now()
      };

      // Store session for initial setup
      try {
        const encryptedSession = btoa(JSON.stringify(session));
        sessionStorage.setItem(this.SESSION_KEY, encryptedSession);
      } catch (error) {
        throw new Error('Failed to create setup session');
      }

      return session;
    }

    // NORMAL MODE: Owner is configured, verify access
    const isOwner = this.isOwnerPrincipal(principal);
    const isTeamMember = !isOwner ? this.isTeamMember(principal) : false;
    
    if (!isOwner && !isTeamMember) {
      throw new Error('Access denied. Only the project owner and approved team members can access the admin dashboard.');
    }

    const session: AdminSession = {
      principal,
      isOwner,
      isTeamMember,
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
    // SETUP MODE: If no owner configured, return false (setup mode)
    if (!this.OWNER_PRINCIPAL || this.OWNER_PRINCIPAL.trim() === '') {
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

  /**
   * Check if system is in setup mode (no owner configured yet)
   */
  static isSetupMode(): boolean {
    return !this.OWNER_PRINCIPAL || this.OWNER_PRINCIPAL.trim() === '';
  }

  /**
   * Get current user's principal (for setup/whoami)
   */
  static async getCurrentPrincipal(): Promise<string | null> {
    const session = await this.getCurrentSession();
    return session?.principal || null;
  }

  /**
   * Check if a principal is a team member
   */
  private static isTeamMember(principal: string): boolean {
    const teamMembers = this.getTeamMembers();
    return teamMembers.some(member => member.principal === principal && member.status === 'active');
  }

  /**
   * Get all team members
   */
  static getTeamMembers(): TeamMember[] {
    try {
      const data = localStorage.getItem(this.TEAM_MEMBERS_KEY);
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  }

  /**
   * Add a new team member (requires owner approval)
   */
  static async requestTeamMemberAddition(candidatePrincipal: string, role: 'admin' | 'member' = 'member'): Promise<void> {
    const session = await this.getCurrentSession();
    if (!session) throw new Error('No active session');

    // Validate principal format
    if (!candidatePrincipal || candidatePrincipal.length < 20) {
      throw new Error('Invalid principal format');
    }

    // Check if already a member
    if (this.isTeamMember(candidatePrincipal) || this.isOwnerPrincipal(candidatePrincipal)) {
      throw new Error('Principal is already a team member or owner');
    }

    // Check if already has pending request
    const pending = this.getPendingApprovals();
    if (pending.some(p => p.candidatePrincipal === candidatePrincipal && p.status === 'pending')) {
      throw new Error('There is already a pending request for this principal');
    }

    const approval: PendingApproval = {
      id: Date.now().toString(),
      candidatePrincipal,
      requestedBy: session.principal,
      requestedAt: Date.now(),
      role,
      status: 'pending'
    };

    const approvals = [...pending, approval];
    localStorage.setItem(this.PENDING_APPROVALS_KEY, JSON.stringify(approvals));
  }

  /**
   * Get pending approvals (owner only)
   */
  static getPendingApprovals(): PendingApproval[] {
    try {
      const data = localStorage.getItem(this.PENDING_APPROVALS_KEY);
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  }

  /**
   * Approve team member addition (owner only)
   */
  static async approveTeamMember(approvalId: string): Promise<void> {
    const session = await this.getCurrentSession();
    if (!session?.isOwner) {
      throw new Error('Only the owner can approve team members');
    }

    const approvals = this.getPendingApprovals();
    const approval = approvals.find(a => a.id === approvalId && a.status === 'pending');
    
    if (!approval) {
      throw new Error('Approval request not found or already processed');
    }

    // Add to team members
    const teamMembers = this.getTeamMembers();
    const newMember: TeamMember = {
      principal: approval.candidatePrincipal,
      addedBy: session.principal,
      addedAt: Date.now(),
      role: approval.role,
      status: 'active',
      earningPercentage: 0 // Default to 0%, owner will set custom percentage
    };

    teamMembers.push(newMember);
    localStorage.setItem(this.TEAM_MEMBERS_KEY, JSON.stringify(teamMembers));

    // Update approval status
    approval.status = 'approved';
    localStorage.setItem(this.PENDING_APPROVALS_KEY, JSON.stringify(approvals));
  }

  /**
   * Reject team member addition (owner only)
   */
  static async rejectTeamMember(approvalId: string): Promise<void> {
    const session = await this.getCurrentSession();
    if (!session?.isOwner) {
      throw new Error('Only the owner can reject team members');
    }

    const approvals = this.getPendingApprovals();
    const approval = approvals.find(a => a.id === approvalId && a.status === 'pending');
    
    if (!approval) {
      throw new Error('Approval request not found or already processed');
    }

    approval.status = 'rejected';
    localStorage.setItem(this.PENDING_APPROVALS_KEY, JSON.stringify(approvals));
  }

  /**
   * Remove team member (owner only)
   */
  static async removeTeamMember(principal: string): Promise<void> {
    const session = await this.getCurrentSession();
    if (!session?.isOwner) {
      throw new Error('Only the owner can remove team members');
    }

    const teamMembers = this.getTeamMembers();
    const updatedMembers = teamMembers.filter(member => member.principal !== principal);
    localStorage.setItem(this.TEAM_MEMBERS_KEY, JSON.stringify(updatedMembers));
  }

  /**
   * Update team member earning percentage (owner only)
   */
  static async updateTeamMemberEarning(principal: string, earningPercentage: number): Promise<void> {
    const session = await this.getCurrentSession();
    if (!session?.isOwner) {
      throw new Error('Only the owner can update earning percentages');
    }

    if (earningPercentage < 0 || earningPercentage > 100) {
      throw new Error('Earning percentage must be between 0 and 100');
    }

    const teamMembers = this.getTeamMembers();
    const memberIndex = teamMembers.findIndex(member => member.principal === principal);
    
    if (memberIndex === -1) {
      throw new Error('Team member not found');
    }

    teamMembers[memberIndex].earningPercentage = earningPercentage;
    localStorage.setItem(this.TEAM_MEMBERS_KEY, JSON.stringify(teamMembers));
  }

  /**
   * Get total earning percentage allocated to team members
   */
  static getTotalEarningPercentage(): number {
    const teamMembers = this.getTeamMembers();
    return teamMembers
      .filter(member => member.status === 'active')
      .reduce((total, member) => total + member.earningPercentage, 0);
  }
}