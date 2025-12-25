import { create } from 'zustand';
import { apiClient } from '@/shared/api/client';
import type { Invite, InviteCountResponse, UserSettings } from '@/shared/types/api';

interface InviteState {
  // User's pending invites (received)
  userInvites: Invite[];
  userInvitesLoading: boolean;
  userInvitesError: string | null;

  // Organization's pending invites (sent)
  orgInvites: Invite[];
  orgInvitesLoading: boolean;
  orgInvitesError: string | null;

  // Invite count for badge
  inviteCount: number;

  // User settings
  settings: UserSettings | null;
  settingsLoading: boolean;

  // Actions for user invites
  fetchUserInvites: () => Promise<void>;
  fetchInviteCount: () => Promise<void>;
  acceptInvite: (inviteId: string) => Promise<void>;
  declineInvite: (inviteId: string) => Promise<void>;

  // Actions for org invites (admin view)
  fetchOrgInvites: (orgId: string) => Promise<void>;
  sendInvite: (orgId: string, email: string, role: 'admin' | 'member') => Promise<Invite>;
  cancelInvite: (orgId: string, inviteId: string) => Promise<void>;

  // Settings actions
  fetchSettings: () => Promise<void>;
  updateSettings: (settings: Partial<UserSettings>) => Promise<void>;

  // Utility
  clearUserInvitesError: () => void;
  clearOrgInvitesError: () => void;
}

export const useInviteStore = create<InviteState>((set) => ({
  userInvites: [],
  userInvitesLoading: false,
  userInvitesError: null,

  orgInvites: [],
  orgInvitesLoading: false,
  orgInvitesError: null,

  inviteCount: 0,

  settings: null,
  settingsLoading: false,

  fetchUserInvites: async () => {
    set({ userInvitesLoading: true, userInvitesError: null });
    try {
      const invites = await apiClient.get<Invite[]>('/invites');
      set({ userInvites: invites, userInvitesLoading: false });
    } catch (error) {
      set({
        userInvitesError: error instanceof Error ? error.message : 'Failed to fetch invites',
        userInvitesLoading: false,
      });
    }
  },

  fetchInviteCount: async () => {
    try {
      const response = await apiClient.get<InviteCountResponse>('/invites/count');
      set({ inviteCount: response.count });
    } catch {
      // Silently fail for count
    }
  },

  acceptInvite: async (inviteId: string) => {
    await apiClient.post(`/invites/${inviteId}/accept`);
    // Remove from local state
    set((state) => ({
      userInvites: state.userInvites.filter((inv) => inv.id !== inviteId),
      inviteCount: Math.max(0, state.inviteCount - 1),
    }));
  },

  declineInvite: async (inviteId: string) => {
    await apiClient.post(`/invites/${inviteId}/decline`);
    // Remove from local state
    set((state) => ({
      userInvites: state.userInvites.filter((inv) => inv.id !== inviteId),
      inviteCount: Math.max(0, state.inviteCount - 1),
    }));
  },

  fetchOrgInvites: async (orgId: string) => {
    set({ orgInvitesLoading: true, orgInvitesError: null });
    try {
      const invites = await apiClient.get<Invite[]>(`/orgs/${orgId}/invites`);
      set({ orgInvites: invites, orgInvitesLoading: false });
    } catch (error) {
      set({
        orgInvitesError: error instanceof Error ? error.message : 'Failed to fetch invites',
        orgInvitesLoading: false,
      });
    }
  },

  sendInvite: async (orgId: string, email: string, role: 'admin' | 'member') => {
    const invite = await apiClient.post<Invite>(`/orgs/${orgId}/invites`, { email, role });
    // Add to local state
    set((state) => ({
      orgInvites: [...state.orgInvites, invite],
    }));
    return invite;
  },

  cancelInvite: async (orgId: string, inviteId: string) => {
    await apiClient.delete(`/orgs/${orgId}/invites/${inviteId}`);
    // Remove from local state
    set((state) => ({
      orgInvites: state.orgInvites.filter((inv) => inv.id !== inviteId),
    }));
  },

  fetchSettings: async () => {
    set({ settingsLoading: true });
    try {
      const settings = await apiClient.get<UserSettings>('/auth/me/settings');
      set({ settings, settingsLoading: false });
    } catch {
      set({ settingsLoading: false });
    }
  },

  updateSettings: async (newSettings: Partial<UserSettings>) => {
    await apiClient.patch('/auth/me/settings', newSettings);
    // Update state directly with the new values
    set((state) => ({
      settings: state.settings
        ? { ...state.settings, ...newSettings }
        : { allow_invites: newSettings.allow_invites ?? true },
    }));
  },

  clearUserInvitesError: () => set({ userInvitesError: null }),
  clearOrgInvitesError: () => set({ orgInvitesError: null }),
}));
