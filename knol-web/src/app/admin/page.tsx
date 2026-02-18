'use client';

import { statusAPI, SystemStatus, auditAPI, AuditLog } from '@/features/admin/api';
import { useAdminFetch } from '@/features/admin/hooks';
import {
  PageHeader,
  Loading,
  ErrorBanner,
  StatCard,
  AdminCard,
  StatusBadge,
  DataTable,
} from '@/features/admin/components';
import { ReactNode } from 'react';

export default function DashboardPage() {
  const statusFetch = useAdminFetch(() => statusAPI.get());
  const auditFetch = useAdminFetch(() => auditAPI.list(undefined, undefined, 10));

  const { data: status, loading: statusLoading, error: statusError, refetch: refetchStatus } = statusFetch;
  const { data: auditLogs, loading: auditLoading, error: auditError, refetch: refetchAuditLogs } = auditFetch;

  const loading = statusLoading || auditLoading;
  const error = statusError || auditError;

  if (loading) {
    return <Loading message="Loading dashboard..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader title="Dashboard" description="Knol operational control plane — context engineering infrastructure" />

      {error && <ErrorBanner message={error} onRetry={() => {
        refetchStatus();
        refetchAuditLogs();
      }} />}

      {/* Quick Stats */}
      {status && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <StatCard label="Configurations" value={status.counts.configs} icon="⚙️" />
          <StatCard label="Credentials" value={status.counts.credentials} icon="🔑" />
          <StatCard label="Tenants" value={status.counts.tenants} icon="👥" />
          <StatCard label="Services" value={status.services?.length || 0} icon="🚀" />
        </div>
      )}

      {/* Platform Overview */}
      <AdminCard>
        <h2 className="text-xl font-semibold text-dark-50 mb-3">Knol Enterprise Control Plane</h2>
        <p className="text-sm text-dark-300 mb-4">
          The context engineering engine is open source (Apache 2.0). This admin surface provides the managed operational layer
          for reliability, security, compliance, and governance.
        </p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm text-dark-300">
          <div className="bg-dark-700/20 rounded-lg p-3">
            <strong className="text-dark-100">Credentials</strong> — Encrypted API keys, LLM provider tokens, webhook secrets. AES-256-GCM at rest.
          </div>
          <div className="bg-dark-700/20 rounded-lg p-3">
            <strong className="text-dark-100">Config</strong> — Runtime behavior, feature flags, provider selection, decay/conflict settings. Zero-downtime changes.
          </div>
          <div className="bg-dark-700/20 rounded-lg p-3">
            <strong className="text-dark-100">Tenants</strong> — Multi-tenant isolation with PostgreSQL RLS. Per-tenant rate limits and plan enforcement.
          </div>
          <div className="bg-dark-700/20 rounded-lg p-3">
            <strong className="text-dark-100">Audit</strong> — Full audit trail for compliance evidence. Every config change, login, and CRUD action logged.
          </div>
        </div>
      </AdminCard>

      {/* Service Health */}
      {status && (
        <AdminCard>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-dark-50">Service Health</h2>
            <button
              onClick={refetchStatus}
              className="text-xs text-brand-400 hover:text-brand-300 transition-colors"
            >
              Refresh
            </button>
          </div>
          <div className="space-y-3">
            {status.services.map((service) => (
              <div key={service.name} className="flex items-center justify-between p-4 bg-dark-700/20 rounded-lg">
                <div className="flex items-center space-x-3">
                  <span className="font-medium text-dark-200">{service.name}</span>
                </div>
                <div className="flex items-center space-x-4">
                  {service.latency_ms !== undefined && (
                    <span className="text-sm text-dark-400">{service.latency_ms}ms</span>
                  )}
                  <StatusBadge status={service.status} />
                </div>
              </div>
            ))}
          </div>
          {status.db && (
            <div className="mt-4 pt-4 border-t border-dark-600/30">
              <p className="text-sm text-dark-400">
                Database: {status.db.version || 'PostgreSQL + pgvector'} (pool: {status.db.pool_size || 'N/A'})
              </p>
            </div>
          )}
        </AdminCard>
      )}

      {/* Pipeline Status */}
      <AdminCard>
        <h2 className="text-xl font-semibold text-dark-50 mb-4">Write Pipeline</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-dark-700/20 rounded-lg p-4 text-center">
            <div className="text-2xl font-bold text-brand-400 mb-1">Active</div>
            <div className="text-xs text-dark-400">LLM Extraction</div>
            <div className="text-xs text-dark-500 mt-1">Entity + relationship extraction</div>
          </div>
          <div className="bg-dark-700/20 rounded-lg p-4 text-center">
            <div className="text-2xl font-bold text-brand-400 mb-1">Active</div>
            <div className="text-xs text-dark-400">Embedding Generation</div>
            <div className="text-xs text-dark-500 mt-1">Write-time vector indexing</div>
          </div>
          <div className="bg-dark-700/20 rounded-lg p-4 text-center">
            <div className="text-2xl font-bold text-brand-400 mb-1">Active</div>
            <div className="text-xs text-dark-400">Conflict Detection</div>
            <div className="text-xs text-dark-500 mt-1">Supersede / skip / merge / review</div>
          </div>
        </div>
      </AdminCard>

      {/* Recent Audit Logs */}
      <AdminCard>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-dark-50">Recent Activity</h2>
          <a href="/admin/audit" className="text-xs text-brand-400 hover:text-brand-300 transition-colors">
            View all
          </a>
        </div>
        {auditLogs && auditLogs.length === 0 ? (
          <p className="text-dark-400 text-sm">No recent activity</p>
        ) : (
          <div className="space-y-3">
            {auditLogs?.map((log) => (
              <div key={log.id} className="flex items-start space-x-4 p-4 bg-dark-700/20 rounded-lg">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-1">
                    <span className="font-medium text-dark-200">{log.admin_email || 'System'}</span>
                    <span className="text-xs bg-brand-500/20 text-brand-400 px-2 py-1 rounded">
                      {log.action}
                    </span>
                  </div>
                  <p className="text-sm text-dark-400">
                    {log.resource_type && `${log.resource_type}${log.resource_id ? ': ' + log.resource_id : ''}`}
                  </p>
                  <p className="text-xs text-dark-500 mt-1">
                    {new Date(log.timestamp).toLocaleString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </AdminCard>
    </div>
  );
}
