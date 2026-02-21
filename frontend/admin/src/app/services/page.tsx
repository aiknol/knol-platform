'use client';

import { useMemo, useState } from 'react';
import { configAPI, Config, statusAPI, SystemStatus } from '@/features/admin/api';
import { useAdminAction, useAdminFetch } from '@/features/admin/hooks';
import {
  AdminCard,
  Button,
  ErrorBanner,
  Loading,
  PageHeader,
  StatusBadge,
} from '@/features/admin/components';

type ConfigValueType = NonNullable<Config['value_type']>;

function inferValueType(config: Config): ConfigValueType {
  if (config.value_type) return config.value_type;
  if (typeof config.value === 'number') return 'number';
  if (typeof config.value === 'boolean') return 'boolean';
  if (Array.isArray(config.value)) return 'string_array';
  if (typeof config.value === 'object' && config.value !== null) return 'json';
  return 'string';
}

function toEditableString(value: Config['value'], valueType: ConfigValueType): string {
  if (value === null || value === undefined) return '';
  if (valueType === 'json' || valueType === 'string_array') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
}

function parseEditedValue(raw: string, valueType: ConfigValueType): { value?: Config['value']; error?: string } {
  const trimmed = raw.trim();
  try {
    switch (valueType) {
      case 'number': {
        if (trimmed.length === 0) return { error: 'Number cannot be empty' };
        const parsed = Number(trimmed);
        if (Number.isNaN(parsed)) return { error: 'Invalid number' };
        return { value: parsed };
      }
      case 'boolean': {
        const lower = trimmed.toLowerCase();
        if (['true', '1', 'yes', 'on'].includes(lower)) return { value: true };
        if (['false', '0', 'no', 'off'].includes(lower)) return { value: false };
        return { error: 'Use true or false' };
      }
      case 'json':
      case 'string_array': {
        const parsed = JSON.parse(raw);
        if (valueType === 'string_array' && !Array.isArray(parsed)) {
          return { error: 'Must be a JSON array' };
        }
        return { value: parsed };
      }
      default:
        return { value: raw };
    }
  } catch (err) {
    return { error: err instanceof Error ? err.message : 'Invalid value' };
  }
}

function statusToBadge(status: string): string {
  if (status === 'healthy' || status === 'up') return 'up';
  if (status === 'unhealthy' || status === 'degraded') return 'degraded';
  return 'down';
}

export default function ServicesPage() {
  const [editingKey, setEditingKey] = useState<string>('');
  const [editValue, setEditValue] = useState<string>('');
  const [message, setMessage] = useState('');

  const statusFetch = useAdminFetch(() => statusAPI.get() as Promise<SystemStatus>);
  const configFetch = useAdminFetch(() => configAPI.getAll());
  const saveAction = useAdminAction();

  const serviceConfigs = useMemo(
    () =>
      (configFetch.data || [])
        .filter((c) => c.key.startsWith('services.'))
        .sort((a, b) => a.key.localeCompare(b.key)),
    [configFetch.data],
  );

  const startEdit = (cfg: Config) => {
    setEditingKey(cfg.key);
    setEditValue(toEditableString(cfg.value, inferValueType(cfg)));
  };

  const cancelEdit = () => {
    setEditingKey('');
    setEditValue('');
  };

  const saveConfig = async () => {
    const cfg = serviceConfigs.find((c) => c.key === editingKey);
    if (!cfg) return;

    const valueType = inferValueType(cfg);
    const parsed = parseEditedValue(editValue, valueType);
    if (parsed.error) {
      return;
    }

    const result = await saveAction.run(() =>
      configAPI.update(cfg.key, {
        value: parsed.value as Config['value'],
        value_type: valueType,
        category: cfg.category,
        description: cfg.description || '',
        env_override: cfg.env_override || null,
      }),
    );

    if (result !== null) {
      setMessage(`Updated ${cfg.key}`);
      cancelEdit();
      configFetch.refetch();
      setTimeout(() => setMessage(''), 2500);
    }
  };

  if (statusFetch.loading || configFetch.loading) {
    return <Loading message="Loading service controls..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Services"
        description="Enterprise service access, health, and runtime configuration controls"
        action={
          <Button variant="secondary" onClick={statusFetch.refetch}>
            Refresh Health
          </Button>
        }
      />

      {(statusFetch.error || configFetch.error || saveAction.error) && (
        <ErrorBanner message={statusFetch.error || configFetch.error || saveAction.error} />
      )}

      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      <AdminCard>
        <h2 className="text-lg font-semibold text-dark-50 mb-4">Service Health</h2>
        <div className="space-y-3">
          {(statusFetch.data?.services || []).map((service) => (
            <div key={service.name} className="p-4 bg-dark-700/20 rounded-lg flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-dark-100">{service.name}</p>
                {service.latency_ms !== undefined && (
                  <p className="text-xs text-dark-500 mt-1">{service.latency_ms} ms</p>
                )}
                {(service as any).error && (
                  <p className="text-xs text-red-400 mt-1">{(service as any).error}</p>
                )}
              </div>
              <StatusBadge status={statusToBadge(service.status)} />
            </div>
          ))}
        </div>
      </AdminCard>

      <AdminCard>
        <h2 className="text-lg font-semibold text-dark-50 mb-4">Runtime Service Configuration</h2>
        <p className="text-xs text-dark-500 mb-4">
          These values map to <code>services.*</code> keys and are applied by backend services reading runtime config.
        </p>

        <div className="space-y-3">
          {serviceConfigs.map((cfg) => {
            const isEditing = editingKey === cfg.key;
            const type = inferValueType(cfg);
            return (
              <div key={cfg.key} className="p-4 bg-dark-700/20 rounded-lg">
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0">
                    <p className="font-mono text-sm text-brand-400">{cfg.key}</p>
                    {cfg.description && <p className="text-xs text-dark-500 mt-1">{cfg.description}</p>}
                    <p className="text-xs text-dark-600 mt-1">type: {type}</p>
                  </div>
                  {!isEditing && (
                    <Button size="sm" variant="secondary" onClick={() => startEdit(cfg)}>
                      Edit
                    </Button>
                  )}
                </div>

                {isEditing ? (
                  <div className="mt-3 space-y-2">
                    {type === 'json' || type === 'string_array' ? (
                      <textarea
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="w-full px-3 py-2 bg-dark-700/50 border border-dark-600/50 rounded text-sm text-dark-100 focus:outline-none focus:border-brand-500/50"
                        rows={5}
                      />
                    ) : (
                      <input
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="w-full px-3 py-2 bg-dark-700/50 border border-dark-600/50 rounded text-sm text-dark-100 focus:outline-none focus:border-brand-500/50"
                      />
                    )}
                    <div className="flex gap-2">
                      <Button size="sm" onClick={saveConfig} disabled={saveAction.loading}>
                        Save
                      </Button>
                      <Button size="sm" variant="ghost" onClick={cancelEdit}>
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  <pre className="mt-3 text-xs text-dark-300 whitespace-pre-wrap break-all">
                    {toEditableString(cfg.value, type)}
                  </pre>
                )}
              </div>
            );
          })}
        </div>
      </AdminCard>
    </div>
  );
}
