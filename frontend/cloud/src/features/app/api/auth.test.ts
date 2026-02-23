import { describe, it, expect, vi, beforeEach } from 'vitest';

const sessionMocks = vi.hoisted(() => ({
  setAppAuthSession: vi.fn(),
  setAppProfile: vi.fn(),
  clearAppAuthSession: vi.fn(),
}));

const clientMocks = vi.hoisted(() => ({
  apiFetch: vi.fn(),
}));

vi.mock('./session', () => ({
  setAppAuthSession: sessionMocks.setAppAuthSession,
  setAppProfile: sessionMocks.setAppProfile,
  clearAppAuthSession: sessionMocks.clearAppAuthSession,
}));

vi.mock('./client', () => ({
  apiFetch: clientMocks.apiFetch,
}));

import { appAuthAPI } from './auth';

const MOCK_SIGNUP_RESPONSE = {
  token: 'jwt_tok_123',
  user: { id: 'u1', email: 'test@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  initial_api_key: 'knol_live_initial',
  expires_at: '2026-02-23T00:00:00Z',
};

const MOCK_LOGIN_RESPONSE = {
  token: 'jwt_tok_456',
  user: { id: 'u1', email: 'test@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  expires_at: '2026-02-23T00:00:00Z',
};

const MOCK_ME_RESPONSE = {
  user: { id: 'u1', email: 'test@test.com', role: 'owner', tenant_id: 't1' },
  tenant: { id: 't1', name: 'TestCo', slug: 'testco', plan: 'free' },
  gateway_base_url: 'https://gateway.example.com',
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('appAuthAPI', () => {
  it('signup calls POST /app/auth/signup', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_SIGNUP_RESPONSE);

    const payload = { company_name: 'TestCo', full_name: 'Test', email: 'test@test.com', password: 'StrongPass1!@' };
    await appAuthAPI.signup(payload);

    expect(clientMocks.apiFetch).toHaveBeenCalledWith('/app/auth/signup', {
      method: 'POST',
      skipAuth: true,
      skipRedirect: true,
      body: JSON.stringify(payload),
    });
  });

  it('signup stores session with initial API key', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_SIGNUP_RESPONSE);

    await appAuthAPI.signup({ company_name: 'Co', full_name: 'T', email: 'a@b.com', password: 'x' });

    expect(sessionMocks.setAppAuthSession).toHaveBeenCalledWith(
      'jwt_tok_123',
      MOCK_SIGNUP_RESPONSE.user,
      MOCK_SIGNUP_RESPONSE.tenant,
      'knol_live_initial',
    );
  });

  it('login calls POST /app/auth/login', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_LOGIN_RESPONSE);

    await appAuthAPI.login('test@test.com', 'password123');

    expect(clientMocks.apiFetch).toHaveBeenCalledWith('/app/auth/login', {
      method: 'POST',
      skipAuth: true,
      skipRedirect: true,
      body: JSON.stringify({ email: 'test@test.com', password: 'password123' }),
    });
  });

  it('login stores session without API key', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_LOGIN_RESPONSE);

    await appAuthAPI.login('test@test.com', 'password123');

    expect(sessionMocks.setAppAuthSession).toHaveBeenCalledWith(
      'jwt_tok_456',
      MOCK_LOGIN_RESPONSE.user,
      MOCK_LOGIN_RESPONSE.tenant,
    );
  });

  it('me calls GET /app/auth/me and updates profile', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_ME_RESPONSE);

    const result = await appAuthAPI.me();

    expect(clientMocks.apiFetch).toHaveBeenCalledWith('/app/auth/me', {
      method: 'GET',
      skipAuth: true,
      skipRedirect: true,
    });
    expect(sessionMocks.setAppProfile).toHaveBeenCalledWith(
      MOCK_ME_RESPONSE.user,
      MOCK_ME_RESPONSE.tenant,
    );
    expect(result.gateway_base_url).toBe('https://gateway.example.com');
  });

  it('logout calls POST and clears session', async () => {
    clientMocks.apiFetch.mockResolvedValue({});

    await appAuthAPI.logout();

    expect(clientMocks.apiFetch).toHaveBeenCalledWith('/app/auth/logout', { method: 'POST' });
    expect(sessionMocks.clearAppAuthSession).toHaveBeenCalled();
  });

  it('all auth methods use skipAuth and skipRedirect', async () => {
    clientMocks.apiFetch.mockResolvedValue(MOCK_SIGNUP_RESPONSE);
    await appAuthAPI.signup({ company_name: 'Co', full_name: 'T', email: 'a@b.com', password: 'x' });
    expect(clientMocks.apiFetch.mock.calls[0][1]).toMatchObject({ skipAuth: true, skipRedirect: true });

    clientMocks.apiFetch.mockResolvedValue(MOCK_LOGIN_RESPONSE);
    await appAuthAPI.login('a@b.com', 'x');
    expect(clientMocks.apiFetch.mock.calls[1][1]).toMatchObject({ skipAuth: true, skipRedirect: true });

    clientMocks.apiFetch.mockResolvedValue(MOCK_ME_RESPONSE);
    await appAuthAPI.me();
    expect(clientMocks.apiFetch.mock.calls[2][1]).toMatchObject({ skipAuth: true, skipRedirect: true });
  });
});
