import { resolveAppApiUrl } from '@/config/urls';
import { clearAppAuthSession, getAppAuthToken } from './session';

const API_URL = resolveAppApiUrl();

export type FetchOptions = RequestInit & {
  skipAuth?: boolean;
  skipRedirect?: boolean;
};

export async function apiFetch<T = any>(endpoint: string, options: FetchOptions = {}): Promise<T> {
  const { skipAuth = false, skipRedirect = false, ...fetchOptions } = options;
  const url = `${API_URL}${endpoint}`;

  const headers = new Headers(fetchOptions.headers || {});
  headers.set('Content-Type', 'application/json');

  if (!skipAuth) {
    const token = getAppAuthToken();
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
  }

  const response = await fetch(url, {
    ...fetchOptions,
    headers,
    credentials: 'include',
  });

  if (response.status === 401) {
    if (!skipRedirect && typeof window !== 'undefined') {
      clearAppAuthSession();
      window.location.href = '/login';
    }
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    let message = `API Error ${response.status}`;
    const contentType = response.headers.get('content-type') || '';
    if (contentType.includes('application/json')) {
      const body = await response.json();
      if (body?.error) {
        message = body.error;
      }
    } else {
      const text = await response.text();
      if (text) {
        message = text;
      }
    }
    throw new Error(message);
  }

  const contentType = response.headers.get('content-type') || '';
  if (contentType.includes('application/json')) {
    return response.json() as Promise<T>;
  }
  return response.text() as unknown as T;
}
