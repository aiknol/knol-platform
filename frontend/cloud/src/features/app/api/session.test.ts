import { describe, it, expect } from 'vitest';
import {
  getAppAuthToken,
  getAppAuthUser,
  getAppTenant,
  setAppAuthSession,
  setAppProfile,
  clearAppAuthSession,
  consumeInitialApiKey,
} from './session';

const mockUser = {
  id: 'user_1',
  email: 'test@example.com',
  full_name: 'Test User',
  role: 'owner',
  tenant_id: 'tenant_1',
  enabled: true,
};

const mockTenant = {
  id: 'tenant_1',
  name: 'Test Co',
  slug: 'test-co',
  plan: 'free',
};

describe('session', () => {
  it('getAppAuthToken always returns null (cookie-based)', () => {
    expect(getAppAuthToken()).toBeNull();
  });

  it('getAppAuthToken clears legacy storage', () => {
    sessionStorage.setItem('app_token', 'old_token');
    localStorage.setItem('app_token', 'old_token');
    getAppAuthToken();
    expect(sessionStorage.getItem('app_token')).toBeNull();
    expect(localStorage.getItem('app_token')).toBeNull();
  });

  it('setAppAuthSession stores user and tenant', () => {
    setAppAuthSession('token', mockUser as any, mockTenant as any);
    expect(sessionStorage.getItem('app_user')).toBeTruthy();
    expect(sessionStorage.getItem('app_tenant')).toBeTruthy();
    const stored = JSON.parse(sessionStorage.getItem('app_user')!);
    expect(stored.email).toBe('test@example.com');
  });

  it('setAppAuthSession stores initial API key', () => {
    setAppAuthSession('token', mockUser as any, mockTenant as any, 'knol_live_test123');
    expect(sessionStorage.getItem('app_initial_api_key')).toBe('knol_live_test123');
  });

  it('setAppAuthSession without API key does not set key', () => {
    setAppAuthSession('token', mockUser as any, mockTenant as any);
    expect(sessionStorage.getItem('app_initial_api_key')).toBeNull();
  });

  it('setAppAuthSession cleans legacy localStorage', () => {
    localStorage.setItem('app_token', 'old');
    localStorage.setItem('app_user', 'old');
    localStorage.setItem('app_tenant', 'old');
    setAppAuthSession('token', mockUser as any, mockTenant as any);
    expect(localStorage.getItem('app_token')).toBeNull();
    expect(localStorage.getItem('app_user')).toBeNull();
    expect(localStorage.getItem('app_tenant')).toBeNull();
  });

  it('getAppAuthUser parses JSON', () => {
    sessionStorage.setItem('app_user', JSON.stringify(mockUser));
    const user = getAppAuthUser();
    expect(user).toBeTruthy();
    expect(user!.email).toBe('test@example.com');
  });

  it('getAppAuthUser returns null for invalid JSON', () => {
    sessionStorage.setItem('app_user', 'not-valid-json');
    expect(getAppAuthUser()).toBeNull();
  });

  it('clearAppAuthSession removes all storage', () => {
    sessionStorage.setItem('app_token', 'x');
    sessionStorage.setItem('app_user', 'x');
    sessionStorage.setItem('app_tenant', 'x');
    sessionStorage.setItem('app_initial_api_key', 'x');
    localStorage.setItem('app_token', 'x');
    localStorage.setItem('app_user', 'x');
    localStorage.setItem('app_tenant', 'x');
    clearAppAuthSession();
    expect(sessionStorage.getItem('app_token')).toBeNull();
    expect(sessionStorage.getItem('app_user')).toBeNull();
    expect(sessionStorage.getItem('app_tenant')).toBeNull();
    expect(sessionStorage.getItem('app_initial_api_key')).toBeNull();
    expect(localStorage.getItem('app_token')).toBeNull();
    expect(localStorage.getItem('app_user')).toBeNull();
    expect(localStorage.getItem('app_tenant')).toBeNull();
  });

  it('consumeInitialApiKey returns and removes key', () => {
    sessionStorage.setItem('app_initial_api_key', 'knol_live_abc');
    const key = consumeInitialApiKey();
    expect(key).toBe('knol_live_abc');
    expect(sessionStorage.getItem('app_initial_api_key')).toBeNull();
  });

  it('consumeInitialApiKey returns null when empty', () => {
    expect(consumeInitialApiKey()).toBeNull();
  });
});
