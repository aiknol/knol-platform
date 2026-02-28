/**
 * E2E-style tests for the API Keys page.
 * These simulate complete user flows: creating keys, toggling visibility,
 * and verifying the key lifecycle.
 */
import React from 'react';
globalThis.React = React;
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
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

import ApiKeysPage from './page';

const MOCK_ME = {
  user: { id: 'u1', email: 'owner@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'builder' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'prod-key', role: 'admin', active: true, created_at: '2026-01-01', key_prefix: 'knol_sk_...ab12' },
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
    api_key: 'knol_sk_e2e_test_key_ABCD',
  });
  apiMocks.appApiKeysAPI.revoke.mockResolvedValue({});
  vi.spyOn(window, 'confirm').mockReturnValue(true);

  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.assign(navigator, { clipboard: { writeText } });
});

describe('API Keys E2E – Create and toggle visibility', () => {
  it('full flow: create key, see masked, reveal, hide, copy', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    // Step 1: Create a key
    fireEvent.submit(screen.getByText('Create Key').closest('form')!);

    await waitFor(() => {
      expect(screen.getByText('Your API key (shown once)')).toBeTruthy();
    });

    // Step 2: Key should be masked (bullets + last 4 chars "ABCD")
    const codeEl = screen.getByText(/ABCD$/);
    expect(codeEl.textContent).toContain('\u2022');
    expect(codeEl.textContent).not.toContain('knol_sk_e2e_test_key_ABCD');

    // Step 3: Click Reveal to show full key
    fireEvent.click(screen.getByText('Reveal'));
    expect(screen.getByText('knol_sk_e2e_test_key_ABCD')).toBeTruthy();

    // Step 4: Click Hide to mask again (in banner)
    const bannerHide = screen.getByText('Hide');
    fireEvent.click(bannerHide);
    expect(screen.queryByText('knol_sk_e2e_test_key_ABCD')).toBeNull();
    expect(screen.getByText('Reveal')).toBeTruthy();

    // Step 5: Copy (should copy full key regardless of visibility)
    fireEvent.click(screen.getByText('Copy'));
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('knol_sk_e2e_test_key_ABCD');

    // Step 6: Verify key was stored in session vault
    expect(apiMocks.storeSessionApiKey).toHaveBeenCalledWith({
      id: 'k_new',
      name: 'new-key',
      role: 'developer',
      api_key: 'knol_sk_e2e_test_key_ABCD',
    });
  });
});

describe('API Keys E2E – Create key resets visibility', () => {
  it('creating a second key resets toggle to masked state', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    // Create first key
    fireEvent.submit(screen.getByText('Create Key').closest('form')!);
    await waitFor(() => expect(screen.getByText('Reveal')).toBeTruthy());

    // Reveal it
    fireEvent.click(screen.getByText('Reveal'));

    // Create second key
    apiMocks.appApiKeysAPI.create.mockResolvedValue({
      id: 'k_new2',
      name: 'second-key',
      role: 'admin',
      api_key: 'knol_sk_second_key_WXYZ',
    });

    // Find the create form (the section after the banner)
    const forms = document.querySelectorAll('form');
    fireEvent.submit(forms[0]);

    await waitFor(() => {
      // Should show Reveal (not Hide in banner) since visibility was reset
      expect(screen.getByText('Reveal')).toBeTruthy();
    });
  });
});

describe('API Keys E2E – Revoke key flow', () => {
  it('revokes a key and refreshes the list', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    // Click Revoke
    fireEvent.click(screen.getByText('Revoke'));

    expect(window.confirm).toHaveBeenCalledWith(
      'Revoke this API key? Existing integrations using it will stop working.',
    );
    expect(apiMocks.appApiKeysAPI.revoke).toHaveBeenCalledWith('k1');

    // List was refreshed
    await waitFor(() => {
      expect(apiMocks.appApiKeysAPI.list).toHaveBeenCalledTimes(2);
    });
  });
});

describe('API Keys E2E – Initial key from signup', () => {
  it('shows initial key masked after signup redirect', async () => {
    apiMocks.consumeInitialApiKey.mockReturnValue('knol_sk_signup_initial_key_XY99');

    render(<ApiKeysPage />);
    await waitFor(() => {
      expect(screen.getByText('Your API key (shown once)')).toBeTruthy();
    });

    // Should be masked with last 4 chars "XY99"
    const codeEl = screen.getByText(/XY99$/);
    expect(codeEl.textContent).toContain('\u2022');

    // Reveal shows full key
    fireEvent.click(screen.getByText('Reveal'));
    expect(screen.getByText('knol_sk_signup_initial_key_XY99')).toBeTruthy();
  });
});

describe('API Keys E2E – Error recovery', () => {
  it('shows error on create failure, allows retry', async () => {
    apiMocks.appApiKeysAPI.create
      .mockRejectedValueOnce(new Error('Rate limited'))
      .mockResolvedValueOnce({
        id: 'k_retry',
        name: 'integration-key',
        role: 'developer',
        api_key: 'knol_sk_retry_success_1234',
      });

    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    // First attempt fails
    fireEvent.submit(screen.getByText('Create Key').closest('form')!);
    await waitFor(() => {
      expect(screen.getByText('Rate limited')).toBeTruthy();
    });

    // Retry succeeds
    fireEvent.submit(screen.getByText('Create Key').closest('form')!);
    await waitFor(() => {
      expect(screen.getByText('Your API key (shown once)')).toBeTruthy();
    });
  });
});

describe('API Keys E2E – Per-key prefix visibility', () => {
  it('show/hide prefix in key list', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    // Prefix should be masked by default
    const showBtn = screen.getByText('Show');
    expect(showBtn).toBeTruthy();

    // Click Show to reveal prefix
    fireEvent.click(showBtn);
    expect(screen.getByText('knol_sk_...ab12')).toBeTruthy();

    // Click Hide to mask again
    // Note: there may be a "Hide" from both the banner toggle context and the list.
    // We need the list-level Hide button.
    const hideBtn = screen.getByText('Hide');
    fireEvent.click(hideBtn);
    expect(screen.queryByText('knol_sk_...ab12')).toBeNull();
  });
});
