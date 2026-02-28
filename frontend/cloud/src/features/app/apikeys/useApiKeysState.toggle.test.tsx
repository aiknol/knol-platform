import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  consumeInitialApiKey: vi.fn(),
  storeSessionApiKey: vi.fn(),
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
  storeSessionApiKey: apiMocks.storeSessionApiKey,
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
  { id: 'k1', name: 'dev-key', role: 'developer', active: true, created_at: '2026-01-01', key_prefix: 'knol_sk_...ab12' },
];

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_ME.user);
  apiMocks.consumeInitialApiKey.mockReturnValue(null);
  apiMocks.appAuthAPI.me.mockResolvedValue(MOCK_ME);
  apiMocks.appApiKeysAPI.list.mockResolvedValue(MOCK_KEYS);
  apiMocks.appApiKeysAPI.create.mockResolvedValue({
    id: 'k_new',
    name: 'new-key',
    role: 'developer',
    api_key: 'knol_live_newkey123',
  });
  vi.spyOn(window, 'confirm').mockReturnValue(true);
});

describe('useApiKeysState – key visibility toggle', () => {
  it('keyVisible defaults to false', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.keyVisible).toBe(false);
  });

  it('toggleKeyVisibility toggles keyVisible', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.toggleKeyVisibility());
    expect(result.current.keyVisible).toBe(true);

    act(() => result.current.toggleKeyVisibility());
    expect(result.current.keyVisible).toBe(false);
  });

  it('creating a new key resets keyVisible to false', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Toggle to visible
    act(() => result.current.toggleKeyVisibility());
    expect(result.current.keyVisible).toBe(true);

    // Create a new key
    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    // Should be reset to false
    expect(result.current.keyVisible).toBe(false);
    expect(result.current.newlyCreatedApiKey).toBe('knol_live_newkey123');
  });

  it('creating a key stores it in session vault', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateKey(mockEvent);
    });

    expect(apiMocks.storeSessionApiKey).toHaveBeenCalledWith({
      id: 'k_new',
      name: 'new-key',
      role: 'developer',
      api_key: 'knol_live_newkey123',
    });
  });

  it('keyVisible and toggleKeyVisibility are returned from hook', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect('keyVisible' in result.current).toBe(true);
    expect('toggleKeyVisibility' in result.current).toBe(true);
    expect(typeof result.current.toggleKeyVisibility).toBe('function');
  });
});

describe('useApiKeysState – per-key list visibility', () => {
  it('all keys are hidden by default', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.isListKeyVisible('k1')).toBe(false);
  });

  it('toggleListKeyVisibility reveals and hides a key', async () => {
    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.toggleListKeyVisibility('k1'));
    expect(result.current.isListKeyVisible('k1')).toBe(true);

    act(() => result.current.toggleListKeyVisibility('k1'));
    expect(result.current.isListKeyVisible('k1')).toBe(false);
  });

  it('toggleListKeyVisibility works independently per key', async () => {
    apiMocks.appApiKeysAPI.list.mockResolvedValue([
      ...MOCK_KEYS,
      { id: 'k2', name: 'key-2', role: 'admin', active: true, created_at: '2026-02-01', key_prefix: 'knol_sk_...cd34' },
    ]);

    const { result } = renderHook(() => useApiKeysState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.toggleListKeyVisibility('k1'));
    expect(result.current.isListKeyVisible('k1')).toBe(true);
    expect(result.current.isListKeyVisible('k2')).toBe(false);
  });
});
