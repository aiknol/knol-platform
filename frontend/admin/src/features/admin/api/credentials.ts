import { apiFetch } from './client';

export interface Credential {
  name: string;
  service?: string;
  value?: string;
  last_rotated?: string;
  description?: string;
}

export const credentialsAPI = {
  list: async (): Promise<Credential[]> => apiFetch('/admin/credentials'),
  update: async (name: string, value: string, service?: string, description?: string) =>
    apiFetch(`/admin/credentials/${name}`, {
      method: 'PUT',
      body: JSON.stringify({ value, ...(service && { service }), ...(description && { description }) }),
    }),
  delete: async (name: string) => apiFetch(`/admin/credentials/${name}`, { method: 'DELETE' }),
  test: async (name: string) => apiFetch(`/admin/credentials/${name}/test`, { method: 'POST' }),
};
