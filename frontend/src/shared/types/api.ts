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
  | 'org_name_changed';

export interface Activity {
  id: string;
  type: ActivityType;
  actor_email: string;
  target_email?: string;
  metadata?: Record<string, string>;
  created_at: string;
}
