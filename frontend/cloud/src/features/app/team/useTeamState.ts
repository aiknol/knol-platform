'use client';

import { FormEvent, useCallback, useEffect, useState } from 'react';
import {
  AppUser,
  TenantUser,
  TenantAuditItem,
  appUsersAPI,
  appInvitesAPI,
  appAuditAPI,
  appAuthAPI,
  getAppAuthUser,
} from '@/features/app/api';
import type { InviteItem } from '@/features/app/api';

export function useTeamState() {
  const [user] = useState<AppUser | null>(getAppAuthUser());
  const [users, setUsers] = useState<TenantUser[]>([]);
  const [invites, setInvites] = useState<InviteItem[]>([]);
  const [auditLogs, setAuditLogs] = useState<TenantAuditItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const canManage = user?.role === 'owner' || user?.role === 'admin';

  // Invite form state
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState<'admin' | 'developer' | 'viewer'>('developer');
  const [inviteBusy, setInviteBusy] = useState(false);
  const [newInviteToken, setNewInviteToken] = useState<string | null>(null);

  // Direct user creation state
  const [newUserName, setNewUserName] = useState('');
  const [newUserEmail, setNewUserEmail] = useState('');
  const [newUserPassword, setNewUserPassword] = useState('');
  const [newUserRole, setNewUserRole] = useState<'admin' | 'developer' | 'read_only'>('developer');
  const [userBusy, setUserBusy] = useState(false);

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const [team, inviteList, logs] = await Promise.all([
        appUsersAPI.list(),
        appInvitesAPI.list().catch(() => [] as InviteItem[]),
        appAuditAPI.list(),
      ]);
      setUsers(team);
      setInvites(inviteList);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load team data');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (canManage) {
      load().catch(() => undefined);
    } else {
      setLoading(false);
    }
  }, [load, canManage]);

  const onInvite = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setInviteBusy(true);
    setNewInviteToken(null);
    try {
      const result = await appInvitesAPI.create({ email: inviteEmail, role: inviteRole });
      setNewInviteToken(result.token);
      setInviteEmail('');
      const [inviteList, logs] = await Promise.all([appInvitesAPI.list().catch(() => [] as InviteItem[]), appAuditAPI.list()]);
      setInvites(inviteList);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create invite');
    } finally {
      setInviteBusy(false);
    }
  };

  const onRevokeInvite = async (id: string) => {
    if (!window.confirm('Revoke this invite?')) return;
    setError('');
    try {
      await appInvitesAPI.revoke(id);
      const [inviteList, logs] = await Promise.all([appInvitesAPI.list().catch(() => [] as InviteItem[]), appAuditAPI.list()]);
      setInvites(inviteList);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke invite');
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
    setError('');
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
      // Ignore clipboard errors.
    }
  };

  return {
    user,
    users,
    invites,
    auditLogs,
    loading,
    error,
    canManage,
    // Invite form
    inviteEmail,
    setInviteEmail,
    inviteRole,
    setInviteRole,
    inviteBusy,
    newInviteToken,
    onInvite,
    onRevokeInvite,
    // Direct user creation
    newUserName,
    setNewUserName,
    newUserEmail,
    setNewUserEmail,
    newUserPassword,
    setNewUserPassword,
    newUserRole,
    setNewUserRole,
    userBusy,
    onCreateUser,
    onToggleUser,
    copyToClipboard,
  };
}
