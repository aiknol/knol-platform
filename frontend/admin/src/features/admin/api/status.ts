import { apiFetch } from './client';

export interface ServiceStatus {
  name: string;
  status: 'up' | 'down' | 'degraded';
  latency_ms?: number;
}

export interface DatabaseStatus {
  version?: string;
  pool_size?: number;
}

export interface SystemStatus {
  services: ServiceStatus[];
  db?: DatabaseStatus;
  database?: DatabaseStatus;
  counts: { configs: number; credentials: number; tenants: number };
}

export const statusAPI = {
  get: async (): Promise<SystemStatus> => {
    const data = await apiFetch<SystemStatus>('/admin/status');
    if (data?.database && !data?.db) {
      data.db = data.database;
    }
    return data;
  },
};
