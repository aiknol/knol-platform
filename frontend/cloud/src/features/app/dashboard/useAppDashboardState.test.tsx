import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, vi } from 'vitest';
import { useAppDashboardState } from './useAppDashboardState';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  getAppTenant: vi.fn(),
  appAuthAPI: {
    me: vi.fn(),
  },
  appApiKeysAPI: {
    list: vi.fn(),
  },
  appUsersAPI: {
    list: vi.fn(),
  },
  appBillingAPI: {
    getUsage: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  getAppAuthUser: apiMocks.getAppAuthUser,
  getAppTenant: apiMocks.getAppTenant,
  appAuthAPI: apiMocks.appAuthAPI,
  appApiKeysAPI: apiMocks.appApiKeysAPI,
  appUsersAPI: apiMocks.appUsersAPI,
  appBillingAPI: apiMocks.appBillingAPI,
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

const USAGE_RESPONSE = {
  plan: 'pro',
  ops_this_month: 42,
  ops_limit: 100000,
  usage_percentage: 0.042,
  alerts_triggered: [],
  month: '2026-02',
};

beforeEach(() => {
  vi.clearAllMocks();

  apiMocks.getAppAuthUser.mockReturnValue(null);
  apiMocks.getAppTenant.mockReturnValue(null);

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
  apiMocks.appBillingAPI.getUsage.mockResolvedValue(USAGE_RESPONSE);
});

describe('useAppDashboardState', () => {
  it('loads owner dashboard overview data', async () => {
    const { result } = renderHook(() => useAppDashboardState());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('');
    expect(result.current.canManage).toBe(true);
    expect(result.current.user?.email).toBe('owner@example.com');
    expect(result.current.tenant?.id).toBe('tenant_1');
    expect(result.current.gatewayBaseUrl).toBe('https://gateway.example.com');
    expect(result.current.keyCount).toBe(1);
    expect(result.current.teamCount).toBe(1);
    expect(result.current.usage?.ops_this_month).toBe(42);

    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appUsersAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appBillingAPI.getUsage).toHaveBeenCalledTimes(1);
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
    expect(result.current.keyCount).toBe(1);
    expect(result.current.teamCount).toBe(0);

    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appUsersAPI.list).not.toHaveBeenCalled();
  });

  it('surfaces load errors and exits loading state', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValueOnce(new Error('Failed to load dashboard'));

    const { result } = renderHook(() => useAppDashboardState());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Failed to load dashboard');
    expect(result.current.keyCount).toBe(0);
    expect(result.current.teamCount).toBe(0);
  });
});
