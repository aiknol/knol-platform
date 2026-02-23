'use client';

import { useSettingsState } from '@/features/app/settings/useSettingsState';
import PageHeader from '@/components/PageHeader';

export default function SettingsPage() {
  const {
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
  } = useSettingsState();

  if (loading) {
    return <p className="text-dark-300 py-8">Loading settings...</p>;
  }

  return (
    <div className="space-y-8">
      <PageHeader title="Settings" description="Manage your workspace, profile, and security settings." />

      {error && <div className="alert-error">{error}</div>}
      {success && <div className="alert-success">{success}</div>}

      {/* Workspace Settings */}
      {canManage && (
        <section className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Workspace</h3>
          <form onSubmit={onSaveWorkspace} className="space-y-4 max-w-md">
            <div>
              <label htmlFor="ws-name" className="form-label">Workspace Name</label>
              <input
                id="ws-name"
                value={workspaceName}
                onChange={(e) => setWorkspaceName(e.target.value)}
                className="input-field"
                minLength={2}
                maxLength={100}
                required
              />
            </div>
            <div>
              <label className="form-label">Slug</label>
              <p className="text-sm text-dark-400">
                <code className="bg-dark-900/70 px-2 py-0.5 rounded text-dark-300">{tenant?.slug}</code>
                <span className="ml-2 text-xs text-dark-500">(cannot be changed)</span>
              </p>
            </div>
            <button disabled={workspaceBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
              {workspaceBusy ? 'Saving...' : 'Save Workspace'}
            </button>
          </form>
        </section>
      )}

      {/* Profile Settings */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">Profile</h3>
        <form onSubmit={onSaveProfile} className="space-y-4 max-w-md">
          <div>
            <label htmlFor="profile-name" className="form-label">Full Name</label>
            <input
              id="profile-name"
              value={fullName}
              onChange={(e) => setFullName(e.target.value)}
              className="input-field"
              minLength={2}
              maxLength={100}
              autoComplete="name"
              required
            />
          </div>
          <div>
            <label className="form-label">Email</label>
            <p className="text-sm text-dark-400">
              <code className="bg-dark-900/70 px-2 py-0.5 rounded text-dark-300">{user?.email}</code>
              <span className="ml-2 text-xs text-dark-500">(cannot be changed)</span>
            </p>
          </div>
          <div>
            <label className="form-label">Role</label>
            <p className="text-sm text-dark-400 capitalize">{user?.role?.replace(/_/g, ' ')}</p>
          </div>
          <button disabled={profileBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
            {profileBusy ? 'Saving...' : 'Save Profile'}
          </button>
        </form>
      </section>

      {/* Change Password */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">Change Password</h3>
        <form onSubmit={onChangePassword} className="space-y-4 max-w-md">
          <div>
            <label htmlFor="pw-current" className="form-label">Current Password</label>
            <input
              id="pw-current"
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              className="input-field"
              maxLength={128}
              autoComplete="current-password"
              required
            />
          </div>
          <div>
            <label htmlFor="pw-new" className="form-label">New Password</label>
            <input
              id="pw-new"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              className="input-field"
              minLength={12}
              maxLength={128}
              autoComplete="new-password"
              required
            />
            <p className="text-xs text-dark-500 mt-1">
              Min 12 characters. Must include uppercase, lowercase, digit, and special character.
            </p>
          </div>
          <div>
            <label htmlFor="pw-confirm" className="form-label">Confirm New Password</label>
            <input
              id="pw-confirm"
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="input-field"
              minLength={12}
              maxLength={128}
              autoComplete="new-password"
              required
            />
          </div>
          <button disabled={passwordBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
            {passwordBusy ? 'Changing...' : 'Change Password'}
          </button>
        </form>
      </section>
    </div>
  );
}
