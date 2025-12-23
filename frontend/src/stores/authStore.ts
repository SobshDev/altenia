import { create } from 'zustand';
import { apiClient } from '@/shared/api/client';
import type { AuthResponse, User } from '@/shared/types/api';

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  login: (email: string, password: string, fingerprint: string) => Promise<void>;
  register: (email: string, password: string, fingerprint: string) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  isAuthenticated: !!localStorage.getItem('accessToken'),
  isLoading: false,
  error: null,

  login: async (email: string, password: string, fingerprint: string) => {
    set({ isLoading: true, error: null });
    try {
      localStorage.setItem('deviceFingerprint', fingerprint);
      const response = await apiClient.post<AuthResponse>('/auth/login', {
        email,
        password,
      });

      apiClient.setTokens(response.access_token, response.refresh_token);
      set({
        user: {
          id: response.user_id,
          email: response.email,
          created_at: '',
        },
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Login failed',
        isLoading: false,
      });
      throw error;
    }
  },

  register: async (email: string, password: string, fingerprint: string) => {
    set({ isLoading: true, error: null });
    try {
      localStorage.setItem('deviceFingerprint', fingerprint);
      const response = await apiClient.post<AuthResponse>('/auth/register', {
        email,
        password,
      });

      apiClient.setTokens(response.access_token, response.refresh_token);
      set({
        user: {
          id: response.user_id,
          email: response.email,
          created_at: '',
        },
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Registration failed',
        isLoading: false,
      });
      throw error;
    }
  },

  logout: async () => {
    try {
      await apiClient.post('/auth/logout');
    } catch {
      // Ignore logout errors
    } finally {
      apiClient.clearTokens();
      set({ user: null, isAuthenticated: false });
    }
  },

  checkAuth: async () => {
    if (!apiClient.getAccessToken()) {
      set({ isAuthenticated: false, user: null });
      return;
    }

    set({ isLoading: true });
    try {
      const user = await apiClient.get<User>('/auth/me');
      set({ user, isAuthenticated: true, isLoading: false });
    } catch {
      apiClient.clearTokens();
      set({ user: null, isAuthenticated: false, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
