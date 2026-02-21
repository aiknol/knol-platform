import { apiFetch } from './client';

export interface AdminUser {
  id: string;
  email: string;
  role: 'admin' | 'super_admin';
  enabled: boolean;
  created_at?: string;
}

export const usersAPI = {
  list: async (): Promise<AdminUser[]> => apiFetch('/admin/users'),
  create: async (email: string, password: string, role: 'admin' | 'super_admin') =>
    apiFetch('/admin/users', { method: 'POST', body: JSON.stringify({ email, password, role }) }),
  update: async (id: string, role?: 'admin' | 'super_admin', enabled?: boolean) =>
    apiFetch(`/admin/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ ...(role && { role }), ...(enabled !== undefined && { enabled }) }),
    }),
  delete: async (id: string) => apiFetch(`/admin/users/${id}`, { method: 'DELETE' }),
};
