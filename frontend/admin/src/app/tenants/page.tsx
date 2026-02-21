'use client';

import { useState } from 'react';
import { Tenant, tenantsAPI } from '@/features/admin/api';
import { useAdminFetch, useAdminAction } from '@/features/admin/hooks';
import { PageHeader, Loading, ErrorBanner, Button, AdminCard } from '@/features/admin/components';

interface EditingTenant extends Tenant {
  configInput?: string;
}

export default function TenantsPage() {
  const { data: tenants, loading, error, refetch } = useAdminFetch<Tenant[]>(
    () => tenantsAPI.list(),
    []
  );
  const { run: updateTenant, loading: saving } = useAdminAction();

  const [message, setMessage] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editing, setEditing] = useState<EditingTenant | null>(null);

  const handleExpand = (id: string) => {
    setExpandedId(expandedId === id ? null : id);
  };

  const handleEdit = (tenant: Tenant) => {
    setEditingId(tenant.id);
    setEditing({
      ...tenant,
      configInput: JSON.stringify(tenant.config || {}, null, 2),
    });
  };

  const handleSave = async () => {
    if (!editing || !editingId) return;

    try {
      let config: Record<string, any> = {};
      if (editing.configInput) {
        try {
          config = JSON.parse(editing.configInput);
        } catch (e) {
          throw new Error('Invalid JSON in config');
        }
      }

      const success = await updateTenant(() =>
        tenantsAPI.update(editingId, editing.plan, config, editing.usage_limit, editing.name)
      );

      if (success) {
        // Refetch to update the list
        await refetch();

        setMessage(`Tenant "${editing.name}" updated successfully`);
        setEditingId(null);
        setEditing(null);
        setTimeout(() => setMessage(''), 3000);
      }
    } catch (err) {
      throw err;
    }
  };

  const handleCancel = () => {
    setEditingId(null);
    setEditing(null);
  };

  if (loading) {
    return <Loading />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Tenants"
        description="Manage customer tenants and configurations"
      />

      {error && <ErrorBanner message={error} />}

      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      {/* Tenants Table */}
      <AdminCard>
        {!tenants || tenants.length === 0 ? (
          <div className="p-12 text-center text-dark-400">
            <p>No tenants found</p>
          </div>
        ) : (
          <div className="divide-y divide-dark-700/30">
            {tenants.map((tenant: Tenant) => (
              <div key={tenant.id} className="p-6">
                {editingId === tenant.id && editing ? (
                  // Edit Mode
                  <div className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-dark-200 mb-2">Name</label>
                        <input
                          type="text"
                          value={editing.name || ''}
                          onChange={(e) => setEditing({ ...editing, name: e.target.value })}
                          className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-dark-200 mb-2">Plan</label>
                        <select
                          value={editing.plan || ''}
                          onChange={(e) => setEditing({ ...editing, plan: e.target.value })}
                          className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                        >
                          <option value="">Select plan</option>
                          <option value="free">Free</option>
                          <option value="pro">Pro</option>
                          <option value="enterprise">Enterprise</option>
                        </select>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-dark-200 mb-2">Usage Limit</label>
                        <input
                          type="number"
                          value={editing.usage_limit || ''}
                          onChange={(e) =>
                            setEditing({
                              ...editing,
                              usage_limit: e.target.value ? parseInt(e.target.value) : undefined,
                            })
                          }
                          placeholder="Unlimited if empty"
                          className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                        />
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-dark-200 mb-2">Config (JSON)</label>
                      <textarea
                        value={editing.configInput || ''}
                        onChange={(e) => setEditing({ ...editing, configInput: e.target.value })}
                        className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 font-mono text-sm focus:outline-none focus:border-brand-500/50"
                        rows={6}
                      />
                    </div>

                    <div className="flex space-x-2 pt-2">
                      <Button
                        onClick={handleSave}
                        disabled={saving}
                        variant="primary"
                      >
                        {saving ? 'Saving...' : 'Save'}
                      </Button>
                      <Button
                        onClick={handleCancel}
                        variant="secondary"
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  // View Mode
                  <>
                    <div
                      className="flex items-center justify-between cursor-pointer"
                      onClick={() => handleExpand(tenant.id)}
                    >
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-dark-100">{tenant.name}</h3>
                        <p className="text-sm text-dark-400 mt-1">ID: {tenant.id}</p>
                      </div>
                      <div className="flex items-center space-x-4">
                        {tenant.plan && (
                          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-brand-500/20 text-brand-400">
                            {tenant.plan}
                          </span>
                        )}
                        <span className="text-dark-500">{expandedId === tenant.id ? '▼' : '▶'}</span>
                      </div>
                    </div>

                    {expandedId === tenant.id && (
                      <div className="mt-4 pt-4 border-t border-dark-700/30 space-y-3">
                        {tenant.usage_limit && (
                          <p className="text-sm text-dark-400">
                            <span className="text-dark-600">Usage Limit:</span> {tenant.usage_limit}
                          </p>
                        )}

                        {tenant.created_at && (
                          <p className="text-sm text-dark-400">
                            <span className="text-dark-600">Created:</span>{' '}
                            {new Date(tenant.created_at).toLocaleString()}
                          </p>
                        )}

                        {tenant.config && Object.keys(tenant.config).length > 0 && (
                          <div>
                            <p className="text-sm text-dark-600 mb-2">Configuration:</p>
                            <pre className="bg-dark-900/50 border border-dark-700/30 rounded p-3 text-dark-300 text-xs overflow-x-auto">
                              {JSON.stringify(tenant.config, null, 2)}
                            </pre>
                          </div>
                        )}

                        <div className="flex space-x-2 pt-2">
                          <Button
                            onClick={() => handleEdit(tenant)}
                            variant="primary"
                          >
                            Edit
                          </Button>
                        </div>
                      </div>
                    )}
                  </>
                )}
              </div>
            ))}
          </div>
        )}
      </AdminCard>
    </div>
  );
}
