import type { AppUser, TenantProfile } from './types';
import { apiFetch } from './client';
import { clearAppAuthSession, setAppAuthSession, setAppProfile } from './session';

export interface SignupPayload {
  company_name: string;
  full_name: string;
  email: string;
  password: string;
}

export interface SignupResponse {
  token: string;
  user: AppUser;
  tenant: TenantProfile;
  initial_api_key?: string;
  expires_at: string;
}

export interface LoginResponse {
  token: string;
  user: AppUser;
  tenant: TenantProfile;
  expires_at: string;
  totp_required?: boolean;
  totp_token?: string;
}

export interface MeResponse {
  user: AppUser;
  tenant: TenantProfile;
  gateway_base_url: string;
}

export const appAuthAPI = {
  signup: async (payload: SignupPayload) => {
    const data = await apiFetch<Partial<SignupResponse>>('/app/auth/signup', {
      method: 'POST',
      skipAuth: true,
      skipRedirect: true,
      body: JSON.stringify(payload),
    });
    if (data?.token && data?.user && data?.tenant) {
      setAppAuthSession(data.token, data.user, data.tenant, data.initial_api_key);
    }
    return data as SignupResponse;
  },

  login: async (email: string, password: string) => {
    const data = await apiFetch<Partial<LoginResponse>>('/app/auth/login', {
      method: 'POST',
      skipAuth: true,
      skipRedirect: true,
      body: JSON.stringify({ email, password }),
    });
    // If TOTP is required, return early without setting session
    if (data?.totp_required) {
      return data as LoginResponse;
    }
    if (data?.token && data?.user && data?.tenant) {
      setAppAuthSession(data.token, data.user, data.tenant);
    }
    return data as LoginResponse;
  },

  verifyTotp: async (totpToken: string, code: string) => {
    const data = await apiFetch<LoginResponse>('/app/auth/totp/verify', {
      method: 'POST',
      skipAuth: true,
      skipRedirect: true,
      body: JSON.stringify({ totp_token: totpToken, code }),
    });
    if (data?.token && data?.user && data?.tenant) {
      setAppAuthSession(data.token, data.user, data.tenant);
    }
    return data;
  },

  me: async () => {
    const data = await apiFetch<MeResponse>('/app/auth/me', {
      method: 'GET',
      skipAuth: true,
      skipRedirect: true,
    });
    if (data?.user && data?.tenant) {
      setAppProfile(data.user, data.tenant);
    }
    return data;
  },

  refresh: async () => {
    return apiFetch<{ token: string; expires_at: string }>('/app/auth/refresh', {
      method: 'POST',
    });
  },

  logout: async () => {
    await apiFetch('/app/auth/logout', { method: 'POST' });
    clearAppAuthSession();
  },
};
