// ── Admin panel configuration ───────────────────────────────────

export const ADMIN_NAV_ITEMS = [
  { href: '/dashboard', label: 'Dashboard', icon: '📊' },
  { href: '/services', label: 'Services', icon: '🛰️' },
  { href: '/config', label: 'Config', icon: '⚙️' },
  { href: '/credentials', label: 'Credentials', icon: '🔑' },
  { href: '/tenants', label: 'Tenants', icon: '👥' },
  { href: '/campaigns', label: 'Campaigns', icon: '📢' },
  { href: '/marketing', label: 'Marketing Analytics', icon: '📈' },
  { href: '/users', label: 'Admin Users', icon: '👨‍💼' },
  { href: '/audit', label: 'Audit Log', icon: '📋' },
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
  'graph',
  'search',
  'resilience',
  'marketing',
  'demo',
] as const;

export type ConfigCategory = (typeof CONFIG_CATEGORIES)[number];

export const AUDIT_ACTIONS = ['create', 'update', 'delete', 'login', 'logout', 'test', 'trigger'] as const;
export const AUDIT_RESOURCE_TYPES = ['config', 'credential', 'campaign', 'tenant', 'user', 'admin_user', 'webhook'] as const;
