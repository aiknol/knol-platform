import { apiFetch } from './client';

export interface Tenant {
  id: string;
  name: string;
  plan?: string;
  config?: Record<string, unknown>;
  usage_limit?: number;
  created_at?: string;
}

export const tenantsAPI = {
  list: async (): Promise<Tenant[]> => apiFetch('/admin/tenants'),
  getOne: async (id: string): Promise<Tenant> => apiFetch(`/admin/tenants/${id}`),
  update: async (id: string, plan?: string, config?: Record<string, unknown>, usageLimit?: number, name?: string) =>
    apiFetch(`/admin/tenants/${id}`, {
      method: 'PUT',
      body: JSON.stringify({
        ...(plan && { plan }),
        ...(config && { config }),
        ...(usageLimit !== undefined && { usage_limit: usageLimit }),
        ...(name && { name }),
      }),
    }),
};
