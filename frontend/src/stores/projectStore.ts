import { create } from 'zustand';
import { apiClient } from '@/shared/api/client';
import type {
  Project,
  ApiKey,
  CreateProjectRequest,
  UpdateProjectRequest,
  CreateApiKeyRequest,
  CreateApiKeyResponse,
} from '@/shared/types/api';

interface ProjectState {
  // Projects
  projects: Project[];
  currentProject: Project | null;
  isLoading: boolean;
  error: string | null;

  // Expanded projects in sidebar (UI state)
  expandedProjects: Set<string>;

  // API Keys
  apiKeys: ApiKey[];
  apiKeysLoading: boolean;
  apiKeysError: string | null;

  // Project actions
  fetchProjects: (orgId: string) => Promise<void>;
  createProject: (orgId: string, data: CreateProjectRequest) => Promise<Project>;
  updateProject: (projectId: string, data: UpdateProjectRequest) => Promise<void>;
  deleteProject: (projectId: string) => Promise<void>;
  setCurrentProject: (project: Project | null) => void;
  getProjectById: (projectId: string) => Project | undefined;

  // UI state actions
  toggleProjectExpanded: (projectId: string) => void;
  setProjectExpanded: (projectId: string, expanded: boolean) => void;

  // API Key actions
  fetchApiKeys: (projectId: string) => Promise<void>;
  createApiKey: (projectId: string, data: CreateApiKeyRequest) => Promise<CreateApiKeyResponse>;
  revokeApiKey: (projectId: string, keyId: string) => Promise<void>;

  // Utility
  clearError: () => void;
  clearApiKeysError: () => void;
  reset: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  projects: [],
  currentProject: null,
  isLoading: false,
  error: null,
  expandedProjects: new Set(),
  apiKeys: [],
  apiKeysLoading: false,
  apiKeysError: null,

  fetchProjects: async (orgId: string) => {
    set({ isLoading: true, error: null });
    try {
      const projects = await apiClient.get<Project[]>(`/orgs/${orgId}/projects`);
      set({ projects, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch projects',
        isLoading: false,
      });
    }
  },

  createProject: async (orgId: string, data: CreateProjectRequest) => {
    const project = await apiClient.post<Project>(`/orgs/${orgId}/projects`, data);
    set((state) => ({
      projects: [...state.projects, project],
    }));
    return project;
  },

  updateProject: async (projectId: string, data: UpdateProjectRequest) => {
    const updated = await apiClient.patch<Project>(`/projects/${projectId}`, data);
    set((state) => ({
      projects: state.projects.map((p) => (p.id === projectId ? updated : p)),
      currentProject: state.currentProject?.id === projectId ? updated : state.currentProject,
    }));
  },

  deleteProject: async (projectId: string) => {
    await apiClient.delete(`/projects/${projectId}`);
    set((state) => {
      const newExpanded = new Set(state.expandedProjects);
      newExpanded.delete(projectId);
      return {
        projects: state.projects.filter((p) => p.id !== projectId),
        currentProject: state.currentProject?.id === projectId ? null : state.currentProject,
        expandedProjects: newExpanded,
      };
    });
  },

  setCurrentProject: (project: Project | null) => set({ currentProject: project }),

  getProjectById: (projectId: string) => {
    return get().projects.find((p) => p.id === projectId);
  },

  toggleProjectExpanded: (projectId: string) => {
    set((state) => {
      const newSet = new Set(state.expandedProjects);
      if (newSet.has(projectId)) {
        newSet.delete(projectId);
      } else {
        newSet.add(projectId);
      }
      return { expandedProjects: newSet };
    });
  },

  setProjectExpanded: (projectId: string, expanded: boolean) => {
    set((state) => {
      const newSet = new Set(state.expandedProjects);
      if (expanded) {
        newSet.add(projectId);
      } else {
        newSet.delete(projectId);
      }
      return { expandedProjects: newSet };
    });
  },

  fetchApiKeys: async (projectId: string) => {
    set({ apiKeysLoading: true, apiKeysError: null });
    try {
      const apiKeys = await apiClient.get<ApiKey[]>(`/projects/${projectId}/api-keys`);
      set({ apiKeys, apiKeysLoading: false });
    } catch (error) {
      set({
        apiKeysError: error instanceof Error ? error.message : 'Failed to fetch API keys',
        apiKeysLoading: false,
      });
    }
  },

  createApiKey: async (projectId: string, data: CreateApiKeyRequest) => {
    const response = await apiClient.post<CreateApiKeyResponse>(
      `/projects/${projectId}/api-keys`,
      data
    );
    // Add to list without plain_key
    const apiKey: ApiKey = {
      id: response.id,
      name: response.name,
      key_prefix: response.key_prefix,
      created_at: response.created_at,
      expires_at: response.expires_at,
      is_active: true,
    };
    set((state) => ({
      apiKeys: [...state.apiKeys, apiKey],
    }));
    return response;
  },

  revokeApiKey: async (projectId: string, keyId: string) => {
    await apiClient.delete(`/projects/${projectId}/api-keys/${keyId}`);
    set((state) => ({
      apiKeys: state.apiKeys.filter((k) => k.id !== keyId),
    }));
  },

  clearError: () => set({ error: null }),
  clearApiKeysError: () => set({ apiKeysError: null }),

  reset: () =>
    set({
      projects: [],
      currentProject: null,
      expandedProjects: new Set(),
      apiKeys: [],
      error: null,
      apiKeysError: null,
    }),
}));
