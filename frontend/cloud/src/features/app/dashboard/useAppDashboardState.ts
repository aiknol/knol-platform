'use client';

import { FormEvent, useCallback, useEffect, useState } from 'react';
import {
  ApiKeyItem,
  TenantAuditItem,
  TenantUser,
  AppUser,
  TenantProfile,
  appAuditAPI,
  appApiKeysAPI,
  appAuthAPI,
  appUsersAPI,
  consumeInitialApiKey,
  getAppAuthUser,
  getAppTenant,
} from '@/features/app/api';

export function useAppDashboardState() {
  const [user, setUser] = useState<AppUser | null>(getAppAuthUser());
  const [tenant, setTenant] = useState<TenantProfile | null>(getAppTenant());
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('');
  const [keys, setKeys] = useState<ApiKeyItem[]>([]);
  const [users, setUsers] = useState<TenantUser[]>([]);
  const [auditLogs, setAuditLogs] = useState<TenantAuditItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const canManage = user?.role === 'owner' || user?.role === 'admin';

  const [newName, setNewName] = useState('integration-key');
  const [newRole, setNewRole] = useState<'admin' | 'developer' | 'read_only'>('developer');
  const [newExpiryDays, setNewExpiryDays] = useState('30');
  const [createBusy, setCreateBusy] = useState(false);
  const [newlyCreatedApiKey, setNewlyCreatedApiKey] = useState<string | null>(consumeInitialApiKey());
  const [newUserName, setNewUserName] = useState('');
  const [newUserEmail, setNewUserEmail] = useState('');
  const [newUserPassword, setNewUserPassword] = useState('');
  const [newUserRole, setNewUserRole] = useState<'admin' | 'developer' | 'read_only'>('developer');
  const [userBusy, setUserBusy] = useState(false);

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const me = await appAuthAPI.me();
      setUser(me.user);
      setTenant(me.tenant);
      setGatewayBaseUrl(me.gateway_base_url || '');
      const keyList = await appApiKeysAPI.list();
      setKeys(keyList);
      if (me.user.role === 'owner' || me.user.role === 'admin') {
        const [team, logs] = await Promise.all([appUsersAPI.list(), appAuditAPI.list()]);
        setUsers(team);
        setAuditLogs(logs);
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

  const onCreateKey = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setCreateBusy(true);
    setError('');

    try {
      const expires = Number(newExpiryDays);
      const payload: { name: string; role: 'admin' | 'developer' | 'read_only'; expires_in_days?: number } = {
        name: newName,
        role: newRole,
      };
      if (Number.isFinite(expires) && expires > 0) {
        payload.expires_in_days = expires;
      }

      const created = await appApiKeysAPI.create(payload);
      setNewlyCreatedApiKey(created.api_key);
      const keyList = await appApiKeysAPI.list();
      setKeys(keyList);
      if (canManage) {
        const logs = await appAuditAPI.list();
        setAuditLogs(logs);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setCreateBusy(false);
    }
  };

  const onRevoke = async (id: string) => {
    if (!window.confirm('Revoke this API key? Existing integrations using it will stop working.')) {
      return;
    }

    try {
      await appApiKeysAPI.revoke(id);
      const keyList = await appApiKeysAPI.list();
      setKeys(keyList);
      if (canManage) {
        const logs = await appAuditAPI.list();
        setAuditLogs(logs);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke API key');
    }
  };

  const onCreateUser = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setUserBusy(true);
    try {
      await appUsersAPI.create({
        full_name: newUserName,
        email: newUserEmail,
        password: newUserPassword,
        role: newUserRole,
      });
      setNewUserName('');
      setNewUserEmail('');
      setNewUserPassword('');
      const [team, logs] = await Promise.all([appUsersAPI.list(), appAuditAPI.list()]);
      setUsers(team);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create user');
    } finally {
      setUserBusy(false);
    }
  };

  const onToggleUser = async (member: TenantUser) => {
    try {
      await appUsersAPI.update(member.id, { enabled: !member.enabled });
      const [team, logs] = await Promise.all([appUsersAPI.list(), appAuditAPI.list()]);
      setUsers(team);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user');
    }
  };

  const copyToClipboard = async (value: string) => {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // Ignore clipboard errors in unsupported browsers.
    }
  };

  return {
    user,
    tenant,
    gatewayBaseUrl,
    keys,
    users,
    auditLogs,
    loading,
    error,
    canManage,
    newName,
    setNewName,
    newRole,
    setNewRole,
    newExpiryDays,
    setNewExpiryDays,
    createBusy,
    newlyCreatedApiKey,
    newUserName,
    setNewUserName,
    newUserEmail,
    setNewUserEmail,
    newUserPassword,
    setNewUserPassword,
    newUserRole,
    setNewUserRole,
    userBusy,
    onCreateKey,
    onRevoke,
    onCreateUser,
    onToggleUser,
    copyToClipboard,
  };
}
