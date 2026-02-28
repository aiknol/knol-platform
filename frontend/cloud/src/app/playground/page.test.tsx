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
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'prod-key', role: 'admin', active: true, created_at: '2026-01-01' },
  { id: 'k2', name: 'dev-key', role: 'developer', active: true, created_at: '2026-01-15' },
];

beforeEach(() => {
  vi.clearAllMocks();
  globalThis.localStorage?.clear?.();
  apiMocks.appAuthAPI.me.mockResolvedValue(MOCK_ME);
  apiMocks.appApiKeysAPI.list.mockResolvedValue(MOCK_KEYS);
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_ME.user);
  apiMocks.getInitialApiKey.mockReturnValue(null);
  apiMocks.getSessionApiKeys.mockReturnValue([]);
  apiMocks.getSessionApiKeyValue.mockReturnValue(null);
  // Proxy returns { status, body } JSON – fresh Response per call
  vi.spyOn(globalThis, 'fetch').mockImplementation(() =>
    Promise.resolve(new Response(
      JSON.stringify({ status: 200, body: JSON.stringify({ results: [] }) }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    )),
  );
});

describe('PlaygroundPage', () => {
  it('shows loading state initially', () => {
    render(<PlaygroundPage />);
    expect(screen.getByText('Loading playground...')).toBeTruthy();
  });

  it('renders page header after loading', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => {
      expect(screen.getByText('Playground')).toBeTruthy();
    });
    expect(screen.getByText('Test your API keys against the Knol gateway.')).toBeTruthy();
  });

  it('renders API key input as password type by default', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const input = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(input.type).toBe('password');
  });

  it('toggles API key visibility when Show/Hide clicked', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const input = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(input.type).toBe('password');

    fireEvent.click(screen.getByText('Show'));
    expect(input.type).toBe('text');
    expect(screen.getByText('Hide')).toBeTruthy();

    fireEvent.click(screen.getByText('Hide'));
    expect(input.type).toBe('password');
  });

  it('displays gateway URL', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());
    expect(screen.getByText('http://localhost:3000')).toBeTruthy();
  });

  it('renders operation selector with default operation', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const select = screen.getByLabelText('Operation') as HTMLSelectElement;
    expect(select.value).toBe('search-memory');
    expect(screen.getByText('Semantic search across stored memories.')).toBeTruthy();
  });

  it('renders dynamic fields for search-memory', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    expect(screen.getByLabelText('Query *')).toBeTruthy();
    expect(screen.getByLabelText('User ID')).toBeTruthy();
    expect(screen.getByLabelText('Limit')).toBeTruthy();
  });

  it('changes fields when operation changes', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('Operation'), { target: { value: 'get-memory' } });

    expect(screen.getByLabelText('Memory ID *')).toBeTruthy();
    expect(screen.queryByLabelText('Query *')).toBeNull();
    expect(screen.getByText('Retrieve a single memory by ID.')).toBeTruthy();
  });

  it('renders Send Request button', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    expect(screen.getByText('Send Request')).toBeTruthy();
  });

  it('shows empty state in response panel', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    expect(screen.getByText('Send a request to see the response here.')).toBeTruthy();
  });

  it('shows error when sending without API key', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => {
      expect(screen.getByText('Please enter or select an API key.')).toBeTruthy();
    });
  });

  it('displays response after successful request', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    fireEvent.change(screen.getByLabelText('API Key'), { target: { value: 'knol_sk_test' } });
    fireEvent.click(screen.getByText('Send Request'));

    await waitFor(() => {
      expect(screen.getByText('200')).toBeTruthy();
    });
    expect(screen.queryByText('Send a request to see the response here.')).toBeNull();
  });

  it('shows error alert when API load fails', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValue(new Error('Server error'));
    render(<PlaygroundPage />);
    await waitFor(() => {
      expect(screen.getByText('Server error')).toBeTruthy();
    });
  });

  it('renders Request and Response headings', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    expect(screen.getByText('Request')).toBeTruthy();
    expect(screen.getByText('Response')).toBeTruthy();
  });
});

describe('PlaygroundPage – key selector', () => {
  it('renders key selector dropdown', async () => {
    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const selector = screen.getByLabelText('Select API Key') as HTMLSelectElement;
    expect(selector).toBeTruthy();
    expect(selector.value).toBe('manual');
  });

  it('shows session keys in the dropdown and hides unavailable keys', async () => {
    // Only k1 is in the session vault, k2 is not
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_full' },
    ]);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // k1 has a vault value so it appears; k2 does not
    expect(screen.getByText(/prod-key \(admin\)/)).toBeTruthy();
    expect(screen.queryByText(/dev-key \(developer\)/)).toBeNull();
  });

  it('shows guidance message when all keys lack vault values', async () => {
    // No session keys at all — both keys are unavailable
    apiMocks.getSessionApiKeys.mockReturnValue([]);

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    // Neither key appears in the dropdown
    expect(screen.queryByText(/prod-key/)).toBeNull();
    expect(screen.queryByText(/dev-key/)).toBeNull();

    // Guidance message and quick create button show up
    expect(screen.getByText(/existing keys are not available/)).toBeTruthy();
    expect(screen.getByText('Quick Create Key')).toBeTruthy();
  });

  it('auto-populates initial signup key', async () => {
    apiMocks.getInitialApiKey.mockReturnValue('knol_sk_signup_abc123');

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(keyInput.value).toBe('knol_sk_signup_abc123');

    const selector = screen.getByLabelText('Select API Key') as HTMLSelectElement;
    expect(selector.value).toBe('initial');
  });

  it('fills key when selecting a session key', async () => {
    apiMocks.getSessionApiKeys.mockReturnValue([
      { id: 'k1', name: 'prod-key', role: 'admin', api_key: 'knol_sk_full_value' },
    ]);
    apiMocks.getSessionApiKeyValue.mockReturnValue('knol_sk_full_value');

    render(<PlaygroundPage />);
    await waitFor(() => expect(screen.getByText('Playground')).toBeTruthy());

    const selector = screen.getByLabelText('Select API Key') as HTMLSelectElement;
    fireEvent.change(selector, { target: { value: 'k1' } });

    const keyInput = screen.getByLabelText('API Key') as HTMLInputElement;
    expect(keyInput.value).toBe('knol_sk_full_value');
  });
});
