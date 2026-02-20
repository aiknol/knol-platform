function resolveDefaultAdminApiUrl(): string {
  if (typeof window === 'undefined') {
    return 'https://admin.aiknol.com';
  }
  const host = window.location.hostname;
  const isLocal = host === 'localhost' || host === '127.0.0.1';
  return isLocal ? 'http://localhost:3001' : 'https://admin.aiknol.com';
}

const API_URL = process.env.NEXT_PUBLIC_ADMIN_API_URL || resolveDefaultAdminApiUrl();

if (
  typeof window !== 'undefined' &&
  process.env.NODE_ENV !== 'production' &&
  !process.env.NEXT_PUBLIC_ADMIN_API_URL
) {
  console.warn(
    `[Admin API] NEXT_PUBLIC_ADMIN_API_URL is not set. Falling back to ${API_URL}. ` +
    'Set this env var at build time for production deployments.'
  );
}

type FetchOptions = RequestInit & {
  skipAuth?: boolean;
};

export function getAuthToken(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem('admin_token') || localStorage.getItem('admin_token');
}

export function getAuthUser(): { email: string; role: string } | null {
  if (typeof window === 'undefined') return null;
  const raw = sessionStorage.getItem('admin_user') || localStorage.getItem('admin_user');
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function setAuthSession(token: string, admin: unknown) {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem('admin_token', token);
  sessionStorage.setItem('admin_user', JSON.stringify(admin));
  // Clear legacy persistent storage.
  localStorage.removeItem('admin_token');
  localStorage.removeItem('admin_user');
}

export function clearAuthSession() {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem('admin_token');
  sessionStorage.removeItem('admin_user');
  localStorage.removeItem('admin_token');
  localStorage.removeItem('admin_user');
}

async function apiFetch(endpoint: string, options: FetchOptions = {}) {
  const { skipAuth = false, ...fetchOptions } = options;
  const url = `${API_URL}${endpoint}`;

  const headers = new Headers(fetchOptions.headers || {});
  headers.set('Content-Type', 'application/json');

  if (!skipAuth) {
    const token = getAuthToken();
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
  }

  const response = await fetch(url, {
    ...fetchOptions,
    headers,
    // SECURITY: Include cookies so the HttpOnly JWT cookie is sent automatically.
    // This works alongside the Bearer token for backward compatibility.
    credentials: 'include',
  });

  if (response.status === 401) {
    if (typeof window !== 'undefined') {
      clearAuthSession();
      window.location.href = '/admin/login';
    }
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API Error ${response.status}: ${error}`);
  }

  const contentType = response.headers.get('content-type');
  if (contentType?.includes('application/json')) {
    return response.json();
  }
  return response.text();
}

// ── Auth ─────────────────────────────────────────────────────────

export const authAPI = {
  login: async (email: string, password: string) => {
    const data = await apiFetch('/admin/auth/login', {
      method: 'POST',
      skipAuth: true,
      body: JSON.stringify({ email, password }),
    });
    if (data.token) {
      setAuthSession(data.token, data.admin);
    }
    return data;
  },

  logout: async () => {
    await apiFetch('/admin/auth/logout', { method: 'POST' });
    clearAuthSession();
  },

  changePassword: async (currentPassword: string, newPassword: string) => {
    return apiFetch('/admin/auth/change-password', {
      method: 'POST',
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
    });
  },
};

// ── Config ───────────────────────────────────────────────────────

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

// ── Credentials ──────────────────────────────────────────────────

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

// ── Campaigns ────────────────────────────────────────────────────

export interface Campaign {
  id: string;
  name: string;
  enabled: boolean;
  cron?: string;
  channels?: string[];
  phase?: string;
  description?: string;
  created_at?: string;
  updated_at?: string;
  last_publish?: {
    channel: string;
    success: boolean;
    published_at: string;
  } | null;
  stats?: {
    total_publishes: number;
    successful: number;
    success_rate: number;
  } | null;
}

export interface CampaignLog {
  campaign: string;
  channel: string;
  success: boolean;
  message_id?: string;
  url?: string;
  error?: string;
  published_at: string;
}

export interface MarketingStats {
  period_days: number;
  strategy: string;
  summary: {
    total_publishes: number;
    successful: number;
    success_rate: number;
  };
  by_channel: Array<{
    channel: string;
    total: number;
    successful: number;
    success_rate: number;
  }>;
  by_phase: Array<{
    phase: string;
    total: number;
    successful: number;
  }>;
  daily: Array<{
    date: string;
    total: number;
    successful: number;
  }>;
  metrics: Array<{
    name: string;
    value: number;
    recorded_at: string;
    metadata?: Record<string, unknown>;
  }>;
}

export const campaignsAPI = {
  list: async (): Promise<Campaign[]> => apiFetch('/admin/campaigns'),
  update: async (
    name: string,
    enabled?: boolean,
    cron?: string,
    channels?: string[],
    phase?: string,
    description?: string,
  ) =>
    apiFetch(`/admin/campaigns/${name}`, {
      method: 'PUT',
      body: JSON.stringify({
        ...(enabled !== undefined && { enabled }),
        ...(cron && { cron }),
        ...(channels && { channels }),
        ...(phase && { phase }),
        ...(description && { description }),
      }),
    }),
  getLogs: async (campaignName: string, limit: number = 50): Promise<CampaignLog[]> =>
    apiFetch(`/admin/campaigns/${campaignName}/logs?limit=${limit}`),
  trigger: async (campaignName: string, force: boolean = false) =>
    apiFetch(`/admin/campaigns/${campaignName}/trigger?force=${force}`, { method: 'POST' }),
  getStats: async (days: number = 30): Promise<MarketingStats> =>
    apiFetch(`/admin/marketing/stats?days=${days}`),
  recordMetric: async (metricName: string, metricValue: number, metadata?: Record<string, unknown>) =>
    apiFetch('/admin/marketing/metrics', {
      method: 'POST',
      body: JSON.stringify({ metric_name: metricName, metric_value: metricValue, metadata }),
    }),
};

// ── Tenants ──────────────────────────────────────────────────────

export interface Tenant {
  id: string;
  name: string;
  plan?: string;
  config?: Record<string, any>;
  usage_limit?: number;
  created_at?: string;
}

export const tenantsAPI = {
  list: async (): Promise<Tenant[]> => apiFetch('/admin/tenants'),
  getOne: async (id: string): Promise<Tenant> => apiFetch(`/admin/tenants/${id}`),
  update: async (id: string, plan?: string, config?: Record<string, any>, usageLimit?: number, name?: string) =>
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

// ── Status ───────────────────────────────────────────────────────

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
  db: DatabaseStatus;
  counts: { configs: number; credentials: number; tenants: number };
}

export const statusAPI = {
  get: async (): Promise<SystemStatus> => apiFetch('/admin/status'),
};

// ── Users ────────────────────────────────────────────────────────

export interface AdminUser {
  id: string;
  email: string;
  role: 'admin' | 'super_admin';
  enabled: boolean;
  created_at?: string;
}

export const usersAPI = {
  list: async (): Promise<AdminUser[]> => apiFetch('/admin/users'),
  create: async (email: string, password: string, role: 'admin' | 'super_admin') =>
    apiFetch('/admin/users', { method: 'POST', body: JSON.stringify({ email, password, role }) }),
  update: async (id: string, role?: 'admin' | 'super_admin', enabled?: boolean) =>
    apiFetch(`/admin/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ ...(role && { role }), ...(enabled !== undefined && { enabled }) }),
    }),
  delete: async (id: string) => apiFetch(`/admin/users/${id}`, { method: 'DELETE' }),
};

// ── Audit ────────────────────────────────────────────────────────

export interface AuditLog {
  id: string;
  timestamp: string;
  admin_id?: string;
  admin_email?: string;
  action: string;
  resource_type?: string;
  resource_id?: string;
  old_value?: any;
  new_value?: any;
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
