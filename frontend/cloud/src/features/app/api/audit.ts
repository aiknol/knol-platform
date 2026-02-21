import type { TenantAuditItem } from './types';
import { apiFetch } from './client';

export const appAuditAPI = {
  list: async (): Promise<TenantAuditItem[]> => apiFetch('/app/audit', { method: 'GET' }),
};
