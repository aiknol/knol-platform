import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, vi } from 'vitest';
import { useAppDashboardState } from './useAppDashboardState';

const apiMocks = vi.hoisted(() => ({
  consumeInitialApiKey: vi.fn<() => string | null>(),
  getAppAuthUser: vi.fn(),
  getAppTenant: vi.fn(),
  appAuthAPI: {
    me: vi.fn(),
  },
  appApiKeysAPI: {
    list: vi.fn(),
    create: vi.fn(),
    revoke: vi.fn(),
  },
  appUsersAPI: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
  },
  appAuditAPI: {
    list: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  consumeInitialApiKey: apiMocks.consumeInitialApiKey,
  getAppAuthUser: apiMocks.getAppAuthUser,
  getAppTenant: apiMocks.getAppTenant,
  appAuthAPI: apiMocks.appAuthAPI,
  appApiKeysAPI: apiMocks.appApiKeysAPI,
  appUsersAPI: apiMocks.appUsersAPI,
  appAuditAPI: apiMocks.appAuditAPI,
}));

const OWNER_ME_RESPONSE = {
  user: {
    id: 'user_1',
    email: 'owner@example.com',
    role: 'owner',
    tenant_id: 'tenant_1',
  },
  tenant: {
    id: 'tenant_1',
    name: 'Acme',
    slug: 'acme',
    plan: 'pro',
    usage_ops_month: 12,
  },
  gateway_base_url: 'https://gateway.example.com',
};

const DEVELOPER_ME_RESPONSE = {
  ...OWNER_ME_RESPONSE,
  user: {
    ...OWNER_ME_RESPONSE.user,
    role: 'developer',
  },
};

beforeEach(() => {
  vi.clearAllMocks();

  apiMocks.getAppAuthUser.mockReturnValue(null);
  apiMocks.getAppTenant.mockReturnValue(null);
  apiMocks.consumeInitialApiKey.mockReturnValue('initial-key');

  apiMocks.appAuthAPI.me.mockResolvedValue(OWNER_ME_RESPONSE);
  apiMocks.appApiKeysAPI.list.mockResolvedValue([
    {
      id: 'key_1',
      name: 'integration-key',
      role: 'developer',
      active: true,
      created_at: '2026-02-20T00:00:00Z',
    },
  ]);
  apiMocks.appUsersAPI.list.mockResolvedValue([
    {
      id: 'member_1',
      email: 'member@example.com',
      full_name: 'Member One',
      role: 'developer',
      enabled: true,
      created_at: '2026-02-20T00:00:00Z',
      updated_at: '2026-02-20T00:00:00Z',
    },
  ]);
  apiMocks.appAuditAPI.list.mockResolvedValue([
    {
      id: 'audit_1',
      action: 'api_key.create',
      resource_type: 'api_key',
      created_at: '2026-02-20T00:00:00Z',
    },
  ]);
});

describe('useAppDashboardState', () => {
  it('loads owner dashboard data and enables management capabilities', async () => {
    const { result } = renderHook(() => useAppDashboardState());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('');
    expect(result.current.canManage).toBe(true);
    expect(result.current.user?.email).toBe('owner@example.com');
    expect(result.current.tenant?.id).toBe('tenant_1');
    expect(result.current.gatewayBaseUrl).toBe('https://gateway.example.com');
    expect(result.current.keys).toHaveLength(1);
    expect(result.current.users).toHaveLength(1);
    expect(result.current.auditLogs).toHaveLength(1);
    expect(result.current.newlyCreatedApiKey).toBe('initial-key');

    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appUsersAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appAuditAPI.list).toHaveBeenCalledTimes(1);
  });

  it('loads limited data for non-admin roles', async () => {
    apiMocks.appAuthAPI.me.mockResolvedValueOnce(DEVELOPER_ME_RESPONSE);

    const { result } = renderHook(() => useAppDashboardState());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('');
    expect(result.current.canManage).toBe(false);
    expect(result.current.user?.role).toBe('developer');
    expect(result.current.keys).toHaveLength(1);
    expect(result.current.users).toHaveLength(0);
    expect(result.current.auditLogs).toHaveLength(0);

    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appUsersAPI.list).not.toHaveBeenCalled();
    expect(apiMocks.appAuditAPI.list).not.toHaveBeenCalled();
  });

  it('surfaces load errors and exits loading state', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValueOnce(new Error('Failed to load dashboard'));

    const { result } = renderHook(() => useAppDashboardState());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Failed to load dashboard');
    expect(result.current.keys).toHaveLength(0);
    expect(result.current.users).toHaveLength(0);
    expect(result.current.auditLogs).toHaveLength(0);
  });
});
