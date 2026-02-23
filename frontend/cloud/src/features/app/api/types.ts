export interface AppUser {
  id: string;
  email: string;
  full_name?: string;
  role: 'owner' | 'admin' | 'developer' | 'read_only' | string;
  tenant_id: string;
  email_verified?: boolean;
  totp_enabled?: boolean;
}

export interface TenantProfile {
  id: string;
  name: string;
  slug: string;
  plan: string;
  usage_ops_month: number;
  usage_limit?: number | null;
}

export interface ApiKeyItem {
  id: string;
  name: string;
  role: string;
  active: boolean;
  last_used_at?: string | null;
  expires_at?: string | null;
  created_at: string;
}

export interface TenantUser {
  id: string;
  email: string;
  full_name: string;
  role: 'owner' | 'admin' | 'developer' | 'read_only' | string;
  enabled: boolean;
  last_login_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface TenantAuditItem {
  id: string;
  app_user_email?: string | null;
  action: string;
  resource_type: string;
  resource_key?: string | null;
  old_value?: unknown;
  new_value?: unknown;
  metadata?: unknown;
  created_at: string;
}

export interface InviteItem {
  id: string;
  email: string;
  role: string;
  status: 'pending' | 'accepted' | 'revoked' | 'expired';
  expires_at: string;
  created_at: string;
}

export interface UsageHistoryItem {
  month: string;
  ops_count: number;
  plan: string;
  usage_limit: number | null;
}
