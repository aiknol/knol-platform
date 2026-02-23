'use client';

import Link from 'next/link';
import { useAppDashboardState } from '@/features/app/dashboard/useAppDashboardState';
import PageHeader from '@/components/PageHeader';
import StatusBadge from '@/components/StatusBadge';
import UsageBar from '@/components/UsageBar';

export default function AppDashboardPage() {
  const {
    user,
    tenant,
    gatewayBaseUrl,
    usage,
    keyCount,
    teamCount,
    loading,
    error,
    canManage,
  } = useAppDashboardState();

  if (loading) {
    return <p className="text-dark-300 py-8">Loading dashboard...</p>;
  }

  return (
    <div className="space-y-8">
      <PageHeader title="Overview" description="Your workspace at a glance." />

      {error && <div className="alert-error">{error}</div>}

      {/* Stats Cards */}
      <section className="grid gap-4 sm:grid-cols-2 md:grid-cols-3">
        <article className="card">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Workspace</p>
          <h2 className="text-lg font-semibold text-dark-50">{tenant?.name}</h2>
          <p className="text-sm text-dark-300 mt-2">
            Slug: <code className="bg-dark-900/70 px-1.5 py-0.5 rounded text-dark-200 text-xs">{tenant?.slug}</code>
          </p>
          <div className="mt-2">
            <StatusBadge status={tenant?.plan || 'free'} label={`${tenant?.plan || 'free'} plan`} />
          </div>
        </article>

        <article className="card">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Usage This Month</p>
          <h2 className="text-2xl font-semibold text-dark-50">
            {(usage?.ops_this_month ?? tenant?.usage_ops_month ?? 0).toLocaleString()}
          </h2>
          <p className="text-sm text-dark-300 mt-1">operations</p>
          <UsageBar
            used={usage?.ops_this_month ?? tenant?.usage_ops_month ?? 0}
            limit={usage?.ops_limit ?? tenant?.usage_limit ?? null}
            className="mt-3"
          />
        </article>

        <article className="card sm:col-span-2 md:col-span-1">
          <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Account</p>
          <h2 className="text-lg font-semibold text-dark-50 truncate">{user?.email}</h2>
          <p className="text-sm text-dark-300 mt-2 capitalize">Role: {user?.role?.replace(/_/g, ' ')}</p>
          <p className="text-sm text-dark-300 truncate">
            Gateway: <code className="text-xs text-dark-200">{gatewayBaseUrl || 'Not configured'}</code>
          </p>
        </article>
      </section>

      {/* Quick Actions */}
      <section>
        <h3 className="text-sm font-semibold text-dark-400 uppercase tracking-wide mb-3">Quick Actions</h3>
        <div className="grid gap-3 grid-cols-2 md:grid-cols-4">
          <Link
            href="/api-keys"
            className="rounded-xl border border-dark-600/30 bg-dark-800/30 p-4 hover:border-brand-500/30 transition-colors group"
          >
            <p className="text-sm font-medium text-dark-100 group-hover:text-brand-300">API Keys</p>
            <p className="text-xs text-dark-400 mt-1">{keyCount} key{keyCount !== 1 ? 's' : ''}</p>
          </Link>

          <Link
            href="/billing"
            className="rounded-xl border border-dark-600/30 bg-dark-800/30 p-4 hover:border-brand-500/30 transition-colors group"
          >
            <p className="text-sm font-medium text-dark-100 group-hover:text-brand-300">Billing</p>
            <p className="text-xs text-dark-400 mt-1 capitalize">{tenant?.plan || 'free'} plan</p>
          </Link>

          {canManage && (
            <Link
              href="/team"
              className="rounded-xl border border-dark-600/30 bg-dark-800/30 p-4 hover:border-brand-500/30 transition-colors group"
            >
              <p className="text-sm font-medium text-dark-100 group-hover:text-brand-300">Team</p>
              <p className="text-xs text-dark-400 mt-1">{teamCount} member{teamCount !== 1 ? 's' : ''}</p>
            </Link>
          )}

          <Link
            href="/settings"
            className="rounded-xl border border-dark-600/30 bg-dark-800/30 p-4 hover:border-brand-500/30 transition-colors group"
          >
            <p className="text-sm font-medium text-dark-100 group-hover:text-brand-300">Settings</p>
            <p className="text-xs text-dark-400 mt-1">Workspace &amp; profile</p>
          </Link>
        </div>
      </section>

      {/* Quick Integration */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-3">Quick Integration</h3>
        <p className="text-sm text-dark-300 mb-3">Use your API key against the Knol gateway:</p>
        <pre className="code-block text-xs">{`curl -X POST ${gatewayBaseUrl || '<GATEWAY_URL>'}/v1/memory \\
  -H "Authorization: Bearer <YOUR_API_KEY>" \\
  -H "Content-Type: application/json" \\
  -d '{"user_id":"user-1","content":"User likes concise replies"}'`}</pre>
        <p className="text-xs text-dark-400 mt-3">
          Don&apos;t have an API key? <Link href="/api-keys" className="text-brand-400 hover:text-brand-300">Create one</Link>
        </p>
      </section>
    </div>
  );
}
