// ── Admin panel configuration ───────────────────────────────────

export const ADMIN_NAV_ITEMS = [
  { href: '/admin', label: 'Dashboard', icon: '📊' },
  { href: '/admin/config', label: 'Config', icon: '⚙️' },
  { href: '/admin/credentials', label: 'Credentials', icon: '🔑' },
  { href: '/admin/tenants', label: 'Tenants', icon: '👥' },
  { href: '/admin/campaigns', label: 'Campaigns', icon: '📢' },
  { href: '/admin/marketing', label: 'Marketing Analytics', icon: '📈' },
  { href: '/admin/users', label: 'Admin Users', icon: '👨‍💼' },
  { href: '/admin/audit', label: 'Audit Log', icon: '📋' },
] as const;

export const CONFIG_CATEGORIES = [
  'services',
  'storage',
  'database',
  'consolidation',
  'conflict',
  'decay',
  'retention',
  'gateway',
  'llm',
  'guardrails',
  'grounding',
  'webhooks',
  'embedding',
  'marketing',
  'demo',
] as const;

export type ConfigCategory = (typeof CONFIG_CATEGORIES)[number];

export const AUDIT_ACTIONS = ['create', 'update', 'delete', 'login', 'logout', 'test', 'trigger'] as const;
export const AUDIT_RESOURCE_TYPES = ['config', 'credential', 'campaign', 'tenant', 'user', 'admin_user', 'webhook'] as const;
