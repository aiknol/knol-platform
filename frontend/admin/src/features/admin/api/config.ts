import { apiFetch } from './client';

export interface Config {
  id?: string;
  key: string;
  value: string | number | boolean | Record<string, unknown> | Array<unknown> | null;
  value_type?: 'string' | 'number' | 'boolean' | 'json' | 'string_array';
  category: string;
  description?: string;
  env_override?: string | null;
  updated_at?: string;
}

export const configAPI = {
  getByCategory: async (category: string): Promise<Config[]> =>
    apiFetch(`/admin/config?category=${encodeURIComponent(category)}`),
  getAll: async (): Promise<Config[]> => apiFetch('/admin/config'),
  getOne: async (key: string): Promise<Config> => apiFetch(`/admin/config/${encodeURIComponent(key)}`),
  update: async (
    key: string,
    payload: {
      value: Config['value'];
      value_type?: Config['value_type'];
      category?: string;
      description?: string;
      env_override?: string | null;
    },
  ) =>
    apiFetch(`/admin/config/${encodeURIComponent(key)}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
  delete: async (key: string) => apiFetch(`/admin/config/${encodeURIComponent(key)}`, { method: 'DELETE' }),
};
