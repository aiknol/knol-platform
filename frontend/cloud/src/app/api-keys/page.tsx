'use client';

import { useApiKeysState } from '@/features/app/apikeys/useApiKeysState';
import PageHeader from '@/components/PageHeader';
import StatusBadge from '@/components/StatusBadge';

export default function ApiKeysPage() {
  const {
    gatewayBaseUrl,
    keys,
    loading,
    error,
    newName,
    setNewName,
    newRole,
    setNewRole,
    newExpiryDays,
    setNewExpiryDays,
    createBusy,
    newlyCreatedApiKey,
    onCreateKey,
    onRevoke,
    copyToClipboard,
  } = useApiKeysState();

  if (loading) {
    return <p className="text-dark-300 py-8">Loading API keys...</p>;
  }

  return (
    <div className="space-y-8">
      <PageHeader title="API Keys" description="Create and manage API keys for your integrations." />

      {error && <div className="alert-error">{error}</div>}

      {/* Newly created key banner */}
      {newlyCreatedApiKey && (
        <section className="alert-success !p-5">
          <h3 className="text-sm font-semibold text-emerald-200 mb-2">Your API key (shown once)</h3>
          <div className="flex flex-col md:flex-row gap-3 md:items-center">
            <code className="text-xs md:text-sm break-all text-emerald-100">{newlyCreatedApiKey}</code>
            <button
              onClick={() => copyToClipboard(newlyCreatedApiKey)}
              className="px-3 py-1.5 rounded-lg border border-emerald-400/40 text-emerald-100 text-sm hover:bg-emerald-500/10"
            >
              Copy
            </button>
          </div>
        </section>
      )}

      {/* Create + Integration side by side */}
      <section className="grid gap-6 md:grid-cols-2">
        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Create API Key</h3>
          <form onSubmit={onCreateKey} className="space-y-4">
            <div>
              <label htmlFor="key-name" className="form-label">Name</label>
              <input
                id="key-name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="input-field"
                required
              />
            </div>

            <div>
              <label htmlFor="key-role" className="form-label">Role</label>
              <select
                id="key-role"
                value={newRole}
                onChange={(e) => setNewRole(e.target.value as 'admin' | 'developer' | 'read_only')}
                className="input-field"
              >
                <option value="developer">Developer</option>
                <option value="admin">Admin</option>
                <option value="read_only">Read only</option>
              </select>
            </div>

            <div>
              <label htmlFor="key-expiry" className="form-label">Expires in days (optional)</label>
              <input
                id="key-expiry"
                type="number"
                min={1}
                value={newExpiryDays}
                onChange={(e) => setNewExpiryDays(e.target.value)}
                className="input-field"
              />
            </div>

            <button disabled={createBusy} className="btn-primary !py-2.5 disabled:opacity-60" type="submit">
              {createBusy ? 'Creating...' : 'Create Key'}
            </button>
          </form>
        </article>

        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Quick Integration</h3>
          <p className="text-sm text-dark-300 mb-3">Use your key against the Knol gateway:</p>
          <pre className="code-block text-xs">{`curl -X POST ${gatewayBaseUrl || '<GATEWAY_URL>'}/v1/memory \\
  -H "Authorization: Bearer <YOUR_API_KEY>" \\
  -H "Content-Type: application/json" \\
  -d '{"user_id":"user-1","content":"User likes concise replies"}'`}</pre>
        </article>
      </section>

      {/* Keys list */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">
          Your Keys {keys.length > 0 && <span className="text-dark-400 font-normal text-sm">({keys.length})</span>}
        </h3>
        {keys.length === 0 ? (
          <p className="text-sm text-dark-400 py-4">No API keys found. Create one above to get started.</p>
        ) : (
          <div className="space-y-3">
            {keys.map((k) => (
              <div
                key={k.id}
                className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-4 flex flex-col md:flex-row md:items-center md:justify-between gap-3"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <p className="text-sm text-dark-100 font-medium">{k.name}</p>
                    <StatusBadge status={k.role} />
                    <StatusBadge status={k.active ? 'enabled' : 'disabled'} label={k.active ? 'Active' : 'Revoked'} />
                  </div>
                  <p className="text-xs text-dark-400">
                    Created: {new Date(k.created_at).toLocaleDateString()}
                    {k.last_used_at && ` | Last used: ${new Date(k.last_used_at).toLocaleDateString()}`}
                    {k.expires_at && ` | Expires: ${new Date(k.expires_at).toLocaleDateString()}`}
                  </p>
                </div>
                {k.active && (
                  <button
                    onClick={() => onRevoke(k.id)}
                    className="px-3 py-1.5 rounded-lg border border-red-500/40 text-red-300 text-sm hover:bg-red-500/10 shrink-0"
                  >
                    Revoke
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
