'use client';

import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import { useAdminFetch, useAdminAction, useEditMode } from '@/features/admin/hooks';
import { PageHeader, Loading, ErrorBanner, Button, AdminCard } from '@/features/admin/components';
import { CONFIG_CATEGORIES } from '@/config';
import { configAPI, Config } from '@/features/admin/api';

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
  if (value === null) return '';
  if (valueType === 'json' || valueType === 'string_array') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
}

function parseEditedValue(raw: string, valueType: ConfigValueType): { value?: Config['value']; error?: string } {
  if (valueType === 'number') {
    if (!raw.trim()) {
      return { error: 'Number value cannot be empty' };
    }
    const parsed = Number(raw);
    if (!Number.isFinite(parsed)) {
      return { error: 'Enter a valid numeric value' };
    }
    return { value: parsed };
  }

  if (valueType === 'boolean') {
    if (raw === 'true') return { value: true };
    if (raw === 'false') return { value: false };
    return { error: 'Boolean must be true or false' };
  }

  if (valueType === 'json' || valueType === 'string_array') {
    try {
      const parsed = JSON.parse(raw || 'null');
      if (valueType === 'string_array') {
        const valid = Array.isArray(parsed) && parsed.every((item) => typeof item === 'string');
        if (!valid) return { error: 'Value must be a JSON array of strings' };
      }
      return { value: parsed };
    } catch {
      return { error: 'Enter valid JSON' };
    }
  }

  return { value: raw };
}

function getProviderValue(configs: Config[] | null): string {
  const llmProvider = (configs || []).find((c) => c.key === 'llm.provider');
  if (!llmProvider) return 'anthropic';
  return String(llmProvider.value || 'anthropic').toLowerCase();
}

export default function ConfigPage() {
  const searchParams = useSearchParams();
  const initialSearch = searchParams.get('search') || '';

  const { data: configs, loading, error, refetch } = useAdminFetch(
    () => configAPI.getAll(),
    []
  );

  const { run: updateConfig, loading: saving, error: saveError } = useAdminAction();
  const { editingKey, startEdit, stopEdit, isEditing } = useEditMode<string>();

  const [message, setMessage] = useState('');
  const [editValue, setEditValue] = useState('');
  const [searchTerm, setSearchTerm] = useState(initialSearch);
  const [activeCategory, setActiveCategory] = useState<string>('all');
  const [providerDraft, setProviderDraft] = useState('anthropic');
  const [providerSaving, setProviderSaving] = useState(false);
  const [editError, setEditError] = useState('');
  const [showAdvanced, setShowAdvanced] = useState(false);

  const providerValue = useMemo(() => getProviderValue(configs), [configs]);
  useEffect(() => {
    setProviderDraft(providerValue);
  }, [providerValue]);

  // Pick up search param from URL (e.g. from global search)
  useEffect(() => {
    const s = searchParams.get('search');
    if (s) setSearchTerm(s);
  }, [searchParams]);
  const normalizedSearch = searchTerm.trim().toLowerCase();
  const availableCategories = useMemo(
    () => ['all', ...CONFIG_CATEGORIES.filter((category) => (configs || []).some((item) => item.category === category))],
    [configs],
  );

  const filteredConfigs = useMemo(() => {
    const all = configs || [];
    return all.filter((config) => {
      if (activeCategory !== 'all' && config.category !== activeCategory) return false;
      if (!normalizedSearch) return true;
      const searchable = `${config.key} ${config.description || ''} ${config.category}`.toLowerCase();
      return searchable.includes(normalizedSearch);
    });
  }, [configs, activeCategory, normalizedSearch]);

  const handleSave = async () => {
    if (!editingKey) return;
    const config = (configs || []).find((c) => c.key === editingKey);
    if (!config) return;

    const valueType = inferValueType(config);
    const parsed = parseEditedValue(editValue, valueType);
    if (parsed.error) {
      setEditError(parsed.error);
      return;
    }

    const success = await updateConfig(() =>
      configAPI.update(editingKey, {
        value: parsed.value ?? '',
        value_type: valueType,
        category: config.category,
        description: config.description || '',
        env_override: config.env_override || null,
      })
    );

    if (success) {
      setMessage('Config updated successfully');
      stopEdit();
      setEditValue('');
      setEditError('');
      refetch();
      setTimeout(() => setMessage(''), 3000);
    }
  };

  const handleProviderSave = async () => {
    setProviderSaving(true);
    const success = await updateConfig(() =>
      configAPI.update('llm.provider', {
        value: providerDraft,
        value_type: 'string',
        category: 'llm',
        description: 'Active LLM provider (anthropic, openai, gemini)',
        env_override: 'LLM_PROVIDER',
      }),
    );
    setProviderSaving(false);

    if (success) {
      setMessage('Default LLM provider updated successfully');
      refetch();
      setTimeout(() => setMessage(''), 3000);
    }
  };

  if (loading) {
    return <Loading />;
  }

  const groupedConfigs = CONFIG_CATEGORIES.reduce((acc, category) => {
    acc[category] = filteredConfigs.filter((c) => c.category === category);
    return acc;
  }, {} as Record<string, Config[]>);

  const displayError = error || saveError;

  return (
    <div className="space-y-8">
      <PageHeader
        title="Configuration"
        description="Advanced runtime configuration with typed editing, search, and provider controls"
      />

      {displayError && <ErrorBanner message={displayError} />}

      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      <AdminCard>
        <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-dark-100">Default LLM Provider</h2>
            <p className="text-sm text-dark-400 mt-1">
              This updates <code>llm.provider</code> used by Knol services.
            </p>
          </div>
          <div className="flex items-center gap-3">
            <select
              value={providerDraft}
              onChange={(e) => setProviderDraft(e.target.value)}
              className="px-3 py-2 bg-dark-700/40 border border-dark-600/40 rounded-lg text-dark-100"
            >
              <option value="anthropic">Anthropic</option>
              <option value="openai">OpenAI</option>
              <option value="gemini">Gemini</option>
            </select>
            <Button onClick={handleProviderSave} disabled={providerSaving || providerDraft === providerValue}>
              {providerSaving ? 'Updating...' : 'Set Default Provider'}
            </Button>
          </div>
        </div>
        <p className="text-xs text-dark-500 mt-3">Current default: {providerValue}</p>
      </AdminCard>

      <AdminCard>
        <div className="grid grid-cols-1 md:grid-cols-12 gap-4 items-end">
          <div className="md:col-span-6">
            <label className="block text-xs text-dark-400 mb-1">Search</label>
            <input
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search by key, description, or category"
              className="w-full px-3 py-2 bg-dark-700/40 border border-dark-600/40 rounded-lg text-sm text-dark-100"
            />
          </div>
          <div className="md:col-span-4">
            <label className="block text-xs text-dark-400 mb-1">Category</label>
            <select
              value={activeCategory}
              onChange={(e) => setActiveCategory(e.target.value)}
              className="w-full px-3 py-2 bg-dark-700/40 border border-dark-600/40 rounded-lg text-sm text-dark-100"
            >
              {availableCategories.map((category) => (
                <option key={category} value={category}>
                  {category}
                </option>
              ))}
            </select>
          </div>
          <div className="md:col-span-2">
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => {
                setSearchTerm('');
                setActiveCategory('all');
              }}
            >
              Reset
            </Button>
          </div>
        </div>
        <div className="mt-4 flex items-center justify-between">
          <p className="text-xs text-dark-500">
            Showing {filteredConfigs.length} of {(configs || []).length} settings
          </p>
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-xs text-brand-400 hover:text-brand-300"
          >
            {showAdvanced ? 'Hide advanced metadata' : 'Show advanced metadata'}
          </button>
        </div>
      </AdminCard>

      <div className="space-y-6">
        {CONFIG_CATEGORIES.map((category) => {
          const items = groupedConfigs[category] || [];
          if (items.length === 0) return null;

          return (
            <AdminCard key={category} className="overflow-hidden">
              <div className="bg-dark-600/20 px-6 py-3 border-b border-dark-600/50">
                <h2 className="font-semibold text-dark-100 capitalize">{category}</h2>
              </div>

              <div className="divide-y divide-dark-600/30">
                {items.map((config) => (
                  <div key={config.key} className="p-6">
                    <div className="mb-3">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="font-mono text-sm text-brand-400 font-medium">{config.key}</p>
                        <span className="px-2 py-0.5 rounded-full text-[10px] bg-dark-700/60 border border-dark-600/40 text-dark-300">
                          {inferValueType(config)}
                        </span>
                        {config.env_override && (
                          <span className="px-2 py-0.5 rounded-full text-[10px] bg-brand-500/10 border border-brand-500/30 text-brand-300">
                            env: {config.env_override}
                          </span>
                        )}
                      </div>
                      {config.description && (
                        <p className="text-sm text-dark-400 mt-1">{config.description}</p>
                      )}
                      {showAdvanced && config.updated_at && (
                        <p className="text-xs text-dark-500 mt-1">
                          Updated: {new Date(config.updated_at).toLocaleString()}
                        </p>
                      )}
                    </div>

                    {isEditing(config.key) ? (
                      <div className="space-y-3">
                        {(() => {
                          const valueType = inferValueType(config);
                          if (valueType === 'boolean') {
                            return (
                              <select
                                value={editValue}
                                onChange={(e) => setEditValue(e.target.value)}
                                className="w-full px-4 py-2 bg-dark-600/50 border border-dark-600/50 rounded-lg text-dark-100 text-sm"
                              >
                                <option value="true">true</option>
                                <option value="false">false</option>
                              </select>
                            );
                          }

                          if (valueType === 'number') {
                            return (
                              <input
                                type="number"
                                value={editValue}
                                onChange={(e) => setEditValue(e.target.value)}
                                className="w-full px-4 py-2 bg-dark-600/50 border border-dark-600/50 rounded-lg text-dark-100 text-sm"
                              />
                            );
                          }

                          return (
                            <textarea
                              value={editValue}
                              onChange={(e) => setEditValue(e.target.value)}
                              className="w-full px-4 py-2 bg-dark-600/50 border border-dark-600/50 rounded-lg text-dark-100 font-mono text-sm focus:outline-none focus:border-brand-500/50 focus:ring-1 focus:ring-brand-500/30"
                              rows={valueType === 'json' || valueType === 'string_array' ? 8 : 3}
                            />
                          );
                        })()}
                        {editError && (
                          <p className="text-xs text-red-400">{editError}</p>
                        )}
                        <div className="flex space-x-2">
                          <Button
                            onClick={handleSave}
                            disabled={saving}
                            variant="primary"
                          >
                            {saving ? 'Saving...' : 'Save'}
                          </Button>
                          <Button
                            onClick={() => {
                              stopEdit();
                              setEditValue('');
                              setEditError('');
                            }}
                            variant="secondary"
                          >
                            Cancel
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <code className="block bg-dark-800/50 border border-dark-600/30 rounded p-3 text-dark-400 text-sm overflow-x-auto whitespace-pre-wrap break-words">
                            {toEditableString(config.value, inferValueType(config))}
                          </code>
                        </div>
                        <Button
                          onClick={() => {
                            startEdit(config.key);
                            setEditValue(toEditableString(config.value, inferValueType(config)));
                            setEditError('');
                          }}
                          variant="secondary"
                          className="ml-4"
                        >
                          Edit
                        </Button>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </AdminCard>
          );
        })}
      </div>

      {(!configs || configs.length === 0) && (
        <div className="text-center py-12 text-dark-400">
          <p>No configurations found</p>
        </div>
      )}
    </div>
  );
}
