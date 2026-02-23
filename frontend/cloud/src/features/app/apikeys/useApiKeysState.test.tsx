import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  consumeInitialApiKey: vi.fn(),
  appAuthAPI: {
    me: vi.fn(),
  },
  appApiKeysAPI: {
    list: vi.fn(),
    create: vi.fn(),
    revoke: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  getAppAuthUser: apiMocks.getAppAuthUser,
  consumeInitialApiKey: apiMocks.consumeInitialApiKey,
  appAuthAPI: apiMocks.appAuthAPI,
  appApiKeysAPI: apiMocks.appApiKeysAPI,
}));

import { useApiKeysState } from './useApiKeysState';

const MOCK_ME = {
  user: { id: 'u1', email: 'owner@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'integration-key', role: 'developer', active: true, created_at: '2026-01-01' },
  { id: 'k2', name: 'admin-key', role: 'admin', active: true, created_at: '2026-01-02' },
];

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_ME.user);
  apiMocks.consumeInitialApiKey.mockReturnValue(null);
  apiMocks.appAuthAPI.me.mockResolvedValue(MOCK_ME);
  apiMocks.appApiKeysAPI.list.mockResolvedValue(MOCK_KEYS);
  apiMocks.appApiKeysAPI.create.mockResolvedValue({ api_key: 'knol_live_newkey123' });
  apiMocks.appApiKeysAPI.revoke.mockResolvedValue({});

  vi.spyOn(window, 'confirm').mockReturnValue(true);
});

describe('useApiKeysState', () => {
  it('loads keys and gateway URL', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.error).toBe('');
    expect(result.current.keys).toHaveLength(2);
    expect(result.current.gatewayBaseUrl).toBe('https://gateway.example.com');
    expect(apiMocks.appAuthAPI.me).toHaveBeenCalledTimes(1);
    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(1);
  });

  it('consumes initial API key on mount', async () => {
    apiMocks.consumeInitialApiKey.mockReturnValue('knol_live_initial123');
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.newlyCreatedApiKey).toBe('knol_live_initial123');
  });

  it('onCreateKey with expiry', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    expect(apiMocks.appApiKeysAPI.create).toHaveBeenCalledWith({
      name: 'integration-key',
      role: 'developer',
      expires_in_days: 30,
    });
  });

  it('onCreateKey without expiry when empty', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.setNewExpiryDays('');
    });

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    // expires_in_days should NOT be in payload when expiry is empty/NaN
    expect(apiMocks.appApiKeysAPI.create).toHaveBeenCalledWith({
      name: 'integration-key',
      role: 'developer',
    });
  });

  it('onCreateKey sets newlyCreatedApiKey', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    expect(result.current.newlyCreatedApiKey).toBe('knol_live_newkey123');
  });

  it('onRevoke calls revoke and reloads', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onRevoke('k1');
    });

    expect(apiMocks.appApiKeysAPI.revoke).toHaveBeenCalledWith('k1');
    // list called twice: initial load + after revoke
    expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(2);
  });

  it('error on load failure', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValue(new Error('Network error'));
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.error).toBe('Network error');
  });

  it('error on create failure', async () => {
    apiMocks.appApiKeysAPI.create.mockRejectedValue(new Error('Key limit reached'));
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    expect(result.current.error).toBe('Key limit reached');
  });

  it('copyToClipboard writes to clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.copyToClipboard('knol_live_abc');
    });

    expect(writeText).toHaveBeenCalledWith('knol_live_abc');
  });

  it('no initial key when consumeInitialApiKey returns null', async () => {
    apiMocks.consumeInitialApiKey.mockReturnValue(null);
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.newlyCreatedApiKey).toBeNull();
  });
});
