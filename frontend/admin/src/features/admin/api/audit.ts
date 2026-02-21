import { apiFetch } from './client';

export interface AuditLog {
  id: string;
  timestamp: string;
  admin_id?: string;
  admin_email?: string;
  action: string;
  resource_type?: string;
  resource_id?: string;
  old_value?: unknown;
  new_value?: unknown;
}

export const auditAPI = {
  list: async (action?: string, resourceType?: string, limit: number = 50): Promise<AuditLog[]> => {
    const params = new URLSearchParams();
    if (action) params.append('action', action);
    if (resourceType) params.append('resource_type', resourceType);
    params.append('limit', limit.toString());
    return apiFetch(`/admin/audit?${params.toString()}`);
  },
};
