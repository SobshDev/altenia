import { create } from 'zustand';
import { apiClient } from '@/shared/api/client';
import type { Organization, OrgMember, SwitchOrgResponse } from '@/shared/types/api';

interface OrgState {
  organizations: Organization[];
  currentOrg: Organization | null;
  members: OrgMember[];
  isLoading: boolean;
  error: string | null;

  fetchOrganizations: () => Promise<void>;
  switchOrg: (orgId: string) => Promise<void>;
  createOrg: (name: string) => Promise<Organization>;
  updateOrg: (name: string) => Promise<void>;
  deleteOrg: () => Promise<void>;
  leaveOrg: () => Promise<void>;
  fetchMembers: () => Promise<void>;
  addMember: (email: string, role: string) => Promise<void>;
  updateMemberRole: (userId: string, role: string) => Promise<void>;
  removeMember: (userId: string) => Promise<void>;
  transferOwnership: (newOwnerUserId: string) => Promise<void>;
  clearError: () => void;
}

export const useOrgStore = create<OrgState>((set, get) => ({
  organizations: [],
  currentOrg: null,
  members: [],
  isLoading: false,
  error: null,

  fetchOrganizations: async () => {
    set({ isLoading: true, error: null });
    try {
      const orgs = await apiClient.get<Organization[]>('/orgs');
      const currentOrg = get().currentOrg;
      set({
        organizations: orgs,
        // If no current org selected, select the first one (personal org)
        currentOrg: currentOrg || orgs[0] || null,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch organizations',
        isLoading: false,
      });
    }
  },

  switchOrg: async (orgId: string) => {
    const { organizations } = get();
    const org = organizations.find((o) => o.id === orgId);
    if (!org) return;

    set({ isLoading: true, error: null });
    try {
      const response = await apiClient.post<SwitchOrgResponse>(`/orgs/${orgId}/switch`, {});
      apiClient.setTokens(response.access_token, response.refresh_token);
      set({
        currentOrg: response.organization,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to switch organization',
        isLoading: false,
      });
    }
  },

  createOrg: async (name: string) => {
    set({ isLoading: true, error: null });
    try {
      const org = await apiClient.post<Organization>('/orgs', { name });
      set((state) => ({
        organizations: [...state.organizations, org],
        currentOrg: org,
        isLoading: false,
      }));
      // Switch to the new org to get proper tokens
      const response = await apiClient.post<SwitchOrgResponse>(`/orgs/${org.id}/switch`, {});
      apiClient.setTokens(response.access_token, response.refresh_token);
      return org;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to create organization',
        isLoading: false,
      });
      throw error;
    }
  },

  updateOrg: async (name: string) => {
    const { currentOrg } = get();
    if (!currentOrg) return;

    const updatedOrg = await apiClient.patch<Organization>(`/orgs/${currentOrg.id}`, { name });
    set((state) => ({
      currentOrg: updatedOrg,
      organizations: state.organizations.map((o) =>
        o.id === updatedOrg.id ? updatedOrg : o
      ),
    }));
  },

  deleteOrg: async () => {
    const { currentOrg, organizations } = get();
    if (!currentOrg) return;

    await apiClient.delete(`/orgs/${currentOrg.id}`);
    const remaining = organizations.filter((o) => o.id !== currentOrg.id);
    const personalOrg = remaining.find((o) => o.is_personal) || remaining[0];

    if (personalOrg) {
      const response = await apiClient.post<SwitchOrgResponse>(`/orgs/${personalOrg.id}/switch`, {});
      apiClient.setTokens(response.access_token, response.refresh_token);
      set({
        organizations: remaining,
        currentOrg: response.organization,
        members: [],
      });
    } else {
      set({
        organizations: remaining,
        currentOrg: null,
        members: [],
      });
    }
  },

  leaveOrg: async () => {
    const { currentOrg, organizations } = get();
    if (!currentOrg) return;

    await apiClient.post(`/orgs/${currentOrg.id}/leave`, {});
    const remaining = organizations.filter((o) => o.id !== currentOrg.id);
    const personalOrg = remaining.find((o) => o.is_personal) || remaining[0];

    if (personalOrg) {
      const response = await apiClient.post<SwitchOrgResponse>(`/orgs/${personalOrg.id}/switch`, {});
      apiClient.setTokens(response.access_token, response.refresh_token);
      set({
        organizations: remaining,
        currentOrg: response.organization,
        members: [],
      });
    } else {
      set({
        organizations: remaining,
        currentOrg: null,
        members: [],
      });
    }
  },

  fetchMembers: async () => {
    const { currentOrg } = get();
    if (!currentOrg) return;

    const members = await apiClient.get<OrgMember[]>(`/orgs/${currentOrg.id}/members`);
    set({ members });
  },

  addMember: async (email: string, role: string) => {
    const { currentOrg } = get();
    if (!currentOrg) return;

    const member = await apiClient.post<OrgMember>(`/orgs/${currentOrg.id}/members`, {
      email,
      role,
    });
    set((state) => ({
      members: [...state.members, member],
    }));
  },

  updateMemberRole: async (userId: string, role: string) => {
    const { currentOrg } = get();
    if (!currentOrg) return;

    const member = await apiClient.patch<OrgMember>(
      `/orgs/${currentOrg.id}/members/${userId}`,
      { role }
    );
    set((state) => ({
      members: state.members.map((m) => (m.user_id === userId ? member : m)),
    }));
  },

  removeMember: async (userId: string) => {
    const { currentOrg } = get();
    if (!currentOrg) return;

    await apiClient.delete(`/orgs/${currentOrg.id}/members/${userId}`);
    set((state) => ({
      members: state.members.filter((m) => m.user_id !== userId),
    }));
  },

  transferOwnership: async (newOwnerUserId: string) => {
    const { currentOrg, fetchMembers } = get();
    if (!currentOrg) return;

    const response = await apiClient.post<SwitchOrgResponse>(`/orgs/${currentOrg.id}/transfer`, {
      new_owner_user_id: newOwnerUserId,
    });

    // Update tokens with new role (response includes new tokens after transfer)
    if (response?.access_token && response?.refresh_token) {
      apiClient.setTokens(response.access_token, response.refresh_token);
      set({ currentOrg: response.organization });
    }

    // Refresh members to get updated roles
    await fetchMembers();
  },

  clearError: () => set({ error: null }),
}));
