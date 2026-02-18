'use client';

import { useState } from 'react';
import { useAdminFetch } from '@/features/admin/hooks';
import { PageHeader, Loading, ErrorBanner, DataTable, Button } from '@/features/admin/components';
import { AUDIT_ACTIONS, AUDIT_RESOURCE_TYPES } from '@/config';
import { auditAPI, AuditLog } from '@/features/admin/api';

export default function AuditPage() {
  const [filters, setFilters] = useState({
    action: '',
    resourceType: '',
  });
  const [limit, setLimit] = useState(50);

  const { data: logs, loading, error } = useAdminFetch(
    () => auditAPI.list(filters.action || undefined, filters.resourceType || undefined, limit),
    [filters, limit]
  );

  const handleFilterChange = (key: string, value: string) => {
    setFilters({ ...filters, [key]: value });
  };

  if (loading) {
    return <Loading message="Loading audit logs..." />;
  }

  const columns = [
    {
      header: 'Timestamp',
      accessor: (log: AuditLog) => new Date(log.timestamp).toLocaleString(),
    },
    {
      header: 'Admin',
      accessor: (log: AuditLog) => log.admin_email || 'System',
    },
    {
      header: 'Action',
      accessor: (log: AuditLog) => (
        <span className="px-2 py-1 rounded-full text-xs font-semibold bg-brand-500/20 text-brand-400">
          {log.action}
        </span>
      ),
    },
    {
      header: 'Resource',
      accessor: (log: AuditLog) => (
        <div>
          {log.resource_type && (
            <p className="text-dark-300 capitalize">{log.resource_type}</p>
          )}
          {log.resource_id && (
            <p className="text-dark-500 text-xs font-mono">{log.resource_id}</p>
          )}
        </div>
      ),
    },
    {
      header: 'Changes',
      accessor: (log: AuditLog) =>
        log.old_value !== undefined || log.new_value !== undefined ? (
          <div className="space-y-1">
            {log.old_value !== undefined && (
              <p className="text-red-400 text-xs">
                <span className="text-dark-500">Old:</span> {JSON.stringify(log.old_value)}
              </p>
            )}
            {log.new_value !== undefined && (
              <p className="text-green-400 text-xs">
                <span className="text-dark-500">New:</span> {JSON.stringify(log.new_value)}
              </p>
            )}
          </div>
        ) : (
          <span className="text-dark-500">—</span>
        ),
    },
  ];

  return (
    <div className="space-y-8">
      <PageHeader
        title="Audit Log"
        description="Track all admin actions and changes"
      />

      {error && <ErrorBanner message={error} />}

      {/* Filters */}
      <div className="bg-dark-800/30 border border-dark-600/50 rounded-lg p-6">
        <h2 className="text-lg font-semibold text-dark-100 mb-4">Filters</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm font-medium text-dark-200 mb-2">Action</label>
            <select
              value={filters.action}
              onChange={(e) => handleFilterChange('action', e.target.value)}
              className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
            >
              <option value="">All actions</option>
              {AUDIT_ACTIONS.map((action) => (
                <option key={action} value={action}>
                  {action.charAt(0).toUpperCase() + action.slice(1)}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-dark-200 mb-2">Resource Type</label>
            <select
              value={filters.resourceType}
              onChange={(e) => handleFilterChange('resourceType', e.target.value)}
              className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
            >
              <option value="">All types</option>
              {AUDIT_RESOURCE_TYPES.map((type) => (
                <option key={type} value={type}>
                  {type.charAt(0).toUpperCase() + type.slice(1)}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-dark-200 mb-2">Results per page</label>
            <select
              value={limit}
              onChange={(e) => setLimit(parseInt(e.target.value))}
              className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
            >
              <option value={10}>10</option>
              <option value={25}>25</option>
              <option value={50}>50</option>
              <option value={100}>100</option>
            </select>
          </div>
        </div>
      </div>

      {/* Audit Logs Table */}
      <div className="bg-dark-800/30 border border-dark-600/50 rounded-lg overflow-hidden">
        {logs && logs.length === 0 ? (
          <div className="p-12 text-center text-dark-400">
            <p>No audit logs found matching the filters</p>
          </div>
        ) : (
          <DataTable columns={columns} data={logs || []} rowKey={(log) => log.id} />
        )}
      </div>

      {/* Results Info */}
      <div className="text-center text-dark-400 text-sm">
        <p>Showing {logs?.length || 0} of {limit} results</p>
      </div>
    </div>
  );
}
