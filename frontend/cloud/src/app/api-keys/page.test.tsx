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
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  gateway_base_url: 'https://gateway.example.com',
};

const MOCK_KEYS = [
  { id: 'k1', name: 'prod-key', role: 'admin', active: true, created_at: '2026-01-01', key_prefix: 'knol_sk_...ab12' },
  { id: 'k2', name: 'dev-key', role: 'developer', active: true, created_at: '2026-01-15', key_prefix: 'knol_sk_...cd34' },
];

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_ME.user);
  apiMocks.consumeInitialApiKey.mockReturnValue(null);
  apiMocks.appAuthAPI.me.mockResolvedValue(MOCK_ME);
  apiMocks.appApiKeysAPI.list.mockResolvedValue(MOCK_KEYS);
  apiMocks.appApiKeysAPI.create.mockResolvedValue({
    id: 'k_new',
    name: 'integration-key',
    role: 'developer',
    api_key: 'knol_sk_newkey_abcdef1234',
  });
  vi.spyOn(window, 'confirm').mockReturnValue(true);
});

describe('ApiKeysPage', () => {
  it('shows loading state initially', () => {
    render(<ApiKeysPage />);
    expect(screen.getByText('Loading API keys...')).toBeTruthy();
  });

  it('renders page header after loading', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => {
      expect(screen.getByText('API Keys')).toBeTruthy();
    });
    expect(screen.getByText('Create and manage API keys for your integrations.')).toBeTruthy();
  });

  it('renders key list', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());
    expect(screen.getByText('dev-key')).toBeTruthy();
  });

  it('does not show key banner when no key was just created', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());
    expect(screen.queryByText('Your API key (shown once)')).toBeNull();
  });

  it('shows key banner with masked key after creation', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    fireEvent.submit(screen.getByText('Create Key').closest('form')!);

    await waitFor(() => {
      expect(screen.getByText('Your API key (shown once)')).toBeTruthy();
    });

    // Key should be masked - last 4 chars visible
    const codeEl = screen.getByText(/1234$/);
    expect(codeEl).toBeTruthy();
    expect(codeEl.textContent).toContain('\u2022'); // bullet character
    expect(screen.getByText('Reveal')).toBeTruthy();
    expect(screen.getByText('Copy')).toBeTruthy();
  });

  it('Reveal button shows full key', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    fireEvent.submit(screen.getByText('Create Key').closest('form')!);

    await waitFor(() => {
      expect(screen.getByText('Reveal')).toBeTruthy();
    });

    fireEvent.click(screen.getByText('Reveal'));

    expect(screen.getByText('knol_sk_newkey_abcdef1234')).toBeTruthy();
    expect(screen.getByText('Hide')).toBeTruthy();
  });

  it('Hide button masks key again', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    fireEvent.submit(screen.getByText('Create Key').closest('form')!);
    await waitFor(() => expect(screen.getByText('Reveal')).toBeTruthy());

    fireEvent.click(screen.getByText('Reveal'));
    expect(screen.getByText('knol_sk_newkey_abcdef1234')).toBeTruthy();

    fireEvent.click(screen.getByText('Hide'));
    expect(screen.queryByText('knol_sk_newkey_abcdef1234')).toBeNull();
  });

  it('shows initial API key from signup', async () => {
    apiMocks.consumeInitialApiKey.mockReturnValue('knol_sk_initial_key_9999');
    render(<ApiKeysPage />);
    await waitFor(() => {
      expect(screen.getByText('Your API key (shown once)')).toBeTruthy();
    });
    // Key is masked by default
    expect(screen.getByText('Reveal')).toBeTruthy();
  });

  it('renders create form with correct defaults', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('Create Key')).toBeTruthy());

    const nameInput = screen.getByLabelText('Name') as HTMLInputElement;
    expect(nameInput.value).toBe('integration-key');

    const roleSelect = screen.getByLabelText('Role') as HTMLSelectElement;
    expect(roleSelect.value).toBe('developer');
  });

  it('renders Revoke buttons for active keys', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    const revokeButtons = screen.getAllByText('Revoke');
    expect(revokeButtons.length).toBe(2);
  });

  it('shows error on load failure', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValue(new Error('Auth failed'));
    render(<ApiKeysPage />);
    await waitFor(() => {
      expect(screen.getByText('Auth failed')).toBeTruthy();
    });
  });
});

describe('ApiKeysPage – per-key prefix show/hide', () => {
  it('shows masked key_prefix by default', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    // Both keys have key_prefix and should show masked versions with Show buttons
    const showButtons = screen.getAllByText('Show');
    expect(showButtons.length).toBe(2);

    // Prefix should be masked (contains bullets)
    const maskedEls = screen.getAllByText(/\u2022/);
    expect(maskedEls.length).toBeGreaterThanOrEqual(2);
  });

  it('clicking Show reveals key_prefix', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    const showButtons = screen.getAllByText('Show');
    fireEvent.click(showButtons[0]);

    // First key prefix should be revealed
    expect(screen.getByText('knol_sk_...ab12')).toBeTruthy();
    // Button should now say Hide
    expect(screen.getByText('Hide')).toBeTruthy();
  });

  it('clicking Hide masks key_prefix again', async () => {
    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('prod-key')).toBeTruthy());

    const showButtons = screen.getAllByText('Show');
    fireEvent.click(showButtons[0]);
    expect(screen.getByText('knol_sk_...ab12')).toBeTruthy();

    fireEvent.click(screen.getByText('Hide'));
    expect(screen.queryByText('knol_sk_...ab12')).toBeNull();
  });

  it('does not show prefix toggle for keys without key_prefix', async () => {
    apiMocks.appApiKeysAPI.list.mockResolvedValue([
      { id: 'k1', name: 'no-prefix-key', role: 'admin', active: true, created_at: '2026-01-01' },
    ]);

    render(<ApiKeysPage />);
    await waitFor(() => expect(screen.getByText('no-prefix-key')).toBeTruthy());

    // No Show button since there's no key_prefix
    expect(screen.queryByText('Show')).toBeNull();
  });
});
