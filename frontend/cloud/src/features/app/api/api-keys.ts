import type { ApiKeyItem } from './types';
import { apiFetch } from './client';

export interface CreateApiKeyPayload {
  name: string;
  role: 'admin' | 'developer' | 'read_only';
  expires_in_days?: number;
}

export interface CreateApiKeyResponse {
  id: string;
  name: string;
  role: string;
  expires_at?: string;
  api_key: string;
}

export interface RevokeApiKeyResponse {
  id: string;
  revoked: boolean;
}

export const appApiKeysAPI = {
  list: async (): Promise<ApiKeyItem[]> => apiFetch('/app/api-keys', { method: 'GET' }),
  create: async (payload: CreateApiKeyPayload) =>
    apiFetch<CreateApiKeyResponse>('/app/api-keys', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  revoke: async (id: string) =>
    apiFetch<RevokeApiKeyResponse>(`/app/api-keys/${id}`, {
      method: 'DELETE',
    }),
};
