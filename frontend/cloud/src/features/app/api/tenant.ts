import type { TenantProfile } from './types';
import { apiFetch } from './client';

export const appTenantAPI = {
  get: async (): Promise<TenantProfile> => apiFetch('/app/tenant', { method: 'GET' }),
};
