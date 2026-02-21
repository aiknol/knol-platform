import { apiFetch } from './client';
import { clearAuthSession, setAuthSession, setAuthUser } from './session';

export interface AdminAuthUser {
  id: string;
  email: string;
  role: string;
}

export const authAPI = {
  login: async (email: string, password: string) => {
    const data = await apiFetch<{ token?: string; admin?: AdminAuthUser; expires_at?: string }>(
      '/admin/auth/login',
      {
        method: 'POST',
        skipAuth: true,
        body: JSON.stringify({ email, password }),
      }
    );
    if (data.token && data.admin) {
      setAuthSession(data.token, data.admin);
    }
    return data;
  },

  logout: async () => {
    await apiFetch('/admin/auth/logout', { method: 'POST' });
    clearAuthSession();
  },

  me: async (): Promise<{ admin: AdminAuthUser }> => {
    const data = await apiFetch<{ admin: AdminAuthUser }>('/admin/auth/me', {
      method: 'GET',
      // Allow cookie-based auth even when no bearer token is present.
      skipAuth: true,
    });
    if (data?.admin) {
      setAuthUser(data.admin);
    }
    return data;
  },

  changePassword: async (currentPassword: string, newPassword: string) => {
    return apiFetch('/admin/auth/change-password', {
      method: 'POST',
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
    });
  },
};
