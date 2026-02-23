'use client';

import { useCallback, useEffect, useState } from 'react';
import {
  AppUser,
  TenantProfile,
  appAuthAPI,
  appSettingsAPI,
  getAppAuthUser,
  getAppTenant,
  setAppProfile,
} from '@/features/app/api';

export function useSettingsState() {
  const [user, setUser] = useState<AppUser | null>(getAppAuthUser());
  const [tenant, setTenant] = useState<TenantProfile | null>(getAppTenant());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  const canManage = user?.role === 'owner' || user?.role === 'admin';

  // Workspace form
  const [workspaceName, setWorkspaceName] = useState('');
  const [workspaceBusy, setWorkspaceBusy] = useState(false);

  // Profile form
  const [fullName, setFullName] = useState('');
  const [profileBusy, setProfileBusy] = useState(false);

  // Password form
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordBusy, setPasswordBusy] = useState(false);

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const me = await appAuthAPI.me();
      setUser(me.user);
      setTenant(me.tenant);
      setWorkspaceName(me.tenant.name);
      setFullName(me.user.full_name || '');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load().catch(() => undefined);
  }, [load]);

  const onSaveWorkspace = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');
    setWorkspaceBusy(true);
    try {
      const result = await appSettingsAPI.updateTenant({ name: workspaceName });
      setWorkspaceName(result.name);
      // Refresh session data
      const me = await appAuthAPI.me();
      setTenant(me.tenant);
      setUser(me.user);
      // Persist to sessionStorage so AppShell header reflects the change
      setAppProfile(me.user, me.tenant);
      setSuccess('Workspace name updated.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update workspace');
    } finally {
      setWorkspaceBusy(false);
    }
  };

  const onSaveProfile = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');
    setProfileBusy(true);
    try {
      await appSettingsAPI.updateProfile({ full_name: fullName });
      // Refresh session data
      const me = await appAuthAPI.me();
      setUser(me.user);
      setTenant(me.tenant);
      // Persist to sessionStorage so AppShell header reflects the change
      setAppProfile(me.user, me.tenant);
      setSuccess('Profile updated.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update profile');
    } finally {
      setProfileBusy(false);
    }
  };

  const onChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');

    if (newPassword !== confirmPassword) {
      setError('New passwords do not match.');
      return;
    }
    if (newPassword.length < 12) {
      setError('Password must be at least 12 characters.');
      return;
    }
    if (!/[A-Z]/.test(newPassword)) {
      setError('Password must include an uppercase letter.');
      return;
    }
    if (!/[a-z]/.test(newPassword)) {
      setError('Password must include a lowercase letter.');
      return;
    }
    if (!/[0-9]/.test(newPassword)) {
      setError('Password must include a digit.');
      return;
    }
    if (!/[^A-Za-z0-9]/.test(newPassword)) {
      setError('Password must include a special character.');
      return;
    }

    setPasswordBusy(true);
    try {
      await appSettingsAPI.changePassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      // Refresh session after password change (new token was set via cookie)
      const me = await appAuthAPI.me();
      setUser(me.user);
      setTenant(me.tenant);
      setAppProfile(me.user, me.tenant);
      setSuccess('Password changed. All other sessions have been invalidated.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setPasswordBusy(false);
    }
  };

  return {
    user,
    tenant,
    loading,
    error,
    success,
    canManage,
    // Workspace
    workspaceName,
    setWorkspaceName,
    workspaceBusy,
    onSaveWorkspace,
    // Profile
    fullName,
    setFullName,
    profileBusy,
    onSaveProfile,
    // Password
    currentPassword,
    setCurrentPassword,
    newPassword,
    setNewPassword,
    confirmPassword,
    setConfirmPassword,
    passwordBusy,
    onChangePassword,
  };
}
