import type { TenantUser } from './types';
import { apiFetch } from './client';

export interface CreateTenantUserPayload {
  full_name: string;
  email: string;
  password: string;
  role: 'admin' | 'developer' | 'read_only';
}

export interface CreateTenantUserResponse {
  id: string;
  email: string;
  full_name: string;
  role: string;
  enabled: boolean;
}

export interface UpdateTenantUserPayload {
  full_name?: string;
  role?: 'admin' | 'developer' | 'read_only';
  enabled?: boolean;
}

export interface UpdateTenantUserResponse {
  id: string;
  updated: boolean;
}

export const appUsersAPI = {
  list: async (): Promise<TenantUser[]> => apiFetch('/app/users', { method: 'GET' }),
  create: async (payload: CreateTenantUserPayload) =>
    apiFetch<CreateTenantUserResponse>('/app/users', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  update: async (id: string, payload: UpdateTenantUserPayload) =>
    apiFetch<UpdateTenantUserResponse>(`/app/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
};
