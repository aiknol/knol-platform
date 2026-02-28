'use client';

import { usePlaygroundState, SampleItem } from '@/features/app/playground/usePlaygroundState';
import { OPERATIONS, OperationField } from '@/features/app/playground/operations';
import PageHeader from '@/components/PageHeader';
import EmptyState from '@/components/EmptyState';

export default function PlaygroundPage() {
  const isLocal =
    typeof window !== 'undefined' &&
    (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1');
  const {
    gatewayBaseUrl,
    workspaceMode,
    loading,
    error,
    sampleData,
    fetchSampleData,
    sampleLoading,
    sampleError,
    selectedOperation,
    selectedOperationId,
    onSelectOperation,
    apiKey,
    setApiKey,
    apiKeyVisible,
    toggleApiKeyVisibility,
    fieldValues,
    setFieldValue,
    executing,
    response,
    responseError,
    onExecute,
    keyOptions,
    selectedKeyId,
    setSelectedKeyId,
    onSelectKey,
    creatingKey,
    onCreateQuickKey,
  } = usePlaygroundState();

  if (loading) {
    return <p className="text-dark-300 py-8">Loading playground...</p>;
  }

  const groups = [...new Set(OPERATIONS.map((op) => op.group))];
  const allFields = [...selectedOperation.pathParams, ...selectedOperation.bodyFields];
  const usableKeys = keyOptions.filter((k) => k.hasValue);
  const hasUnavailableKeys = keyOptions.length > usableKeys.length;
  const hasUserIdField = allFields.some((f) => f.name === 'user_id');
  const hasMemoryIdField =
    selectedOperation.group === 'Memory' && selectedOperation.pathParams.some((p) => p.name === 'id');
  const hasEntityIdField =
    selectedOperation.group === 'Graph' && selectedOperation.pathParams.some((p) => p.name === 'id');
  const hasFromField =
    selectedOperation.group === 'Graph' && selectedOperation.pathParams.some((p) => p.name === 'from');
  const hasToField =
    selectedOperation.group === 'Graph' && selectedOperation.pathParams.some((p) => p.name === 'to');

  /**
   * Return dropdown items for a field if sample data is available.
   * Memory ID fields get memory items, entity ID fields get entity items.
   */
  const getDropdownItems = (field: OperationField): SampleItem[] | null => {
    if (!sampleData) return null;
    // Memory ID fields: 'id' param in Memory group
    if (field.name === 'id' && selectedOperation.group === 'Memory') {
      return sampleData.memoryItems.length > 0 ? sampleData.memoryItems : null;
    }
    // User ID fields
    if (field.name === 'user_id' && sampleData.userIds.length > 0) {
      return sampleData.userIds.slice(0, 50).map((id) => ({ id, label: id }));
    }
    // Entity ID fields: 'id' param in Graph group, or 'from'/'to' path params
    if (field.name === 'id' && selectedOperation.group === 'Graph') {
      return sampleData.entityItems.length > 0 ? sampleData.entityItems : null;
    }
    if ((field.name === 'from' || field.name === 'to') && selectedOperation.group === 'Graph') {
      return sampleData.entityItems.length > 0 ? sampleData.entityItems : null;
    }
    return null;
  };

  return (
    <div className="max-w-6xl mx-auto px-4 py-6 sm:py-8 space-y-6">
      <PageHeader title="Playground" description="Test your API keys against the Knol gateway." />

      {error && <div className="alert-error">{error}</div>}

      {/* Gateway */}
      <section className="card">
        <p className="text-sm font-medium text-dark-100">Gateway</p>
        <p className="text-xs text-dark-500 mt-1">
          {isLocal ? 'Local dev gateway:' : 'Deployed gateway:'}{' '}
          <code className="text-dark-300">{gatewayBaseUrl}</code>
        </p>
      </section>

      {/* API Key Selection */}
      <section className="card">
        <label htmlFor="pg-key-select" className="form-label">
          Select API Key
        </label>
        <select
          id="pg-key-select"
          value={selectedKeyId}
          onChange={(e) => onSelectKey(e.target.value)}
          className="input-field mb-3"
        >
          <option value="manual">Enter manually</option>
          {selectedKeyId === 'initial' && (
            <option value="initial">Signup key (auto-populated)</option>
          )}
          {workspaceMode && usableKeys.length > 0 && (
            <optgroup label="Your Keys">
              {usableKeys.map((k) => (
                <option key={k.id} value={k.id}>
                  {k.name} ({k.role})
                </option>
              ))}
            </optgroup>
          )}
        </select>
        {!workspaceMode ? (
          <p className="-mt-1 mb-3 text-xs text-dark-400">
            Log in to list and create workspace API keys, or enter a key manually.
          </p>
        ) : usableKeys.length === 0 && (
          <div className="-mt-1 mb-3 flex items-center gap-3">
            {hasUnavailableKeys && (
              <p className="text-xs text-dark-400">
                Your existing keys are not available in this session.
              </p>
            )}
            <button
              type="button"
              onClick={onCreateQuickKey}
              disabled={creatingKey}
              className="text-xs font-medium text-primary-400 hover:text-primary-300 disabled:opacity-60"
            >
              {creatingKey ? 'Creating...' : 'Quick Create Key'}
            </button>
          </div>
        )}

        <label htmlFor="pg-api-key" className="form-label">
          API Key
        </label>
        <div className="flex gap-2">
          <input
            id="pg-api-key"
            type={apiKeyVisible ? 'text' : 'password'}
            value={apiKey}
            onChange={(e) => {
              setApiKey(e.target.value);
              if (selectedKeyId !== 'manual') {
                setSelectedKeyId('manual');
              }
            }}
            className="input-field font-mono text-sm"
            placeholder="knol_sk_..."
          />
          <button
            type="button"
            onClick={toggleApiKeyVisibility}
            className="btn-secondary !px-3 !py-2 text-sm shrink-0"
          >
            {apiKeyVisible ? 'Hide' : 'Show'}
          </button>
        </div>
      </section>

      {/* Sample Data */}
      <section className="card">
        <div className="flex items-center justify-between gap-3">
          <div className="min-w-0">
            <p className="text-sm font-medium text-dark-100">Sample Data</p>
            <p className="text-xs text-dark-500">
              Loads real IDs from the gateway to power dropdowns (Memory ID, Entity ID, User ID).
            </p>
          </div>
          <button
            type="button"
            onClick={fetchSampleData}
            disabled={sampleLoading || !apiKey.trim() || !gatewayBaseUrl.trim()}
            className="btn-secondary !px-3 !py-2 text-sm shrink-0 disabled:opacity-60"
          >
            {sampleLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>

        {sampleError && <div className="alert-error mt-3">{sampleError}</div>}

        {sampleData ? (
          <div className="mt-3 grid gap-2 sm:grid-cols-3 text-xs text-dark-400">
            <p>
              <span className="text-dark-200 font-medium">{sampleData.memoryIds.length}</span> memories
            </p>
            <p>
              <span className="text-dark-200 font-medium">{sampleData.entityIds.length}</span> entities
            </p>
            <p>
              <span className="text-dark-200 font-medium">{sampleData.userIds.length}</span> user IDs
            </p>
          </div>
        ) : (
          <p className="mt-3 text-xs text-dark-400">
            Paste an API key and click Refresh to load real IDs.
          </p>
        )}

        {sampleData && (
          <div className="mt-4 grid gap-4 md:grid-cols-2">
            {/* Memories */}
            <div>
              <p className="text-xs font-medium text-dark-200 mb-2">Recent memories</p>
              {sampleData.memoryItems.length === 0 ? (
                <p className="text-xs text-dark-500">
                  No memories found. Try “Write Memory”, then Refresh.
                </p>
              ) : (
                <ul className="space-y-2">
                  {sampleData.memoryItems.slice(0, 6).map((m) => (
                    <li key={m.id} className="flex items-center justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-xs text-dark-300 truncate">{m.label}</p>
                        <p className="text-[11px] text-dark-600 font-mono truncate">{m.id}</p>
                      </div>
                      {hasMemoryIdField && (
                        <button
                          type="button"
                          className="btn-secondary !px-2.5 !py-1 text-xs shrink-0"
                          onClick={() => setFieldValue('id', m.id)}
                        >
                          Use as ID
                        </button>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {/* Entities / Users */}
            <div>
              <p className="text-xs font-medium text-dark-200 mb-2">Recent entities</p>
              {sampleData.entityItems.length === 0 ? (
                <p className="text-xs text-dark-500">No entities found (or your key lacks access).</p>
              ) : (
                <ul className="space-y-2">
                  {sampleData.entityItems.slice(0, 6).map((e) => (
                    <li key={e.id} className="flex items-center justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-xs text-dark-300 truncate">{e.label}</p>
                        <p className="text-[11px] text-dark-600 font-mono truncate">{e.id}</p>
                      </div>
                      <div className="flex gap-2 shrink-0">
                        {hasEntityIdField && (
                          <button
                            type="button"
                            className="btn-secondary !px-2.5 !py-1 text-xs"
                            onClick={() => setFieldValue('id', e.id)}
                          >
                            Use as ID
                          </button>
                        )}
                        {hasFromField && (
                          <button
                            type="button"
                            className="btn-secondary !px-2.5 !py-1 text-xs"
                            onClick={() => setFieldValue('from', e.id)}
                          >
                            Use as From
                          </button>
                        )}
                        {hasToField && (
                          <button
                            type="button"
                            className="btn-secondary !px-2.5 !py-1 text-xs"
                            onClick={() => setFieldValue('to', e.id)}
                          >
                            Use as To
                          </button>
                        )}
                      </div>
                    </li>
                  ))}
                </ul>
              )}

              {hasUserIdField && sampleData.userIds.length > 0 && (
                <div className="mt-4">
                  <p className="text-xs font-medium text-dark-200 mb-2">User IDs</p>
                  <div className="flex flex-wrap gap-2">
                    {sampleData.userIds.slice(0, 8).map((uid) => (
                      <button
                        key={uid}
                        type="button"
                        className="btn-secondary !px-2 !py-1 text-[11px] font-mono"
                        onClick={() => setFieldValue('user_id', uid)}
                      >
                        {uid.slice(0, 8)}…
                      </button>
                    ))}
                    {sampleData.userIds.length > 8 && (
                      <span className="text-[11px] text-dark-500 self-center">
                        +{sampleData.userIds.length - 8} more
                      </span>
                    )}
                  </div>
                  <p className="text-[11px] text-dark-500 mt-2">
                    Tip: leave User ID blank to search across all users.
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </section>

      {/* Request + Response side by side */}
      <section className="grid gap-6 md:grid-cols-2">
        {/* Request Panel */}
        <article className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Request</h3>
          <div className="space-y-4">
            <div>
              <label htmlFor="pg-operation" className="form-label">
                Operation
              </label>
              <select
                id="pg-operation"
                value={selectedOperationId}
                onChange={(e) => onSelectOperation(e.target.value)}
                className="input-field"
              >
                {groups.map((group) => (
                  <optgroup key={group} label={group}>
                    {OPERATIONS.filter((op) => op.group === group).map((op) => (
                      <option key={op.id} value={op.id}>
                        {op.method} - {op.label}
                      </option>
                    ))}
                  </optgroup>
                ))}
              </select>
              <p className="text-xs text-dark-500 mt-1">{selectedOperation.description}</p>
            </div>

            {allFields.map((field) => {
              const dropdownItems = getDropdownItems(field);
              return (
                <div key={field.name}>
                  <label htmlFor={`pg-${field.name}`} className="form-label">
                    {field.label}
                    {field.required && ' *'}
                  </label>
                  {dropdownItems ? (
                    <select
                      id={`pg-${field.name}`}
                      value={fieldValues[field.name] || ''}
                      onChange={(e) => setFieldValue(field.name, e.target.value)}
                      className="input-field"
                      required={field.required}
                    >
                      <option value="">Select...</option>
                      {dropdownItems.map((item) => (
                        <option key={item.id} value={item.id}>
                          {item.label} ({item.id.slice(0, 8)}...)
                        </option>
                      ))}
                    </select>
                  ) : field.type === 'textarea' || field.type === 'json' ? (
                    <textarea
                      id={`pg-${field.name}`}
                      value={fieldValues[field.name] || ''}
                      onChange={(e) => setFieldValue(field.name, e.target.value)}
                      className="input-field font-mono text-sm min-h-[100px]"
                      placeholder={field.placeholder}
                      required={field.required}
                    />
                  ) : (
                    <input
                      id={`pg-${field.name}`}
                      type={field.type === 'number' ? 'number' : 'text'}
                      value={fieldValues[field.name] || ''}
                      onChange={(e) => setFieldValue(field.name, e.target.value)}
                      className="input-field"
                      placeholder={field.placeholder}
                      required={field.required}
                    />
                  )}
                </div>
              );
            })}

            <button
              onClick={onExecute}
              disabled={executing}
              className="btn-primary !py-2.5 disabled:opacity-60"
            >
              {executing ? 'Sending...' : 'Send Request'}
            </button>
          </div>
        </article>

        {/* Response Panel */}
        <article className="card flex flex-col">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Response</h3>

          {responseError && <div className="alert-error mb-4">{responseError}</div>}

          {response ? (
            <div className="flex-1 flex flex-col">
              <div className="flex items-center gap-3 mb-3">
                <span
                  className={`text-sm font-mono font-semibold ${
                    response.status >= 200 && response.status < 300
                      ? 'text-emerald-400'
                      : response.status >= 400
                        ? 'text-red-400'
                        : 'text-amber-400'
                  }`}
                >
                  {response.status}
                </span>
                <span className="text-xs text-dark-500">{response.duration}ms</span>
              </div>
              <pre className="code-block text-xs flex-1 max-h-[500px] overflow-y-auto">
                {response.body}
              </pre>
            </div>
          ) : (
            <EmptyState message="Send a request to see the response here." />
          )}
        </article>
      </section>
    </div>
  );
}
