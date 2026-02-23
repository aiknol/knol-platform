'use client';

import { useCallback, useEffect, useState } from 'react';
import {
  AppUser,
  TenantProfile,
  appAuthAPI,
  appApiKeysAPI,
  appUsersAPI,
  appBillingAPI,
  getAppAuthUser,
  getAppTenant,
} from '@/features/app/api';
import type { UsageInfo } from '@/features/app/api';

export function useAppDashboardState() {
  const [user, setUser] = useState<AppUser | null>(getAppAuthUser());
  const [tenant, setTenant] = useState<TenantProfile | null>(getAppTenant());
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('');
  const [usage, setUsage] = useState<UsageInfo | null>(null);
  const [keyCount, setKeyCount] = useState(0);
  const [teamCount, setTeamCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const canManage = user?.role === 'owner' || user?.role === 'admin';

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const [me, keyList, usageData] = await Promise.all([
        appAuthAPI.me(),
        appApiKeysAPI.list(),
        appBillingAPI.getUsage().catch(() => null),
      ]);
      setUser(me.user);
      setTenant(me.tenant);
      setGatewayBaseUrl(me.gateway_base_url || '');
      setKeyCount(keyList.length);
      setUsage(usageData);

      if (me.user.role === 'owner' || me.user.role === 'admin') {
        const team = await appUsersAPI.list();
        setTeamCount(team.length);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load dashboard');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load().catch(() => undefined);
  }, [load]);

  return {
    user,
    tenant,
    gatewayBaseUrl,
    usage,
    keyCount,
    teamCount,
    loading,
    error,
    canManage,
  };
}
