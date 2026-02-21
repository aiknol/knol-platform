'use client';

import { useAppDashboardState } from '@/features/app/dashboard/useAppDashboardState';

export default function AppDashboardPage() {
  const {
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
  } = useAppDashboardState();

  if (loading) {
    return <p className="text-dark-300">Loading dashboard...</p>;
  }

  return (
    <div className="space-y-8">
      {error && (
        <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <section className="grid gap-4 md:grid-cols-3">
        <article className="card">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Workspace</p>
          <h2 className="text-lg font-semibold text-dark-50">{tenant?.name}</h2>
          <p className="text-sm text-dark-300 mt-2">Slug: <code>{tenant?.slug}</code></p>
          <p className="text-sm text-dark-300">Plan: <span className="capitalize">{tenant?.plan}</span></p>
        </article>

        <article className="card">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Usage</p>
          <h2 className="text-2xl font-semibold text-dark-50">{tenant?.usage_ops_month ?? 0}</h2>
          <p className="text-sm text-dark-300 mt-2">Ops this month</p>
          <p className="text-sm text-dark-300">
            Limit: {tenant?.usage_limit ?? 'Unlimited'}
          </p>
        </article>

        <article className="card">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Owner</p>
          <h2 className="text-lg font-semibold text-dark-50">{user?.email}</h2>
          <p className="text-sm text-dark-300 mt-2 capitalize">Role: {user?.role}</p>
          <p className="text-sm text-dark-300">Gateway: <code>{gatewayBaseUrl || 'Not configured'}</code></p>
        </article>
      </section>

      {newlyCreatedApiKey && (
        <section className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-5">
          <h3 className="text-sm font-semibold text-emerald-200 mb-2">Your API key (shown once)</h3>
          <div className="flex flex-col md:flex-row gap-3 md:items-center">
            <code className="text-xs md:text-sm break-all text-emerald-100">{newlyCreatedApiKey}</code>
            <button
              onClick={() => copyToClipboard(newlyCreatedApiKey)}
              className="px-3 py-1.5 rounded-lg border border-emerald-400/40 text-emerald-100 text-sm"
            >
              Copy
            </button>
          </div>
        </section>
      )}

      <section className="grid gap-6 md:grid-cols-2">
        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Create API key</h3>
          <form onSubmit={onCreateKey} className="space-y-4">
            <div>
              <label htmlFor="name" className="block text-sm text-dark-300 mb-2">Name</label>
              <input
                id="name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
                required
              />
            </div>

            <div>
              <label htmlFor="role" className="block text-sm text-dark-300 mb-2">Role</label>
              <select
                id="role"
                value={newRole}
                onChange={(e) => setNewRole(e.target.value as 'admin' | 'developer' | 'read_only')}
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              >
                <option value="developer">Developer</option>
                <option value="admin">Admin</option>
                <option value="read_only">Read only</option>
              </select>
            </div>

            <div>
              <label htmlFor="expiry" className="block text-sm text-dark-300 mb-2">Expires in days (optional)</label>
              <input
                id="expiry"
                type="number"
                min={1}
                value={newExpiryDays}
                onChange={(e) => setNewExpiryDays(e.target.value)}
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              />
            </div>

            <button disabled={createBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
              {createBusy ? 'Creating...' : 'Create Key'}
            </button>
          </form>
        </article>

        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Quick integration</h3>
          <p className="text-sm text-dark-300 mb-3">Use your key against the Knol gateway:</p>
          <pre className="code-block text-xs">{`curl -X POST ${gatewayBaseUrl || '<GATEWAY_URL>'}/v1/memory \\
  -H "Authorization: Bearer <YOUR_API_KEY>" \\
  -H "Content-Type: application/json" \\
  -d '{"user_id":"user-1","text":"User likes concise replies"}'`}</pre>
        </article>
      </section>

      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">API keys</h3>
        {keys.length === 0 ? (
          <p className="text-sm text-dark-400">No keys found.</p>
        ) : (
          <div className="space-y-3">
            {keys.map((k) => (
              <div key={k.id} className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-4 flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                <div>
                  <p className="text-sm text-dark-100 font-medium">{k.name}</p>
                  <p className="text-xs text-dark-400">
                    Role: {k.role} | Active: {k.active ? 'Yes' : 'No'} | Created: {new Date(k.created_at).toLocaleString()}
                  </p>
                </div>
                {k.active && (
                  <button
                    onClick={() => onRevoke(k.id)}
                    className="px-3 py-1.5 rounded-lg border border-red-500/40 text-red-300 text-sm"
                  >
                    Revoke
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      {canManage && (
        <section className="grid gap-6 md:grid-cols-2">
          <article className="card">
            <h3 className="text-lg font-semibold text-dark-50 mb-4">Team users</h3>
            <form onSubmit={onCreateUser} className="space-y-3 mb-5">
              <input
                value={newUserName}
                onChange={(e) => setNewUserName(e.target.value)}
                placeholder="Full name"
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
                required
              />
              <input
                value={newUserEmail}
                onChange={(e) => setNewUserEmail(e.target.value)}
                placeholder="user@company.com"
                type="email"
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
                required
              />
              <input
                value={newUserPassword}
                onChange={(e) => setNewUserPassword(e.target.value)}
                placeholder="Temporary password (min 10 chars)"
                type="password"
                minLength={10}
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
                required
              />
              <select
                value={newUserRole}
                onChange={(e) => setNewUserRole(e.target.value as 'admin' | 'developer' | 'read_only')}
                className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              >
                <option value="developer">Developer</option>
                <option value="admin">Admin</option>
                <option value="read_only">Read only</option>
              </select>
              <button disabled={userBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
                {userBusy ? 'Creating user...' : 'Add User'}
              </button>
            </form>

            <div className="space-y-2">
              {users.length === 0 ? (
                <p className="text-sm text-dark-400">No tenant users.</p>
              ) : (
                users.map((member) => (
                  <div key={member.id} className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-3 flex items-center justify-between gap-2">
                    <div>
                      <p className="text-sm text-dark-100">{member.full_name} <span className="text-dark-400">({member.email})</span></p>
                      <p className="text-xs text-dark-400">Role: {member.role} | Enabled: {member.enabled ? 'Yes' : 'No'}</p>
                    </div>
                    {member.id !== user?.id && (
                      <button
                        onClick={() => onToggleUser(member)}
                        className="px-2.5 py-1.5 rounded-lg border border-dark-600/40 text-sm text-dark-200 hover:bg-dark-800/60"
                      >
                        {member.enabled ? 'Disable' : 'Enable'}
                      </button>
                    )}
                  </div>
                ))
              )}
            </div>
          </article>

          <article className="card">
            <h3 className="text-lg font-semibold text-dark-50 mb-4">Tenant audit log</h3>
            <div className="space-y-2 max-h-96 overflow-auto pr-1">
              {auditLogs.length === 0 ? (
                <p className="text-sm text-dark-400">No audit entries yet.</p>
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
          </article>
        </section>
      )}
    </div>
  );
}
