/**
 * E2E-style tests for the Playground page.
 * These simulate complete user flows: selecting operations, filling fields,
 * sending requests, and verifying the full response cycle.
 */
import React from 'react';
globalThis.React = React;
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
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

import PlaygroundPage from './page';

const MOCK_ME = {
  user: { id: 'u1', email: 'dev@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'builder' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'prod-key', role: 'admin', active: true, created_at: '2026-01-01' },
];

const MOCK_RESPONSE_BODY = { results: [{ id: 'mem_1', content: 'User prefers dark mode' }] };

/** Find the onExecute proxy call (excludes sample-data fetches). */
function findExecuteCall(): [string, RequestInit] | undefined {
  const mockFn = globalThis.fetch as ReturnType<typeof vi.fn>;
  // Iterate in reverse so the most recent execute call is found first
  for (let i = mockFn.mock.calls.length - 1; i >= 0; i--) {
    const [url, opts] = mockFn.mock.calls[i];
    if (url !== '/api/playground/proxy') continue;
    const body = JSON.parse((opts as RequestInit)?.body as string || '{}');
    // Sample-data calls use export/entities/audit paths
    const isSampleData = body.url?.includes('/v1/memory/export')
      || body.url?.includes('/v1/graph/entities?limit=');
    if (!isSampleData) return [url, opts as RequestInit];
  }
  return undefined;
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
  // Fresh Response per call (body can only be consumed once)
  vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
    Promise.resolve(new Response(
      JSON.stringify({ status: 200, body: JSON.stringify(MOCK_RESPONSE_BODY) }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    )),
  );
});

describe('Playground E2E – Search Memory flow', () => {
  it('completes full search flow: enter key, fill query, send, see response', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Step 1: Enter API key
    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_live_abc' } });

    // Step 2: Fill search fields
    fireEvent.change(screen.getByLabelText('Query *'), { target: { value: 'user preferences' } });
    fireEvent.change(screen.getByLabelText('User ID'), { target: { value: '550e8400-e29b-41d4-a716-446655440000' } });

    // Step 3: Send request
    fireEvent.click(screen.getByText('Send Request'));

    // Step 4: Verify response
    await waitFor(() => {
      expect(screen.getByText('200')).toBeTruthy();
    });

    // Verify proxy was called with correct params
    const call = findExecuteCall();
    expect(call).toBeDefined();
    const [proxyUrl, proxyOpts] = call!;
    expect(proxyUrl).toBe('/api/playground/proxy');
    const proxy = JSON.parse((proxyOpts as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/search');
    expect(proxy.method).toBe('POST');
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_live_abc');
    const body = JSON.parse(proxy.body);
    expect(body.query).toBe('user preferences');
    expect(body.user_id).toBe('550e8400-e29b-41d4-a716-446655440000');

    // Response body is displayed
    expect(screen.getByText(/User prefers dark mode/)).toBeTruthy();
  });
});

describe('Playground E2E – Write Memory flow', () => {
  it('switches to write-memory, fills fields, sends POST', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Enter key
    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });

    // Switch to Write Memory
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'write-memory' } });
    expect(screen.getByText('Store a new memory.')).toBeTruthy();

    // Fill fields
    fireEvent.change(screen.getByLabelText('User ID'), { target: { value: '550e8400-e29b-41d4-a716-446655440000' } });
    fireEvent.change(screen.getByLabelText('Content *'), { target: { value: 'Likes TypeScript' } });

    // Send
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => {
      expect(screen.getByText('200')).toBeTruthy();
    });

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory');
    expect(proxy.method).toBe('POST');
    const body = JSON.parse(proxy.body);
    expect(body.user_id).toBe('550e8400-e29b-41d4-a716-446655440000');
    expect(body.content).toBe('Likes TypeScript');
  });
});

describe('Playground E2E – Get Memory with path param', () => {
  it('sends GET with memory ID in URL path', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'get-memory' } });
    fireEvent.change(screen.getByLabelText('Memory ID *'), { target: { value: 'mem_xyz789' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/mem_xyz789');
    expect(proxy.method).toBe('GET');
    expect(proxy.body).toBeUndefined();
  });
});

describe('Playground E2E – Graph shortest path', () => {
  it('sends GET with two path params', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'shortest-path' } });
    fireEvent.change(screen.getByLabelText('From Entity ID *'), { target: { value: 'ent_a' } });
    fireEvent.change(screen.getByLabelText('To Entity ID *'), { target: { value: 'ent_b' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/graph/path/ent_a/ent_b');
  });
});

describe('Playground E2E – Batch Write flow', () => {
  it('sends raw JSON array body for batch-write (_rootBody)', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'batch-write' } });
    expect(screen.getByText('Store multiple memories in one request.')).toBeTruthy();

    fireEvent.change(screen.getByLabelText('Memories (JSON array) *'), {
      target: { value: '[{"user_id":"u1","content":"hi"},{"user_id":"u2","content":"bye"}]' },
    });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/batch');
    expect(proxy.method).toBe('POST');
    // Body should be a raw array, NOT wrapped in {"_rootBody": ...}
    const body = JSON.parse(proxy.body);
    expect(Array.isArray(body)).toBe(true);
    expect(body).toHaveLength(2);
    expect(body[0].user_id).toBe('u1');
    expect(body[1].content).toBe('bye');
  });
});

describe('Playground E2E – Update Memory flow', () => {
  it('sends PUT with path param and body', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'update-memory' } });
    expect(screen.getByText('Update an existing memory.')).toBeTruthy();

    fireEvent.change(screen.getByLabelText('Memory ID *'), { target: { value: 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee' } });
    fireEvent.change(screen.getByLabelText('Content *'), { target: { value: 'Updated preference' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee');
    expect(proxy.method).toBe('PUT');
    const body = JSON.parse(proxy.body);
    expect(body.content).toBe('Updated preference');
  });
});

describe('Playground E2E – Delete Memory flow', () => {
  it('sends DELETE with memory UUID in path', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'delete-memory' } });
    fireEvent.change(screen.getByLabelText('Memory ID *'), { target: { value: 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee');
    expect(proxy.method).toBe('DELETE');
    expect(proxy.body).toBeUndefined();
  });
});

describe('Playground E2E – List Entities flow', () => {
  it('sends GET with no body to list entities', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'list-entities' } });
    expect(screen.getByText('List all graph entities.')).toBeTruthy();

    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/graph/entities');
    expect(proxy.method).toBe('GET');
    expect(proxy.body).toBeUndefined();
  });
});

describe('Playground E2E – Get Entity flow', () => {
  it('sends GET with entity UUID in path', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'get-entity' } });
    fireEvent.change(screen.getByLabelText('Entity ID *'), { target: { value: 'aaaaaaaa-1111-2222-3333-444444444444' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/graph/entities/aaaaaaaa-1111-2222-3333-444444444444');
    expect(proxy.method).toBe('GET');
  });
});

describe('Playground E2E – Get Edges flow', () => {
  it('sends GET with entity UUID to get edges', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'get-edges' } });
    fireEvent.change(screen.getByLabelText('Entity ID *'), { target: { value: 'bbbbbbbb-1111-2222-3333-444444444444' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/graph/entities/bbbbbbbb-1111-2222-3333-444444444444/edges');
    expect(proxy.method).toBe('GET');
  });
});

describe('Playground E2E – Create Webhook flow', () => {
  it('sends POST with URL and events JSON', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'create-webhook' } });
    expect(screen.getByText('Register a webhook for events.')).toBeTruthy();

    fireEvent.change(screen.getByLabelText('Webhook URL *'), { target: { value: 'https://hooks.example.com/test' } });
    fireEvent.change(screen.getByLabelText('Events (JSON array)'), {
      target: { value: '["memory.created","memory.deleted"]' },
    });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/webhooks');
    expect(proxy.method).toBe('POST');
    const body = JSON.parse(proxy.body);
    expect(body.url).toBe('https://hooks.example.com/test');
    expect(body.events).toEqual(['memory.created', 'memory.deleted']);
  });
});

describe('Playground E2E – Audit Log flow', () => {
  it('sends GET to view audit log', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'audit-log' } });
    expect(screen.getByText('View the audit log.')).toBeTruthy();

    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/admin/audit');
    expect(proxy.method).toBe('GET');
    expect(proxy.body).toBeUndefined();
  });
});

describe('Playground E2E – Auto-key creation on load', () => {
  it('auto-creates and selects a key when no usable keys exist', async () => {
    apiMocks.appApiKeysAPI.create.mockResolvedValue({
      id: 'k_auto',
      name: 'playground-key',
      role: 'developer',
      api_key: 'knol_sk_auto_created',
    });
    apiMocks.appApiKeysAPI.list
      .mockResolvedValueOnce(MOCK_KEYS)
      .mockResolvedValueOnce([...MOCK_KEYS, {
        id: 'k_auto', name: 'playground-key', role: 'developer', active: true, created_at: '2026-02-25',
      }]);
    apiMocks.getSessionApiKeys
      .mockReturnValueOnce([])   // during init
      .mockReturnValue([         // after key creation
        { id: 'k_auto', name: 'playground-key', role: 'developer', api_key: 'knol_sk_auto_created' },
      ]);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Key should be auto-created and selected
    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    await waitFor(() => expect(keyInput.value).toBe('knol_sk_auto_created'));

    // Should be able to send a request immediately
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());
  });
});

describe('Playground E2E – Error handling flow', () => {
  it('displays error response for 401', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
      Promise.resolve(new Response(JSON.stringify({
        status: 401,
        body: JSON.stringify({ error: 'Invalid API key' }),
      }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })),
    );

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'bad_key' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => {
      expect(screen.getByText('401')).toBeTruthy();
    });
    expect(screen.getAllByText(/Invalid API key/).length).toBeGreaterThan(0);
  });

  it('displays network error message', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('Failed to fetch'));

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => {
      expect(screen.getByText('Failed to fetch')).toBeTruthy();
    });
  });
});

describe('Playground E2E – API key toggle during workflow', () => {
  it('key stays entered when toggling visibility', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const input = screen.getByLabelText('API Key') as HTMLInputElement;
    fireEvent.change(input, { target: { value: 'knol_sk_secret123' } });

    // Toggle show
    fireEvent.click(screen.getByText('Show'));
    expect(input.type).toBe('text');
    expect(input.value).toBe('knol_sk_secret123');

    // Toggle hide
    fireEvent.click(screen.getByText('Hide'));
    expect(input.type).toBe('password');
    expect(input.value).toBe('knol_sk_secret123');

    // Key still works for sending
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_secret123');
  });
});

describe('Playground E2E – Operation switching preserves key', () => {
  it('API key persists when switching operations', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    fireEvent.change(keyInput, { target: { value: 'knol_sk_persistent' } });

    // Switch operations
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'get-memory' } });
    expect(keyInput.value).toBe('knol_sk_persistent');

    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'list-entities' } });
    expect(keyInput.value).toBe('knol_sk_persistent');

    // Send request with persisted key
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_persistent');
  });
});

describe('Playground E2E – Signup key auto-population flow', () => {
  it('uses initial signup key for API request', async () => {
    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_signup_initial_XXXX');

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Key should be auto-populated
    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(keyInput.value).toBe('knol_sk_signup_initial_XXXX');

    // Fill search fields and send
    fireEvent.change(screen.getByLabelText('Query *'), { target: { value: 'test query' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_signup_initial_XXXX');
  });
});

describe('Playground E2E – Key selector with session keys', () => {
  it('selects a session key, sends request, then switches to manual', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_session_full' },
    ]);
    apiMocks.getSessionApiKeyValue.mockReturnValue('knol_sk_session_full');

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Select the session key
    const selector = screen.getByLabelText('Select API Key') as HTMLSelectElement;
    fireEvent.change(selector, { target: { value: 'k1' } });

    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(keyInput.value).toBe('knol_sk_session_full');

    // Send request
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_session_full');

    // Switch back to manual
    fireEvent.change(selector, { target: { value: 'manual' } });
    expect(keyInput.value).toBe('');
  });
});

describe('Playground E2E – Unavailable keys hidden from dropdown', () => {
  it('hides keys without vault values and shows guidance message', async () => {
    // Keys exist in backend but NOT in session vault
    apiMocks.getSessionApiKeys.mockReturnValue([]);
    apiMocks.getSessionApiKeyValue.mockReturnValue(null);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Keys should NOT appear in the dropdown
    expect(screen.queryByText(/prod-key/)).toBeNull();

    // Guidance message should be visible
    expect(screen.getByText(/existing keys are not available/)).toBeTruthy();
    expect(screen.getByText('Quick Create Key')).toBeTruthy();
  });

  it('manual entry flow: paste key → send succeeds', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([]);
    apiMocks.getSessionApiKeyValue.mockReturnValue(null);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Paste key manually
    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    fireEvent.change(keyInput, { target: { value: 'knol_sk_pasted_full_key' } });
    expect(keyInput.value).toBe('knol_sk_pasted_full_key');

    // Send request → should succeed
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toBe('http://localhost:3000/v1/memory/search');
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_pasted_full_key');
  });

  it('manual entry → switch operation → key persists → send succeeds', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([]);
    apiMocks.getSessionApiKeyValue.mockReturnValue(null);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Paste key manually
    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    fireEvent.change(keyInput, { target: { value: 'knol_sk_my_key' } });

    // Switch to a different operation
    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'delete-memory' } });
    expect(screen.getByText('Delete a memory by ID.')).toBeTruthy();

    // Key should persist
    expect(keyInput.value).toBe('knol_sk_my_key');

    // Fill required path param
    fireEvent.change(screen.getByLabelText('Memory ID *'), { target: { value: 'mem_xyz789' } });

    // Send → works
    fireEvent.click(screen.getByText('Send Request'));
    await waitFor(() => expect(screen.getByText('200')).toBeTruthy());

    const call = findExecuteCall();
    expect(call).toBeDefined();
    const proxy = JSON.parse((call![1] as RequestInit).body as string);
    expect(proxy.url).toContain('/v1/memory/');
    expect(proxy.headers['Authorization']).toBe('Bearer knol_sk_my_key');
    expect(proxy.method).toBe('DELETE');
  });

  it('no guidance message when session keys are available', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_session_full' },
    ]);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Session key appears in the dropdown
    expect(screen.getByText(/prod-key \(admin\)/)).toBeTruthy();

    // Guidance message should NOT appear (at least one usable key exists)
    expect(screen.queryByText(/existing keys are not available/)).toBeNull();
  });
});
