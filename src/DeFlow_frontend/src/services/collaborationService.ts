// Workflow sharing and collaboration service for DeFlow
import { Workflow } from '../types'
import { User } from './authService'
import { TimestampUtils } from '../utils/timestamp-utils'

export interface SharedWorkflow {
  id: string
  workflowId: string
  ownerId: string
  permissions: WorkflowPermissions
  shareSettings: ShareSettings
  invitations: WorkflowInvitation[]
  collaborators: WorkflowCollaborator[]
  createdAt: string
  updatedAt: string
  isActive: boolean
}

export interface WorkflowPermissions {
  view: boolean
  edit: boolean
  execute: boolean
  share: boolean
  delete: boolean
  comment: boolean
  analytics: boolean
}

export interface ShareSettings {
  visibility: 'private' | 'team' | 'organization' | 'public'
  allowComments: boolean
  allowFork: boolean
  allowExport: boolean
  expiresAt?: string
  accessCode?: string
  domains?: string[] // Email domain restrictions
}

export interface WorkflowInvitation {
  id: string
  sharedWorkflowId: string
  invitedBy: string
  invitedEmail: string
  permissions: WorkflowPermissions
  message?: string
  status: 'pending' | 'accepted' | 'declined' | 'expired'
  invitedAt: string
  respondedAt?: string
  expiresAt: string
}

export interface WorkflowCollaborator {
  id: string
  userId: string
  sharedWorkflowId: string
  permissions: WorkflowPermissions
  addedBy: string
  addedAt: string
  lastAccessedAt?: string
  isActive: boolean
  role: 'owner' | 'editor' | 'viewer' | 'commenter'
}

export interface WorkflowComment {
  id: string
  workflowId: string
  userId: string
  content: string
  type: 'general' | 'node' | 'connection' | 'suggestion'
  nodeId?: string
  connectionId?: string
  parentCommentId?: string
  status: 'active' | 'resolved' | 'deleted'
  createdAt: string
  updatedAt?: string
  reactions: CommentReaction[]
  attachments?: CommentAttachment[]
}

export interface CommentReaction {
  userId: string
  emoji: string
  createdAt: string
}

export interface CommentAttachment {
  id: string
  name: string
  type: string
  size: number
  url: string
}

export interface WorkflowActivity {
  id: string
  workflowId: string
  userId: string
  type: 'edit' | 'comment' | 'execute' | 'share' | 'fork' | 'export'
  description: string
  details: Record<string, any>
  timestamp: string
}

export interface WorkflowFork {
  id: string
  originalWorkflowId: string
  forkedWorkflowId: string
  userId: string
  name: string
  description?: string
  visibility: 'private' | 'public'
  createdAt: string
  syncStatus: 'up-to-date' | 'behind' | 'diverged'
  lastSyncAt?: string
}

export interface Team {
  id: string
  name: string
  description?: string
  ownerId: string
  members: TeamMember[]
  permissions: TeamPermissions
  settings: TeamSettings
  createdAt: string
  updatedAt: string
}

export interface TeamMember {
  userId: string
  role: 'admin' | 'member' | 'viewer'
  joinedAt: string
  permissions: WorkflowPermissions
}

export interface TeamPermissions {
  defaultWorkflowPermissions: WorkflowPermissions
  canInviteMembers: boolean
  canCreateWorkflows: boolean
  canManageTeam: boolean
}

export interface TeamSettings {
  visibility: 'private' | 'organization' | 'public'
  allowExternalSharing: boolean
  requireApprovalForSharing: boolean
  defaultShareExpiration: number // days
}

class CollaborationService {
  private sharedWorkflows: Map<string, SharedWorkflow> = new Map()
  private comments: Map<string, WorkflowComment> = new Map()
  private activities: Map<string, WorkflowActivity[]> = new Map()
  private forks: Map<string, WorkflowFork> = new Map()
  private teams: Map<string, Team> = new Map()
  private invitations: Map<string, WorkflowInvitation> = new Map()

  constructor() {
    this.initializeDemoData()
  }

  // Workflow sharing
  shareWorkflow(
    workflowId: string,
    ownerId: string,
    settings: ShareSettings,
    permissions: WorkflowPermissions
  ): SharedWorkflow {
    const sharedWorkflow: SharedWorkflow = {
      id: this.generateId('share'),
      workflowId,
      ownerId,
      permissions,
      shareSettings: settings,
      invitations: [],
      collaborators: [{
        id: this.generateId('collab'),
        userId: ownerId,
        sharedWorkflowId: '',
        permissions: { ...permissions, delete: true, share: true }, // Owner has all permissions
        addedBy: ownerId,
        addedAt: TimestampUtils.dateToICPTimestamp(new Date()),
        isActive: true,
        role: 'owner'
      }],
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      updatedAt: TimestampUtils.dateToICPTimestamp(new Date()),
      isActive: true
    }

    sharedWorkflow.collaborators[0].sharedWorkflowId = sharedWorkflow.id
    this.sharedWorkflows.set(sharedWorkflow.id, sharedWorkflow)
    
    this.logActivity(workflowId, ownerId, 'share', 'Workflow shared', { 
      visibility: settings.visibility,
      permissions 
    })

    return sharedWorkflow
  }

  updateShareSettings(shareId: string, userId: string, settings: Partial<ShareSettings>): SharedWorkflow {
    const shared = this.sharedWorkflows.get(shareId)
    if (!shared) {
      throw new Error('Shared workflow not found')
    }

    if (!this.canManageSharing(shared, userId)) {
      throw new Error('Insufficient permissions to update share settings')
    }

    shared.shareSettings = { ...shared.shareSettings, ...settings }
    shared.updatedAt = TimestampUtils.dateToICPTimestamp(new Date())
    this.sharedWorkflows.set(shareId, shared)

    this.logActivity(shared.workflowId, userId, 'edit', 'Share settings updated', settings)

    return shared
  }

  stopSharing(shareId: string, userId: string): boolean {
    const shared = this.sharedWorkflows.get(shareId)
    if (!shared) {
      throw new Error('Shared workflow not found')
    }

    if (!this.canManageSharing(shared, userId)) {
      throw new Error('Insufficient permissions to stop sharing')
    }

    shared.isActive = false
    shared.updatedAt = TimestampUtils.dateToICPTimestamp(new Date())
    this.sharedWorkflows.set(shareId, shared)

    this.logActivity(shared.workflowId, userId, 'edit', 'Sharing stopped', {})

    return true
  }

  // Invitations
  inviteCollaborators(
    shareId: string,
    inviterId: string,
    emails: string[],
    permissions: WorkflowPermissions,
    message?: string
  ): WorkflowInvitation[] {
    const shared = this.sharedWorkflows.get(shareId)
    if (!shared) {
      throw new Error('Shared workflow not found')
    }

    if (!this.canInviteCollaborators(shared, inviterId)) {
      throw new Error('Insufficient permissions to invite collaborators')
    }

    const invitations: WorkflowInvitation[] = []
    const expiresAt = new Date()
    expiresAt.setDate(expiresAt.getDate() + 7) // 7 days expiration

    for (const email of emails) {
      // Check if already invited or is collaborator
      const existingInvitation = shared.invitations.find(i => 
        i.invitedEmail === email && i.status === 'pending'
      )
      if (existingInvitation) continue

      const existingCollaborator = shared.collaborators.find(c => {
        // In real implementation, would look up user by email
        return false // Simplified check
      })
      if (existingCollaborator) continue

      const invitation: WorkflowInvitation = {
        id: this.generateId('invite'),
        sharedWorkflowId: shareId,
        invitedBy: inviterId,
        invitedEmail: email,
        permissions,
        message,
        status: 'pending',
        invitedAt: TimestampUtils.dateToICPTimestamp(new Date()),
        expiresAt: TimestampUtils.dateToICPTimestamp(expiresAt)
      }

      invitations.push(invitation)
      shared.invitations.push(invitation)
      this.invitations.set(invitation.id, invitation)
    }

    this.sharedWorkflows.set(shareId, shared)
    
    this.logActivity(shared.workflowId, inviterId, 'share', 
      `Invited ${invitations.length} collaborators`, { emails })

    // In real implementation, send email invitations
    this.sendInvitationEmails(invitations)

    return invitations
  }

  respondToInvitation(invitationId: string, userId: string, response: 'accept' | 'decline'): WorkflowCollaborator | null {
    const invitation = this.invitations.get(invitationId)
    if (!invitation) {
      throw new Error('Invitation not found')
    }

    if (invitation.status !== 'pending') {
      throw new Error('Invitation already responded to')
    }

    const expiresAt = TimestampUtils.icpTimestampToDate(invitation.expiresAt)
    if (expiresAt <= new Date()) {
      invitation.status = 'expired'
      this.invitations.set(invitationId, invitation)
      throw new Error('Invitation has expired')
    }

    invitation.status = response === 'accept' ? 'accepted' : 'declined'
    invitation.respondedAt = TimestampUtils.dateToICPTimestamp(new Date())
    this.invitations.set(invitationId, invitation)

    if (response === 'accept') {
      const shared = this.sharedWorkflows.get(invitation.sharedWorkflowId)
      if (shared) {
        const collaborator: WorkflowCollaborator = {
          id: this.generateId('collab'),
          userId,
          sharedWorkflowId: invitation.sharedWorkflowId,
          permissions: invitation.permissions,
          addedBy: invitation.invitedBy,
          addedAt: TimestampUtils.dateToICPTimestamp(new Date()),
          isActive: true,
          role: this.getCollaboratorRole(invitation.permissions)
        }

        shared.collaborators.push(collaborator)
        this.sharedWorkflows.set(shared.id, shared)

        this.logActivity(shared.workflowId, userId, 'share', 'Joined as collaborator', {})

        return collaborator
      }
    }

    return null
  }

  // Collaborator management
  updateCollaboratorPermissions(
    shareId: string,
    collaboratorId: string,
    updaterId: string,
    permissions: WorkflowPermissions
  ): WorkflowCollaborator {
    const shared = this.sharedWorkflows.get(shareId)
    if (!shared) {
      throw new Error('Shared workflow not found')
    }

    if (!this.canManageCollaborators(shared, updaterId)) {
      throw new Error('Insufficient permissions to update collaborator permissions')
    }

    const collaborator = shared.collaborators.find(c => c.id === collaboratorId)
    if (!collaborator) {
      throw new Error('Collaborator not found')
    }

    collaborator.permissions = permissions
    collaborator.role = this.getCollaboratorRole(permissions)
    this.sharedWorkflows.set(shareId, shared)

    this.logActivity(shared.workflowId, updaterId, 'edit', 
      'Updated collaborator permissions', { collaboratorId, permissions })

    return collaborator
  }

  removeCollaborator(shareId: string, collaboratorId: string, removerId: string): boolean {
    const shared = this.sharedWorkflows.get(shareId)
    if (!shared) {
      throw new Error('Shared workflow not found')
    }

    if (!this.canManageCollaborators(shared, removerId)) {
      throw new Error('Insufficient permissions to remove collaborator')
    }

    const index = shared.collaborators.findIndex(c => c.id === collaboratorId)
    if (index === -1) {
      throw new Error('Collaborator not found')
    }

    const collaborator = shared.collaborators[index]
    if (collaborator.role === 'owner' && collaborator.userId !== removerId) {
      throw new Error('Cannot remove workflow owner')
    }

    shared.collaborators.splice(index, 1)
    this.sharedWorkflows.set(shareId, shared)

    this.logActivity(shared.workflowId, removerId, 'edit', 
      'Removed collaborator', { collaboratorId })

    return true
  }

  // Comments system
  addComment(
    workflowId: string,
    userId: string,
    content: string,
    type: WorkflowComment['type'] = 'general',
    nodeId?: string,
    connectionId?: string,
    parentCommentId?: string
  ): WorkflowComment {
    if (!this.canComment(workflowId, userId)) {
      throw new Error('Insufficient permissions to comment')
    }

    const comment: WorkflowComment = {
      id: this.generateId('comment'),
      workflowId,
      userId,
      content,
      type,
      nodeId,
      connectionId,
      parentCommentId,
      status: 'active',
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      reactions: []
    }

    this.comments.set(comment.id, comment)
    
    this.logActivity(workflowId, userId, 'comment', 'Added comment', { 
      type, nodeId, parentCommentId 
    })

    return comment
  }

  updateComment(commentId: string, userId: string, content: string): WorkflowComment {
    const comment = this.comments.get(commentId)
    if (!comment) {
      throw new Error('Comment not found')
    }

    if (comment.userId !== userId) {
      throw new Error('Can only edit your own comments')
    }

    comment.content = content
    comment.updatedAt = TimestampUtils.dateToICPTimestamp(new Date())
    this.comments.set(commentId, comment)

    this.logActivity(comment.workflowId, userId, 'edit', 'Updated comment', { commentId })

    return comment
  }

  deleteComment(commentId: string, userId: string): boolean {
    const comment = this.comments.get(commentId)
    if (!comment) {
      throw new Error('Comment not found')
    }

    if (comment.userId !== userId && !this.canModerateComments(comment.workflowId, userId)) {
      throw new Error('Insufficient permissions to delete comment')
    }

    comment.status = 'deleted'
    this.comments.set(commentId, comment)

    this.logActivity(comment.workflowId, userId, 'edit', 'Deleted comment', { commentId })

    return true
  }

  addCommentReaction(commentId: string, userId: string, emoji: string): WorkflowComment {
    const comment = this.comments.get(commentId)
    if (!comment) {
      throw new Error('Comment not found')
    }

    // Remove existing reaction from this user
    comment.reactions = comment.reactions.filter(r => r.userId !== userId)

    // Add new reaction
    comment.reactions.push({
      userId,
      emoji,
      createdAt: TimestampUtils.dateToICPTimestamp(new Date())
    })

    this.comments.set(commentId, comment)
    return comment
  }

  getWorkflowComments(workflowId: string, userId: string): WorkflowComment[] {
    if (!this.canViewComments(workflowId, userId)) {
      throw new Error('Insufficient permissions to view comments')
    }

    return Array.from(this.comments.values())
      .filter(c => c.workflowId === workflowId && c.status === 'active')
      .sort((a, b) => a.createdAt.localeCompare(b.createdAt))
  }

  // Workflow forking
  forkWorkflow(
    originalWorkflowId: string,
    userId: string,
    name: string,
    description?: string,
    visibility: 'private' | 'public' = 'private'
  ): WorkflowFork {
    // Check if user can fork this workflow
    if (!this.canForkWorkflow(originalWorkflowId, userId)) {
      throw new Error('Insufficient permissions to fork workflow')
    }

    const forkedWorkflowId = this.generateId('workflow') // In real implementation, would create actual workflow

    const fork: WorkflowFork = {
      id: this.generateId('fork'),
      originalWorkflowId,
      forkedWorkflowId,
      userId,
      name,
      description,
      visibility,
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      syncStatus: 'up-to-date'
    }

    this.forks.set(fork.id, fork)
    
    this.logActivity(originalWorkflowId, userId, 'fork', 'Forked workflow', { 
      forkId: fork.id, name 
    })

    return fork
  }

  // Activity tracking
  getWorkflowActivity(workflowId: string, userId: string, limit: number = 50): WorkflowActivity[] {
    if (!this.canViewActivity(workflowId, userId)) {
      throw new Error('Insufficient permissions to view activity')
    }

    const activities = this.activities.get(workflowId) || []
    return activities
      .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
      .slice(0, limit)
  }

  private logActivity(
    workflowId: string,
    userId: string,
    type: WorkflowActivity['type'],
    description: string,
    details: Record<string, any>
  ): void {
    const activity: WorkflowActivity = {
      id: this.generateId('activity'),
      workflowId,
      userId,
      type,
      description,
      details,
      timestamp: TimestampUtils.dateToICPTimestamp(new Date())
    }

    const activities = this.activities.get(workflowId) || []
    activities.push(activity)
    
    // Keep only last 1000 activities
    if (activities.length > 1000) {
      activities.splice(0, activities.length - 1000)
    }
    
    this.activities.set(workflowId, activities)
  }

  // Permission checks
  private canManageSharing(shared: SharedWorkflow, userId: string): boolean {
    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.permissions.share || collaborator?.role === 'owner' || false
  }

  private canInviteCollaborators(shared: SharedWorkflow, userId: string): boolean {
    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.permissions.share || collaborator?.role === 'owner' || false
  }

  private canManageCollaborators(shared: SharedWorkflow, userId: string): boolean {
    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.role === 'owner' || false
  }

  private canComment(workflowId: string, userId: string): boolean {
    const shared = Array.from(this.sharedWorkflows.values()).find(s => s.workflowId === workflowId)
    if (!shared) return false

    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.permissions.comment || false
  }

  private canViewComments(workflowId: string, userId: string): boolean {
    const shared = Array.from(this.sharedWorkflows.values()).find(s => s.workflowId === workflowId)
    if (!shared) return false

    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.permissions.view || false
  }

  private canModerateComments(workflowId: string, userId: string): boolean {
    const shared = Array.from(this.sharedWorkflows.values()).find(s => s.workflowId === workflowId)
    if (!shared) return false

    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.role === 'owner' || false
  }

  private canForkWorkflow(workflowId: string, userId: string): boolean {
    const shared = Array.from(this.sharedWorkflows.values()).find(s => s.workflowId === workflowId)
    if (!shared) return false

    return shared.shareSettings.allowFork && 
           (shared.shareSettings.visibility === 'public' || 
            shared.collaborators.some(c => c.userId === userId && c.permissions.view))
  }

  private canViewActivity(workflowId: string, userId: string): boolean {
    const shared = Array.from(this.sharedWorkflows.values()).find(s => s.workflowId === workflowId)
    if (!shared) return false

    const collaborator = shared.collaborators.find(c => c.userId === userId)
    return collaborator?.permissions.view || false
  }

  // Helper methods
  private getCollaboratorRole(permissions: WorkflowPermissions): WorkflowCollaborator['role'] {
    if (permissions.delete && permissions.share) return 'owner'
    if (permissions.edit) return 'editor'
    if (permissions.comment) return 'commenter'
    return 'viewer'
  }

  private async sendInvitationEmails(invitations: WorkflowInvitation[]): Promise<void> {
    // Simulate sending emails
    for (const invitation of invitations) {
      console.log(`[Email] Invitation sent to ${invitation.invitedEmail}`)
    }
  }

  private generateId(prefix: string): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  // Public API methods
  getSharedWorkflow(shareId: string): SharedWorkflow | null {
    return this.sharedWorkflows.get(shareId) || null
  }

  getUserSharedWorkflows(userId: string): SharedWorkflow[] {
    return Array.from(this.sharedWorkflows.values()).filter(s =>
      s.collaborators.some(c => c.userId === userId && c.isActive)
    )
  }

  getUserInvitations(userId: string): WorkflowInvitation[] {
    return Array.from(this.invitations.values()).filter(i =>
      i.invitedEmail === userId && i.status === 'pending' // Simplified: using userId as email
    )
  }

  getWorkflowForks(workflowId: string): WorkflowFork[] {
    return Array.from(this.forks.values()).filter(f =>
      f.originalWorkflowId === workflowId
    )
  }

  // Demo data initialization
  private initializeDemoData(): void {
    // Initialize some demo shared workflows and comments
    const demoShare: SharedWorkflow = {
      id: 'share_001',
      workflowId: 'workflow_001',
      ownerId: 'user_001',
      permissions: {
        view: true,
        edit: true,
        execute: true,
        share: true,
        delete: true,
        comment: true,
        analytics: true
      },
      shareSettings: {
        visibility: 'team',
        allowComments: true,
        allowFork: true,
        allowExport: false
      },
      invitations: [],
      collaborators: [{
        id: 'collab_001',
        userId: 'user_001',
        sharedWorkflowId: 'share_001',
        permissions: {
          view: true,
          edit: true,
          execute: true,
          share: true,
          delete: true,
          comment: true,
          analytics: true
        },
        addedBy: 'user_001',
        addedAt: TimestampUtils.dateToICPTimestamp(new Date()),
        isActive: true,
        role: 'owner'
      }],
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      updatedAt: TimestampUtils.dateToICPTimestamp(new Date()),
      isActive: true
    }

    this.sharedWorkflows.set(demoShare.id, demoShare)
  }
}

// Export singleton instance
export const collaborationService = new CollaborationService()
export default collaborationService