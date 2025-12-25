export interface ApiError {
  error: string;
  message?: string;
}

export interface AuthResponse {
  user_id: string;
  email: string;
  display_name?: string;
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface User {
  id: string;
  email: string;
  display_name?: string;
  created_at?: string;
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  is_personal: boolean;
  role: string;
  created_at: string;
}

export interface CreateOrgRequest {
  name: string;
}

export interface SwitchOrgResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  organization: Organization;
}

export interface OrgMember {
  id: string;
  user_id: string;
  email: string;
  display_name?: string;
  role: 'owner' | 'admin' | 'member';
  joined_at: string;
}

export type ActivityType =
  | 'member_added'
  | 'member_removed'
  | 'member_role_changed'
  | 'org_created'
  | 'org_name_changed'
  | 'invite_sent'
  | 'invite_accepted'
  | 'invite_declined';

export interface Activity {
  id: string;
  type: ActivityType;
  actor_email: string;
  target_email?: string;
  metadata?: Record<string, string>;
  created_at: string;
}

export type InviteStatus = 'pending' | 'accepted' | 'declined' | 'expired';

export interface Invite {
  id: string;
  organization_id: string;
  organization_name: string;
  inviter_email: string;
  invitee_email: string;
  role: 'admin' | 'member';
  status: InviteStatus;
  expires_at: string;
  created_at: string;
}

export interface InviteCountResponse {
  count: number;
}

export interface UserSettings {
  allow_invites: boolean;
}

export interface SendInviteRequest {
  email: string;
  role: 'admin' | 'member';
}
