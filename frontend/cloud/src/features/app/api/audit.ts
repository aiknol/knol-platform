import type { TenantAuditItem } from './types';
import { apiFetch } from './client';

export const appAuditAPI = {
  list: async (): Promise<TenantAuditItem[]> => {
    const res = await apiFetch<{ data: TenantAuditItem[] }>('/app/audit', { method: 'GET' });
    return res.data;
  },
};
