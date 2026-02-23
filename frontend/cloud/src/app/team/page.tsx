'use client';

import { useTeamState } from '@/features/app/team/useTeamState';
import PageHeader from '@/components/PageHeader';
import StatusBadge from '@/components/StatusBadge';
import EmptyState from '@/components/EmptyState';

export default function TeamPage() {
  const {
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
  } = useTeamState();

  if (!canManage) {
    return (
      <div className="py-12 text-center">
        <p className="text-dark-400">You don&apos;t have permission to manage the team.</p>
      </div>
    );
  }

  if (loading) {
    return <p className="text-dark-300 py-8">Loading team...</p>;
  }

  const pendingInvites = invites.filter((i) => i.status === 'pending');
  const otherInvites = invites.filter((i) => i.status !== 'pending');

  return (
    <div className="space-y-8">
      <PageHeader title="Team" description="Manage team members, invitations, and view audit activity." />

      {error && <div className="alert-error">{error}</div>}

      {/* Invite token banner */}
      {newInviteToken && (
        <section className="alert-success !p-5">
          <h3 className="text-sm font-semibold text-emerald-200 mb-2">Invite Created</h3>
          <p className="text-sm text-emerald-300 mb-2">Share this token with the invitee. It expires in 7 days.</p>
          <div className="flex flex-col md:flex-row gap-3 md:items-center">
            <code className="text-xs md:text-sm break-all text-emerald-100">{newInviteToken}</code>
            <button
              onClick={() => copyToClipboard(newInviteToken)}
              className="px-3 py-1.5 rounded-lg border border-emerald-400/40 text-emerald-100 text-sm hover:bg-emerald-500/10 shrink-0"
            >
              Copy
            </button>
          </div>
        </section>
      )}

      {/* Invite + Add User side by side */}
      <section className="grid gap-6 md:grid-cols-2">
        {/* Invite by email */}
        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Invite by Email</h3>
          <form onSubmit={onInvite} className="space-y-4">
            <div>
              <label htmlFor="invite-email" className="form-label">Email Address</label>
              <input
                id="invite-email"
                type="email"
                value={inviteEmail}
                onChange={(e) => setInviteEmail(e.target.value)}
                placeholder="colleague@company.com"
                className="input-field"
                required
              />
            </div>
            <div>
              <label htmlFor="invite-role" className="form-label">Role</label>
              <select
                id="invite-role"
                value={inviteRole}
                onChange={(e) => setInviteRole(e.target.value as 'admin' | 'developer' | 'viewer')}
                className="input-field"
              >
                <option value="developer">Developer</option>
                <option value="admin">Admin</option>
                <option value="viewer">Viewer</option>
              </select>
            </div>
            <button disabled={inviteBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
              {inviteBusy ? 'Sending...' : 'Send Invite'}
            </button>
          </form>
        </article>

        {/* Direct user creation */}
        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Add User Directly</h3>
          <form onSubmit={onCreateUser} className="space-y-3">
            <input
              value={newUserName}
              onChange={(e) => setNewUserName(e.target.value)}
              placeholder="Full name"
              className="input-field"
              required
            />
            <input
              value={newUserEmail}
              onChange={(e) => setNewUserEmail(e.target.value)}
              placeholder="user@company.com"
              type="email"
              className="input-field"
              required
            />
            <input
              value={newUserPassword}
              onChange={(e) => setNewUserPassword(e.target.value)}
              placeholder="Temporary password (min 12 chars)"
              type="password"
              minLength={12}
              className="input-field"
              required
            />
            <select
              value={newUserRole}
              onChange={(e) => setNewUserRole(e.target.value as 'admin' | 'developer' | 'read_only')}
              className="input-field"
            >
              <option value="developer">Developer</option>
              <option value="admin">Admin</option>
              <option value="read_only">Read only</option>
            </select>
            <button disabled={userBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
              {userBusy ? 'Creating...' : 'Add User'}
            </button>
          </form>
        </article>
      </section>

      {/* Pending Invites */}
      {pendingInvites.length > 0 && (
        <section className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">
            Pending Invites <span className="text-dark-400 font-normal text-sm">({pendingInvites.length})</span>
          </h3>
          <div className="space-y-2">
            {pendingInvites.map((inv) => (
              <div
                key={inv.id}
                className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2"
              >
                <div>
                  <p className="text-sm text-dark-100">{inv.email}</p>
                  <p className="text-xs text-dark-400">
                    Role: {inv.role} | Expires: {new Date(inv.expires_at).toLocaleDateString()}
                  </p>
                </div>
                <button
                  onClick={() => onRevokeInvite(inv.id)}
                  className="px-3 py-1.5 rounded-lg border border-red-500/40 text-red-300 text-sm hover:bg-red-500/10 shrink-0"
                >
                  Revoke
                </button>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Team Members */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">
          Team Members <span className="text-dark-400 font-normal text-sm">({users.length})</span>
        </h3>
        {users.length === 0 ? (
          <EmptyState message="No team members yet." />
        ) : (
          <div className="space-y-2">
            {users.map((member) => (
              <div
                key={member.id}
                className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <p className="text-sm text-dark-100 font-medium truncate">{member.full_name}</p>
                    <StatusBadge status={member.role} />
                    <StatusBadge status={member.enabled ? 'enabled' : 'disabled'} label={member.enabled ? 'Active' : 'Disabled'} />
                  </div>
                  <p className="text-xs text-dark-400 truncate">
                    {member.email}
                    {member.last_login_at && ` | Last login: ${new Date(member.last_login_at).toLocaleDateString()}`}
                  </p>
                </div>
                {member.id !== user?.id && (
                  <button
                    onClick={() => onToggleUser(member)}
                    className="px-3 py-1.5 rounded-lg border border-dark-600/40 text-sm text-dark-200 hover:bg-dark-800/60 shrink-0"
                  >
                    {member.enabled ? 'Disable' : 'Enable'}
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Audit Log */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">Audit Log</h3>
        <div className="space-y-2 max-h-96 overflow-auto pr-1">
          {auditLogs.length === 0 ? (
            <EmptyState message="No audit entries yet." />
          ) : (
            auditLogs.map((log) => (
              <div key={log.id} className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-3">
                <p className="text-sm text-dark-100">
                  {log.action} {log.resource_type}
                  {log.resource_key ? ` (${log.resource_key})` : ''}
                </p>
                <p className="text-xs text-dark-400">
                  {log.app_user_email || 'system'} | {new Date(log.created_at).toLocaleString()}
                </p>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
