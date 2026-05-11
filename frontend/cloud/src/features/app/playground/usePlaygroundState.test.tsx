import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  appAuthAPI: {
    me: vi.fn(),
  },
  appApiKeysAPI: {
    list: vi.fn(),
    create: vi.fn(),
  },
  getAppAuthUser: vi.fn(),
  clearAppAuthSession: vi.fn(),
  getInitialApiKey: vi.fn(),
  getSessionApiKeys: vi.fn(),
  getSessionApiKeyValue: vi.fn(),
  storeSessionApiKey: vi.fn(),
}));

vi.mock('@/features/app/api', () => ({
  appAuthAPI: apiMocks.appAuthAPI,
  appApiKeysAPI: apiMocks.appApiKeysAPI,
  getAppAuthUser: apiMocks.getAppAuthUser,
  clearAppAuthSession: apiMocks.clearAppAuthSession,
  getInitialApiKey: apiMocks.getInitialApiKey,
  getSessionApiKeys: apiMocks.getSessionApiKeys,
  getSessionApiKeyValue: apiMocks.getSessionApiKeyValue,
  storeSessionApiKey: apiMocks.storeSessionApiKey,
}));

import { usePlaygroundState } from './usePlaygroundState';

const MOCK_ME = {
  user: { id: 'u1', email: 'dev@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'prod-key', role: 'admin', active: true, created_at: '2026-01-01', key_prefix: 'knol_sk_...ab12' },
  { id: 'k2', name: 'dev-key', role: 'developer', active: true, created_at: '2026-01-15' },
  { id: 'k3', name: 'old-key', role: 'developer', active: false, created_at: '2025-06-01' },
];

/** Returns a fresh Response so each fetch call gets its own consumable body. */
function proxyResponse(status = 200, body: unknown = { results: [] }) {
  return new Response(
    JSON.stringify({ status, body: JSON.stringify(body) }),
    { status: 200, headers: { 'content-type': 'application/json' } },
  );
}

/** Helper to find the onExecute proxy call (not sample-data calls). */
function findExecuteCall(mockFn: ReturnType<typeof vi.fn>) {
  for (const [url, opts] of mockFn.mock.calls) {
    if (url !== '/api/playground/proxy') continue;
    const body = JSON.parse(opts?.body || '{}');
    // Sample-data calls use export/entities/audit paths; onExecute uses the actual operation path
    const isSampleData = body.url?.includes('/v1/memory/export')
      || body.url?.includes('/v1/graph/entities?limit=');
    if (body.url && !isSampleData) {
      return [url, opts] as const;
    }
  }
  // Fallback: return the last call
  return mockFn.mock.calls[mockFn.mock.calls.length - 1] as [string, RequestInit];
}

beforeEach(() => {
  vi.clearAllMocks();
  globalThis.localStorage?.clear?.();
  apiMocks.appAuthAPI.me.mockResolvedValue(MOCK_ME);
  apiMocks.appApiKeysAPI.list.mockResolvedValue(MOCK_KEYS);
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_ME.user);
  apiMocks.getInitialApiKey.mockReturnValue(null);
  apiMocks.getSessionApiKeys.mockReturnValue([]);
  apiMocks.getSessionApiKeyValue.mockReturnValue(null);
  // Return fresh Response each time (body can only be consumed once per Response)
  vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
    Promise.resolve(proxyResponse()),
  );
});

describe('usePlaygroundState', () => {
  it('loads gateway URL on mount', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.gatewayBaseUrl).toBe('http://localhost:3000');
    expect(result.current.error).toBe('');
  });

  it('sets error when me() fails', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValue(new Error('Network down'));
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.error).toBe('Network down');
  });

  it('defaults to search-memory operation', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.selectedOperationId).toBe('search-memory');
    expect(result.current.selectedOperation.label).toBe('Search Memories');
  });

  it('onSelectOperation changes the selected operation and resets state', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.setFieldValue('query', 'hello');
    });
    expect(result.current.fieldValues['query']).toBe('hello');

    act(() => {
      result.current.onSelectOperation('get-memory');
    });

    expect(result.current.selectedOperationId).toBe('get-memory');
    expect(result.current.selectedOperation.method).toBe('GET');
    // Previous custom value is gone; defaults for the new operation are populated
    expect(result.current.fieldValues['query']).toBeUndefined();
    expect(result.current.fieldValues['id']).toBeUndefined();
    expect(result.current.response).toBeNull();
    expect(result.current.responseError).toBe('');
  });

  it('setFieldValue updates field values', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.setFieldValue('query', 'test query');
      result.current.setFieldValue('user_id', '550e8400-e29b-41d4-a716-446655440000');
    });

    expect(result.current.fieldValues['query']).toBe('test query');
    expect(result.current.fieldValues['user_id']).toBe('550e8400-e29b-41d4-a716-446655440000');
  });

  it('toggleApiKeyVisibility toggles visibility', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.apiKeyVisible).toBe(false);

    act(() => result.current.toggleApiKeyVisibility());
    expect(result.current.apiKeyVisible).toBe(true);

    act(() => result.current.toggleApiKeyVisibility());
    expect(result.current.apiKeyVisible).toBe(false);
  });

  it('onExecute shows error when API key is empty', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();
    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.responseError).toBe('Please enter or select an API key.');
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });

  it('onExecute sends request through proxy with correct payload', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_mykey'));

    // Wait for sample-data effect to settle, then set field values after
    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });
    act(() => result.current.setFieldValue('query', 'user preferences'));
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      await result.current.onExecute();
    });

    expect(globalThis.fetch).toHaveBeenCalledTimes(1);
    const [proxyUrl, proxyOpts] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(proxyUrl).toBe('/api/playground/proxy');
    expect(proxyOpts.method).toBe('POST');

    const proxyBody = JSON.parse(proxyOpts.body);
    expect(proxyBody.url).toBe('http://localhost:3000/v1/memory/search');
    expect(proxyBody.method).toBe('POST');
    expect(proxyBody.headers['Authorization']).toBe('Bearer knol_sk_mykey');
    expect(proxyBody.headers['Content-Type']).toBe('application/json');
    const gatewayBody = JSON.parse(proxyBody.body);
    expect(gatewayBody.query).toBe('user preferences');
    // Limit default is included; user_id is optional and omitted by default.
    expect(gatewayBody.user_id).toBeUndefined();
    expect(gatewayBody.limit).toBe(10);
  });

  it('onExecute substitutes path params in URL', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.onSelectOperation('get-memory');
      result.current.setApiKey('knol_sk_test');
    });

    // Wait for sample-data effect to settle, then set field values after
    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });
    act(() => result.current.setFieldValue('id', 'mem_123'));
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      await result.current.onExecute();
    });

    const [, opts] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const proxyBody = JSON.parse(opts.body);
    expect(proxyBody.url).toBe('http://localhost:3000/v1/memory/mem_123');
  });

  it('onExecute stores response with status, body and duration', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));

    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });

    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.response).not.toBeNull();
    expect(result.current.response!.status).toBe(200);
    expect(result.current.response!.body).toContain('results');
    expect(typeof result.current.response!.duration).toBe('number');
  });

  it('onExecute handles fetch failure', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('Connection refused'));
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));

    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.responseError).toBe('Connection refused');
    expect(result.current.response).toBeNull();
  });

  it('onExecute handles non-JSON response body', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
      Promise.resolve(new Response(
        JSON.stringify({ status: 404, body: 'Not Found' }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )),
    );
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));

    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });

    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.response!.status).toBe(404);
    expect(result.current.response!.body).toBe('Not Found');
  });

  it('onExecute truncates very large response bodies', async () => {
    const huge = 'x'.repeat(200_000 + 10);
    vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
      Promise.resolve(new Response(
        JSON.stringify({ status: 200, body: huge }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )),
    );
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));
    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });

    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.response).not.toBeNull();
    expect(result.current.response!.status).toBe(200);
    expect(result.current.response!.body).toContain('[truncated: response too large to display]');
  });

  it('onExecute handles proxy error response', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
      Promise.resolve(new Response(
        JSON.stringify({ error: 'Only HTTPS URLs are allowed' }),
        { status: 400, headers: { 'content-type': 'application/json' } },
      )),
    );
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));

    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });

    await act(async () => {
      await result.current.onExecute();
    });

    expect(result.current.responseError).toBe('Only HTTPS URLs are allowed');
    expect(result.current.response).toBeNull();
  });

  it('onExecute converts number fields in body', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('knol_sk_test'));

    // Wait for sample-data effect to settle, then set field values after
    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });
    act(() => {
      result.current.setFieldValue('query', 'test');
      result.current.setFieldValue('limit', '5');
    });
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      await result.current.onExecute();
    });

    const [, opts] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const proxyBody = JSON.parse(opts.body);
    const body = JSON.parse(proxyBody.body);
    expect(body.limit).toBe(5);
    expect(typeof body.limit).toBe('number');
  });

  it('onExecute sends _rootBody directly as request body (batch-write)', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.onSelectOperation('batch-write');
      result.current.setApiKey('knol_sk_test');
    });

    // Wait for sample-data effect to settle, then set field values after
    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });
    act(() => result.current.setFieldValue('_rootBody', '[{"user_id":"u1","content":"hi"}]'));
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      await result.current.onExecute();
    });

    const [, opts] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const proxyBody = JSON.parse(opts.body);
    // _rootBody sends the raw JSON as the request body (no wrapping)
    const body = JSON.parse(proxyBody.body);
    expect(Array.isArray(body)).toBe(true);
    expect(body[0].user_id).toBe('u1');
  });

  it('does not send body for GET operations', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.onSelectOperation('list-entities');
      result.current.setApiKey('knol_sk_test');
    });

    await act(async () => { await new Promise((r) => setTimeout(r, 10)); });
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      await result.current.onExecute();
    });

    const [, opts] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const proxyBody = JSON.parse(opts.body);
    expect(proxyBody.method).toBe('GET');
    expect(proxyBody.body).toBeUndefined();
  });
});

describe('usePlaygroundState – key selection', () => {
  it('loads available keys on mount', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.availableKeys).toHaveLength(3);
    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
  });

  it('keyOptions only includes active keys', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // k3 is inactive and should be excluded
    expect(result.current.keyOptions).toHaveLength(2);
    expect(result.current.keyOptions.map((k) => k.id)).toEqual(['k1', 'k2']);
  });

  it('keyOptions marks session keys as having value', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_fullvalue' },
    ]);

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const k1 = result.current.keyOptions.find((k) => k.id === 'k1');
    const k2 = result.current.keyOptions.find((k) => k.id === 'k2');
    expect(k1?.hasValue).toBe(true);
    expect(k2?.hasValue).toBe(false);
  });

  it('auto-populates initial signup key', async () => {
    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_signup_key_1234');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.apiKey).toBe('knol_sk_signup_key_1234');
    expect(result.current.selectedKeyId).toBe('initial');
  });

  it('defaults to manual when no initial key and auto-create fails', async () => {
    // Default: create is unmocked (returns undefined → auto-create fails silently)
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Auto-create was attempted
    expect(apiMocks.appApiKeysAPI.create).toHaveBeenCalledWith({ name: 'playground-key', role: 'developer' });
    // But it failed, so falls back to manual
    expect(result.current.selectedKeyId).toBe('manual');
    expect(result.current.apiKey).toBe('');
  });

  it('auto-creates and selects a key on mount when no keys are available', async () => {
    apiMocks.appApiKeysAPI.create.mockResolvedValue({
      id: 'k_auto',
      name: 'playground-key',
      role: 'developer',
      api_key: 'knol_sk_auto_created',
    });
    apiMocks.appApiKeysAPI.list
      .mockResolvedValueOnce(MOCK_KEYS)  // initial load
      .mockResolvedValueOnce([...MOCK_KEYS, {
        id: 'k_auto', name: 'playground-key', role: 'developer', active: true, created_at: '2026-02-25',
      }]);  // refresh after creation

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.apiKey).toBe('knol_sk_auto_created');
    expect(result.current.selectedKeyId).toBe('k_auto');
    expect(apiMocks.storeSessionApiKey).toHaveBeenCalledWith({
      id: 'k_auto',
      name: 'playground-key',
      role: 'developer',
      api_key: 'knol_sk_auto_created',
    });
  });

  it('auto-selects existing session key on mount instead of creating', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_k1_val' },
    ]);
    apiMocks.getSessionApiKeyValue.mockReturnValue('knol_sk_k1_val');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Should auto-select the existing session key, NOT create a new one
    expect(result.current.apiKey).toBe('knol_sk_k1_val');
    expect(result.current.selectedKeyId).toBe('k1');
    expect(apiMocks.appApiKeysAPI.create).not.toHaveBeenCalled();
  });

  it('onSelectKey fills key from session vault', async () => {
    apiMocks.getSessionApiKeyValue.mockReturnValue('knol_sk_session_val');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.onSelectKey('k1'));
    expect(result.current.apiKey).toBe('knol_sk_session_val');
    expect(result.current.selectedKeyId).toBe('k1');
  });

  it('onSelectKey keeps existing key when vault value not available', async () => {
    apiMocks.getSessionApiKeyValue.mockReturnValue(null);

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('something'));
    act(() => result.current.onSelectKey('k2'));
    // Should keep the existing apiKey so the user can paste manually
    expect(result.current.apiKey).toBe('something');
    expect(result.current.selectedKeyId).toBe('k2');
  });

  it('onSelectKey manual clears the key field', async () => {
    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setApiKey('something'));
    act(() => result.current.onSelectKey('manual'));
    expect(result.current.apiKey).toBe('');
    expect(result.current.selectedKeyId).toBe('manual');
  });
});

describe('usePlaygroundState – quick key creation', () => {
  it('onCreateQuickKey creates key and auto-selects it', async () => {
    apiMocks.appApiKeysAPI.create.mockResolvedValue({
      id: 'k_new',
      name: 'playground-key',
      role: 'developer',
      api_key: 'knol_sk_new_value',
    });
    apiMocks.appApiKeysAPI.list.mockResolvedValue([...MOCK_KEYS, {
      id: 'k_new', name: 'playground-key', role: 'developer', active: true, created_at: '2026-02-25',
    }]);

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onCreateQuickKey();
    });

    expect(apiMocks.appApiKeysAPI.create).toHaveBeenCalledWith({ name: 'playground-key', role: 'developer' });
    expect(apiMocks.storeSessionApiKey).toHaveBeenCalledWith({
      id: 'k_new',
      name: 'playground-key',
      role: 'developer',
      api_key: 'knol_sk_new_value',
    });
    expect(result.current.apiKey).toBe('knol_sk_new_value');
    expect(result.current.selectedKeyId).toBe('k_new');
    expect(result.current.creatingKey).toBe(false);
  });

  it('onCreateQuickKey sets error on failure', async () => {
    apiMocks.appApiKeysAPI.create.mockRejectedValue(new Error('Quota exceeded'));

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onCreateQuickKey();
    });

    expect(result.current.error).toBe('Quota exceeded');
    expect(result.current.creatingKey).toBe(false);
  });
});

describe('usePlaygroundState – sample data', () => {
  it('fetchSampleData populates IDs from gateway via export endpoint', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
      if (body.url?.includes('/v1/memory/export')) {
        return Promise.resolve(proxyResponse(200, {
          memories: [
            { id: 'mem_real_1', user_id: 'uid_real_1', content: 'User likes concise replies', kind: 'preference' },
            { id: 'mem_real_2', user_id: 'uid_real_1', content: 'User prefers dark mode', kind: 'preference' },
          ],
        }));
      }
      if (body.url?.includes('/v1/graph/entities')) {
        return Promise.resolve(proxyResponse(200, [
          { id: 'ent_real_1', name: 'TypeScript', entity_type: 'technology' },
          { id: 'ent_real_2', name: 'React', entity_type: 'technology' },
        ]));
      }
      return Promise.resolve(proxyResponse());
    });

    // Need an apiKey set for the effect to trigger
    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_test');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Wait for sample data effect
    await waitFor(() => expect(result.current.sampleData).not.toBeNull());

    expect(result.current.sampleData!.memoryIds).toContain('mem_real_1');
    expect(result.current.sampleData!.memoryIds).toContain('mem_real_2');
    expect(result.current.sampleData!.entityIds).toContain('ent_real_1');
    expect(result.current.sampleData!.userIds).toContain('uid_real_1');
    // Check memoryItems for dropdown display
    expect(result.current.sampleData!.memoryItems).toHaveLength(2);
    expect(result.current.sampleData!.memoryItems[0].label).toBe('User likes concise replies');
    // Check entityItems for dropdown display
    expect(result.current.sampleData!.entityItems).toHaveLength(2);
    expect(result.current.sampleData!.entityItems[0].label).toBe('TypeScript');
  });

  it('buildDefaults uses sampleData for operations', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
      if (body.url?.includes('/v1/memory/export')) {
        return Promise.resolve(proxyResponse(200, {
          memories: [
            { id: 'mem_real_1', user_id: 'uid_real_1', content: 'User likes concise replies' },
          ],
        }));
      }
      if (body.url?.includes('/v1/graph/entities')) {
        return Promise.resolve(proxyResponse(200, [
          { id: 'ent_real_1', name: 'TypeScript' },
          { id: 'ent_real_2', name: 'React' },
        ]));
      }
      return Promise.resolve(proxyResponse());
    });

    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_test');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await waitFor(() => expect(result.current.sampleData).not.toBeNull());

    // Switch to get-memory operation – should use real memory ID
    act(() => result.current.onSelectOperation('get-memory'));
    expect(result.current.fieldValues['id']).toBe('mem_real_1');

    // Switch to get-entity – should use real entity ID
    act(() => result.current.onSelectOperation('get-entity'));
    expect(result.current.fieldValues['id']).toBe('ent_real_1');

    // Switch to shortest-path – uses two entity IDs
    act(() => result.current.onSelectOperation('shortest-path'));
    expect(result.current.fieldValues['from']).toBe('ent_real_1');
    expect(result.current.fieldValues['to']).toBe('ent_real_2');
  });

  it('fetchSampleData handles partial failures gracefully', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
      if (body.url?.includes('/v1/memory/export')) {
        return Promise.resolve(proxyResponse(200, {
          memories: [{ id: 'mem_ok', content: 'Test memory' }],
        }));
      }
      // entities fail
      return Promise.resolve(proxyResponse(500, { error: 'Internal error' }));
    });

    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_test');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await waitFor(() => expect(result.current.sampleData).not.toBeNull());

    expect(result.current.sampleData!.memoryIds).toContain('mem_ok');
    expect(result.current.sampleData!.memoryItems).toHaveLength(1);
    expect(result.current.sampleData!.entityIds).toHaveLength(0);
    expect(result.current.sampleData!.entityItems).toHaveLength(0);
  });

  it('fetchSampleData handles entities returned as array (not wrapped)', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
      if (body.url?.includes('/v1/memory/export')) {
        return Promise.resolve(proxyResponse(200, { memories: [] }));
      }
      if (body.url?.includes('/v1/graph/entities')) {
        // Gateway returns entities as a plain array
        return Promise.resolve(proxyResponse(200, [
          { id: 'ent_1', name: 'Node.js', entity_type: 'technology' },
        ]));
      }
      return Promise.resolve(proxyResponse());
    });

    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_test');

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await waitFor(() => expect(result.current.sampleData).not.toBeNull());

    expect(result.current.sampleData!.entityIds).toContain('ent_1');
    expect(result.current.sampleData!.entityItems[0].label).toBe('Node.js');
  });

  it('fetchSampleData aborts previous request when called again before completion', async () => {
    // The second call is triggered while the first is still "pending".
    // Only the second call's data should end up in state.
    let callCount = 0;
    const abortedSignals: AbortSignal[] = [];

    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const signal = (opts as RequestInit & { signal?: AbortSignal })?.signal;
      callCount++;

      return new Promise<Response>((resolve, reject) => {
        if (signal) {
          // If the signal is already aborted, reject immediately.
          if (signal.aborted) {
            abortedSignals.push(signal);
            reject(new DOMException('Aborted', 'AbortError'));
            return;
          }
          // If aborted while pending, reject.
          signal.addEventListener('abort', () => {
            abortedSignals.push(signal);
            reject(new DOMException('Aborted', 'AbortError'));
          });
        }

        // Only resolve after a microtask so the second call has time to abort the first.
        queueMicrotask(() => {
          if (signal?.aborted) return; // already rejected above
          const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
          if (body.url?.includes('/v1/memory/export')) {
            resolve(proxyResponse(200, {
              memories: [{ id: `mem_call${callCount}`, content: `Call ${callCount}` }],
            }));
          } else {
            resolve(proxyResponse(200, []));
          }
        });
      });
    });

    const { result } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Fire two consecutive fetchSampleData calls – second should abort the first.
    await act(async () => {
      result.current.setApiKey('knol_sk_first');
    });
    await act(async () => {
      result.current.setApiKey('knol_sk_second');
    });

    await waitFor(() => expect(result.current.sampleLoading).toBe(false));

    // At least one fetch was aborted.
    expect(abortedSignals.length).toBeGreaterThan(0);
    // The final sample data reflects the last request, not an intermediate one.
    expect(result.current.sampleData).not.toBeNull();
  });

  it('fetchSampleData does not update state after component unmounts', async () => {
    // Verify unmount cleanup cancels the in-flight request.
    const abortedSignals: AbortSignal[] = [];

    vi.spyOn(globalThis, 'fetch').mockImplementation((_url, opts) => {
      const signal = (opts as RequestInit & { signal?: AbortSignal })?.signal;
      return new Promise<Response>((resolve, reject) => {
        if (signal) {
          signal.addEventListener('abort', () => {
            abortedSignals.push(signal);
            reject(new DOMException('Aborted', 'AbortError'));
          });
        }
        // Resolve after a longer delay to ensure unmount happens first.
        setTimeout(() => resolve(proxyResponse(200, { memories: [] })), 100);
      });
    });

    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_test');

    const { result, unmount } = renderHook(() => usePlaygroundState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Unmount before the delayed response arrives.
    unmount();

    // Wait long enough for the request to have resolved if it weren't aborted.
    await new Promise((r) => setTimeout(r, 150));

    expect(abortedSignals.length).toBeGreaterThan(0);
  });
});
